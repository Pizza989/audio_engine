use std::collections::HashMap;

use audio_buffer::buffers::interleaved::InterleavedBuffer;
use audio_buffer::core::Buffer;
use audio_buffer::core::BufferMut;
use audio_buffer::core::axis::BufferAxisMut;
use audio_buffer::dasp;
use daggy::petgraph::visit::IntoNeighbors;
use daggy::{Dag, EdgeIndex, NodeIndex, Walker, petgraph};
use time::FrameTime;
use time::SampleRate;

use crate::buffer_pool::BufferArena;
use crate::error::GraphError;
use crate::pin_matrix::PinMatrix;
use crate::processor::AudioProcessor;
use crate::processor::ProcessorConfiguration;

pub use daggy;

pub mod buffer_pool;
pub mod error;
pub mod pin_matrix;
pub mod processor;

pub struct Connection {
    matrix: PinMatrix,
}

// INVARIANT: "PinMatrix Validity"
// The matrix stored on graph edges must always have
// as many input channels as the source has output
// channels and as many output channels as the
// destination has input channels.
// INVARIANT: "Output Validity"
// `self.output` must always be a valid index
pub struct AudioGraph<T, N>
where
    T: dasp::Sample,
    N: AudioProcessor<T>,
{
    dag: Dag<N, Connection>,
    execution_order: Vec<NodeIndex>,
    buffer_arena: BufferArena<T>,

    // stores cached buffer -> last consumer
    // this means that during processing a cached buffer
    // can be released once the last consumer has been
    // processed
    buffer_lifetimes: HashMap<NodeIndex, NodeIndex>,
    block_size: FrameTime,
    sample_rate: SampleRate,
    output: NodeIndex,
}

impl<T, N> AudioGraph<T, N>
where
    T: dasp::Sample + 'static,
    N: processor::AudioProcessor<T>,
{
    pub fn new(node: N, sample_rate: SampleRate, block_size: FrameTime) -> (Self, NodeIndex) {
        let mut graph = Self {
            dag: Dag::new(),
            execution_order: vec![],
            buffer_arena: BufferArena::new(sample_rate),
            sample_rate: sample_rate,
            block_size,
            output: 0.into(),
            buffer_lifetimes: HashMap::new(),
        };

        let node_idx = graph.add_node(node);

        graph
            .set_output_index(node_idx)
            .expect("node_idx is not dangling");

        graph.update_buffer_pool();
        graph.recompute_execution_order();

        (graph, node_idx)
    }

    // Invalid States:
    // - src or dst indices are dangling
    // - the pin_matrix configuration doesn't match
    //   with the src and dst nodes configurations
    // - the connection would cycle
    // - the execution order is outdated after
    //   adding a new connection
    pub fn add_connection(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
        pin_matrix: PinMatrix,
    ) -> Result<EdgeIndex, GraphError> {
        let src_node = self
            .dag
            .node_weight(src)
            .ok_or(GraphError::WouldInvalidNode(src))?;

        let dst_node = self
            .dag
            .node_weight(dst)
            .ok_or(GraphError::WouldInvalidNode(dst))?;

        let src_config = src_node.config();
        let dst_config = dst_node.config();
        if !(pin_matrix.input_channels() == src_config.num_output_channels
            && pin_matrix.output_channels() == dst_config.num_input_channels)
        {
            return Err(GraphError::WouldInvalidPinMatrix);
        }

        let edge_index = self
            .dag
            .add_edge(src, dst, Connection { matrix: pin_matrix })?;

        self.recompute_execution_order();
        Ok(edge_index)
    }

    pub fn update_connection(
        &mut self,
        edge_index: EdgeIndex,
        matrix: PinMatrix,
    ) -> Option<PinMatrix> {
        let (start_config, end_config) = {
            let (start, end) = self.dag.edge_endpoints(edge_index)?;
            (
                self.dag.node_weight(start)?.config(),
                self.dag.node_weight(end)?.config(),
            )
        };

        if matrix.input_channels() == start_config.num_output_channels
            && matrix.output_channels() == end_config.num_input_channels
        {
            let connection = self.dag.edge_weight_mut(edge_index)?;
            let old_matrix = connection.matrix.clone();

            connection.matrix = matrix;
            Some(old_matrix)
        } else {
            None
        }
    }

    // Invalid States:
    // - the execution order is outdated
    //   after removing a connection
    pub fn remove_connection(&mut self, edge_index: EdgeIndex) -> Option<Connection> {
        let connection = self.dag.remove_edge(edge_index);
        self.recompute_execution_order();
        connection
    }

    // Invalid States:
    // - index could be self.input
    // - index could be self.output
    // - index could be part of a connection
    pub fn remove_node(&mut self, index: NodeIndex) -> Result<Option<N>, GraphError> {
        if index == self.output {
            return Err(GraphError::WouldInvalidNode(self.output));
        } else if self.dag.neighbors(index).peekable().peek().is_some() {
            return Err(GraphError::WouldDanglingNodeInConnection);
        }

        Ok(self.dag.remove_node(index))
    }

    // Invalid States:
    // - index could be dangling
    pub fn set_output_index(&mut self, index: NodeIndex) -> Result<(), GraphError> {
        if self.dag.node_weight(index).is_some() {
            self.output = index;
            Ok(())
        } else {
            Err(GraphError::WouldInvalidNode(index))
        }
    }

    // Invalid States:
    // - buffer pool could be out of date
    pub fn set_block_size(&mut self, block_size: FrameTime) -> FrameTime {
        let old = self.block_size;
        self.block_size = block_size;

        self.update_buffer_pool();
        old
    }
}

