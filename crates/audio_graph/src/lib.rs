use std::collections::HashMap;

use audio_buffer::buffers::interleaved::InterleavedBuffer;
use audio_buffer::core::Buffer;
use audio_buffer::core::BufferMut;
use audio_buffer::core::axis::BufferAxisMut;
use audio_buffer::dasp;
use daggy::petgraph::visit::IntoNeighbors;
use daggy::{Dag, EdgeIndex, NodeIndex, Walker, petgraph};
use time::SampleRate;

use crate::buffer_pool::BufferPool;
use crate::error::GraphError;
use crate::pin_matrix::PinMatrix;
use crate::processor::AudioProcessor;

pub use daggy;

pub mod buffer_pool;
pub mod error;
pub mod pin_matrix;
pub mod processor;

pub struct Connection {
    matrix: PinMatrix,
}

pub struct AudioGraph<T, N>
where
    T: dasp::Sample,
    N: AudioProcessor<T>,
{
    dag: Dag<N, Connection>,
    execution_order: Vec<NodeIndex>,
    buffer_pool: BufferPool<T>,
    block_size: usize,
    sample_rate: SampleRate,
    input: NodeIndex,
    output: NodeIndex,
}

impl<T, N> AudioGraph<T, N>
where
    T: dasp::Sample + 'static,
    N: processor::AudioProcessor<T>,
{
    pub fn new(node: N, sample_rate: SampleRate, block_size: usize) -> Self {
        let mut graph = Self {
            dag: Dag::new(),
            execution_order: vec![],
            buffer_pool: BufferPool::new(sample_rate),
            sample_rate: sample_rate,
            block_size,
            input: 0.into(),
            output: 0.into(),
        };

        let node_idx = graph.add_node(node);

        graph
            .set_input_index(node_idx)
            .expect("node_idx is not dangline");
        graph
            .set_output_index(node_idx)
            .expect("node_idx is not dangling");

        graph.update_buffer_pool();
        graph.recompute_execution_order();

        graph
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

        if !(pin_matrix.input_channels() == src_node.output_channels()
            && pin_matrix.output_channels() == dst_node.input_channels())
        {
            return Err(GraphError::WouldInvalidPinMatrix);
        }

        let edge_index = self
            .dag
            .add_edge(src, dst, Connection { matrix: pin_matrix })?;

        self.recompute_execution_order();
        Ok(edge_index)
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
        } else if index == self.input {
            return Err(GraphError::WouldInvalidNode(self.input));
        } else if self.dag.neighbors(index).peekable().peek().is_some() {
            return Err(GraphError::WouldDanglingNodeInConnection);
        }

        Ok(self.dag.remove_node(index))
    }

    // Invalid States:
    // - index could be dangling
    pub fn set_input_index(&mut self, index: NodeIndex) -> Result<(), GraphError> {
        if self.dag.node_weight(index).is_some() {
            self.input = index;
            Ok(())
        } else {
            Err(GraphError::WouldInvalidNode(index))
        }
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
    pub fn set_block_size(&mut self, block_size: usize) -> usize {
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
    fn process_linear(&mut self, input: &InterleavedBuffer<T>, output: &mut InterleavedBuffer<T>) {
        let mut node_outputs: HashMap<NodeIndex, InterleavedBuffer<T>> = HashMap::new();

        for &node_idx in &self.execution_order {
            let node = self
                .dag
                .node_weight(node_idx)
                .expect("must be valid due to invariants");
            // node_input has as many channels as node_idx has input channels
            let node_input = if node_idx == self.input {
                // input must have as many channels as self.input has input channels
                RefOrOwned::Ref(input)
            } else {
                // mixed must have as many channels as node_idx has input channels
                let mut mixed = self
                    .buffer_pool
                    .aquire(node.input_channels(), self.block_size);

                for (edge, parent) in self.dag.parents(node_idx).iter(&self.dag) {
                    let parent_out = node_outputs
                        .get(&parent)
                        .expect("must be cached due to execution order");

                    let connection = self.dag.edge_weight(edge).expect("exists");

                    for (parent_channel_idx, mixed_channel_idx) in
                        connection.matrix.channel_connections()
                    {
                        let parent_channel = parent_out
                            .get_channel(parent_channel_idx)
                            .expect("must be valid due to invariants");

                        mixed.map_channels_mut(
                            |mut mixed_channel, _| {
                                mixed_channel.map_samples_mut(|out_sample, sample_index| {
                                    match parent_channel.get(sample_index) {
                                        Some(in_sample) => {
                                            *out_sample = out_sample.add_amp(
                                                dasp::Sample::to_signed_sample(*in_sample),
                                            );
                                            Some(())
                                        }
                                        None => {
                                            unreachable!("buffers should always have the same size")
                                        }
                                    }
                                });
                                None::<usize>
                            },
                            Some(mixed_channel_idx),
                        );
                    }
                }
                RefOrOwned::Owned(mixed)
            };

            // node_output has as many channels as node_idx has output channels
            if node_idx == self.output {
                self.dag
                    .node_weight_mut(node_idx)
                    .expect("must be valid due to invariants")
                    .process_unchecked(node_input.as_ref(), output);

                return;
            } else {
                let mut node_output = self
                    .buffer_pool
                    .aquire(node.output_channels(), self.block_size);
                self.dag
                    .node_weight_mut(node_idx)
                    .expect("must be valid due to invariants")
                    .process_unchecked(node_input.as_ref(), &mut node_output);

                node_outputs.insert(node_idx, node_output);
            };
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
            increment(&mut buffers_required, node.output_channels());
            increment(&mut buffers_required, node.input_channels());
        }

        for (channels, amount) in buffers_required {
            self.buffer_pool
                .ensure_capacity(channels, self.block_size, amount);
        }
    }

    fn recompute_execution_order(&mut self) {
        self.execution_order = petgraph::algo::toposort(&self.dag, None)
            .expect("graph must be acyclic")
            .into_iter()
            .rev()
            .collect();
    }

    pub fn add_node(&mut self, weight: N) -> NodeIndex {
        self.update_buffer_pool();
        self.dag.add_node(weight)
    }

    pub fn get_input(&self) -> &N {
        self.dag
            .node_weight(self.input)
            .expect("invariant: self.input must always be valid")
    }

    pub fn get_output(&self) -> &N {
        self.dag
            .node_weight(self.output)
            .expect("invariant: self.output must always be valid")
    }

    pub fn get_input_index(&self) -> NodeIndex {
        self.input
    }

    pub fn get_output_index(&self) -> NodeIndex {
        self.output
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

impl<T, N> AudioProcessor<T> for AudioGraph<T, N>
where
    T: dasp::Sample + 'static,
    N: processor::AudioProcessor<T>,
{
    fn process_unchecked(
        &mut self,
        input: &InterleavedBuffer<T>,
        output: &mut InterleavedBuffer<T>,
    ) {
        self.process_linear(input, output);
    }

    fn input_channels(&self) -> usize {
        self.get_input().input_channels()
    }

    fn output_channels(&self) -> usize {
        self.get_output().output_channels()
    }
}

enum RefOrOwned<'a, T> {
    Ref(&'a T),
    Owned(T),
}

impl<'a, T> AsRef<T> for RefOrOwned<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            RefOrOwned::Ref(r) => r,
            RefOrOwned::Owned(o) => &o,
        }
    }
}