impl<T, N> AudioGraph<T, N>
where
    T: dasp::Sample + 'static,
    N: processor::AudioProcessor<T>,
{
    // PRECONDITIONS:
    // a) self.execution_order is an up to date topological ordering of the graph
    // b) self.buffer_pool can provide enough buffers with the right configuration
    // POSTCONDITIONS:
    // a) self.buffer_pool can provide enough buffers again for next block
    pub fn process_block(
        &mut self,
        inputs: &HashMap<NodeIndex, &InterleavedBuffer<T>>,
        output: &mut InterleavedBuffer<T>,
    ) {
        let mut node_outputs: HashMap<NodeIndex, InterleavedBuffer<T>> = HashMap::new();

        for &node_idx in &self.execution_order {
            let node_config = self
                .dag
                .node_weight(node_idx)
                .expect("precondition a")
                .config();

            if let Some(external_input) = inputs.get(&node_idx) {
                self.dag
                    .node_weight_mut(node_idx)
                    .expect("precondition a")
                    .process_unchecked(
                        &external_input,
                        if node_idx == self.output {
                            output
                        } else {
                            let node_output = self
                                .buffer_arena
                                .take(node_config.num_output_channels, self.block_size)
                                .expect("precondition b");
                            node_outputs.insert(node_idx, node_output);
                            node_outputs.get_mut(&node_idx).unwrap()
                        },
                    );
            } else {
                let mut mixed = self
                    .buffer_arena
                    .take(node_config.num_input_channels, self.block_size)
                    .expect("precondition b");

                self.mix_parents_from_cache(node_idx, &mut node_outputs, &mut mixed);

                self.dag
                    .node_weight_mut(node_idx)
                    .expect("precondition a")
                    .process_unchecked(
                        &mixed,
                        if node_idx == self.output {
                            output
                        } else {
                            let node_output = self
                                .buffer_arena
                                .take(node_config.num_output_channels, self.block_size)
                                .expect("precondition b");
                            node_outputs.insert(node_idx, node_output);
                            node_outputs.get_mut(&node_idx).unwrap()
                        },
                    );

                mixed.set_to_equilibrium();
                self.buffer_arena.release(mixed);
            };

            if node_idx == self.output {
                break;
            }

            for (&cached, &last_consumer) in &self.buffer_lifetimes {
                if last_consumer == node_idx {
                    if let Some(mut buffer) = node_outputs.remove(&cached) {
                        buffer.set_to_equilibrium();
                        self.buffer_arena.release(buffer);
                    }
                }
            }
        }

        for (_index, mut buffer) in node_outputs.drain() {
            buffer.set_to_equilibrium();
            self.buffer_arena.release(buffer);
        }
    }

    // PRECONDITIONS:
    // a) parent_outputs_cache must contain a buffer for every parent
    // b) the size of the output buffer and all buffers in parent_outputs_cache
    //    must be the same
    fn mix_parents_from_cache(
        &self,
        index: NodeIndex,
        parent_outputs_cache: &mut HashMap<NodeIndex, InterleavedBuffer<T>>,
        output: &mut InterleavedBuffer<T>,
    ) {
        for (edge, parent) in self.dag.parents(index).iter(&self.dag) {
            let parent_out = parent_outputs_cache
                .get(&parent)
                .expect("must be cached due to precondition a");

            let connection = self
                .dag
                .edge_weight(edge)
                .expect("was just returned by self.dag.parents call");

            for (parent_channel_idx, mixed_channel_idx) in connection.matrix.channel_connections() {
                let parent_channel = parent_out
                    .get_channel(parent_channel_idx)
                    .expect("must be valid due to PinMatrix Validity");

                output.map_channels_mut(
                    |mut mixed_channel, _| {
                        mixed_channel.map_samples_mut(
                            |out_sample, sample_index| match parent_channel.get(sample_index) {
                                Some(in_sample) => {
                                    *out_sample = out_sample
                                        .add_amp(dasp::Sample::to_signed_sample(*in_sample));
                                    Some(())
                                }
                                None => {
                                    unreachable!("precondition b")
                                }
                            },
                            None,
                        );
                        None::<usize>
                    },
                    Some(mixed_channel_idx),
                );
            }
        }
    }

    // TODO: free buffers that aren't neccessary
    fn update_buffer_pool(&mut self) {
        let mut buffers_required: HashMap<usize, usize> = HashMap::new();

        fn increment(map: &mut HashMap<usize, usize>, channels: usize) {
            match map.get_mut(&channels) {
                Some(number) => *number = *number + 1usize,
                None => {
                    map.insert(channels, 1);
                }
            }
        }

        for node in self.dag.node_weights_mut() {
            let config = node.config();
            increment(&mut buffers_required, config.num_output_channels);
            increment(&mut buffers_required, config.num_input_channels);
        }

        for (channels, amount) in buffers_required {
            self.buffer_arena
                .ensure_capacity(channels, self.block_size, amount);
        }
    }

    fn recompute_execution_order(&mut self) {
        self.execution_order = petgraph::algo::toposort(&self.dag, None)
            .expect("graph must be acyclic")
            .into_iter()
            .collect();

        self.compute_buffer_lifetimes();
    }

    fn compute_buffer_lifetimes(&mut self) {
        self.buffer_lifetimes.clear();

        for &node_idx in &self.execution_order {
            for (_, parent) in self.dag.parents(node_idx).iter(&self.dag) {
                // Update the last user of this parent's buffer
                self.buffer_lifetimes.insert(parent, node_idx);
            }
        }
    }

    pub fn add_node(&mut self, weight: N) -> NodeIndex {
        self.update_buffer_pool();
        self.dag.add_node(weight)
    }

    pub fn get_output(&self) -> &N {
        self.dag
            .node_weight(self.output)
            .expect("invariant: self.output must always be valid")
    }

    pub fn get_output_index(&self) -> NodeIndex {
        self.output
    }

    pub fn get_node_config(&self, index: NodeIndex) -> Option<ProcessorConfiguration> {
        self.dag.node_weight(index).map(|node| node.config())
    }

    // TODO: might be problematic with reconfiguration
    pub fn get_dag(&self) -> &Dag<N, Connection> {
        &self.dag
    }

    pub fn get_node(&self, index: NodeIndex) -> Option<&N> {
        self.dag.node_weight(index)
    }

    pub fn get_node_mut(&mut self, index: NodeIndex) -> Option<&mut N> {
        self.dag.node_weight_mut(index)
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}
