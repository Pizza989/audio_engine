use std::collections::HashMap;

use audio_buffer::core::BufferMut;
use audio_buffer::dasp;
use audio_buffer::{buffers::fixed_frames::FixedFrameBuffer, core::Buffer};
use daggy::petgraph::visit::IntoNeighbors;
use daggy::{Dag, EdgeIndex, NodeIndex, Walker, petgraph};

use crate::buffer_pool::BufferPool;
use crate::error::{GraphError, ProcessingError};
use crate::pin_matrix::PinMatrix;

pub mod buffer_pool;
pub mod error;
pub mod pin_matrix;

pub trait AudioProcessor<T: dasp::Sample, const BLOCK_SIZE: usize> {
    fn process(
        &mut self,
        input: &FixedFrameBuffer<T, BLOCK_SIZE>,
        output: &mut FixedFrameBuffer<T, BLOCK_SIZE>,
    ) -> Result<(), ProcessingError> {
        if self.input_channels() != input.channels() || self.output_channels() != output.channels()
        {
            return Err(ProcessingError::InvalidBuffers);
        } else {
            self.process_unchecked(input, output);
            Ok(())
        }
    }

    fn process_unchecked(
        &mut self,
        input: &FixedFrameBuffer<T, BLOCK_SIZE>,
        output: &mut FixedFrameBuffer<T, BLOCK_SIZE>,
    );

    fn input_channels(&self) -> usize;
    fn output_channels(&self) -> usize;
}

pub struct AudioNode<T, const BLOCK_SIZE: usize>
where
    T: audio_buffer::dasp::Sample,
{
    processor: Box<dyn AudioProcessor<T, BLOCK_SIZE>>,
}

pub struct Connection {
    matrix: PinMatrix,
}

pub struct AudioGraph<T, const BLOCK_SIZE: usize>
where
    T: dasp::Sample,
{
    dag: Dag<AudioNode<T, BLOCK_SIZE>, Connection>,
    execution_order: Vec<NodeIndex>,
    buffer_pool: BufferPool<T, BLOCK_SIZE>,
    sample_rate: usize,
    input: NodeIndex,
    output: NodeIndex,
}

impl<T: dasp::Sample + 'static, const BLOCK_SIZE: usize> AudioGraph<T, BLOCK_SIZE> {
    pub fn new(sample_rate: usize, node: AudioNode<T, BLOCK_SIZE>) -> Self {
        let mut graph = Self {
            dag: Dag::new(),
            execution_order: vec![],
            buffer_pool: BufferPool::new(sample_rate),
            sample_rate: sample_rate,
            input: 0.into(),
            output: 0.into(),
        };

        let node_idx = graph.add_node(node);

        graph.set_input(node_idx).expect("node_idx is not dangline");
        graph
            .set_output(node_idx)
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

        if !(pin_matrix.input_channels() == src_node.processor.output_channels()
            && pin_matrix.output_channels() == dst_node.processor.input_channels())
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
    pub fn remove_node(
        &mut self,
        index: NodeIndex,
    ) -> Result<Option<AudioNode<T, BLOCK_SIZE>>, GraphError> {
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
    pub fn set_input(&mut self, index: NodeIndex) -> Result<(), GraphError> {
        if self.dag.node_weight(index).is_some() {
            self.input = index;
            Ok(())
        } else {
            Err(GraphError::WouldInvalidNode(index))
        }
    }

    // Invalid States:
    // - index could be dangling
    pub fn set_output(&mut self, index: NodeIndex) -> Result<(), GraphError> {
        if self.dag.node_weight(index).is_some() {
            self.output = index;
            Ok(())
        } else {
            Err(GraphError::WouldInvalidNode(index))
        }
    }
}

impl<T: dasp::Sample + 'static, const BLOCK_SIZE: usize> AudioGraph<T, BLOCK_SIZE> {
    fn process_linear(
        &mut self,
        input: &FixedFrameBuffer<T, BLOCK_SIZE>,
        output: &mut FixedFrameBuffer<T, BLOCK_SIZE>,
    ) {
        let mut node_outputs: HashMap<NodeIndex, FixedFrameBuffer<T, BLOCK_SIZE>> = HashMap::new();

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
                let mut mixed = self.buffer_pool.aquire(node.processor.input_channels());

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
                            |mixed_channel, _| {
                                for (in_sample, out_sample) in
                                    parent_channel.iter().zip(mixed_channel.iter_mut())
                                {
                                    *out_sample = out_sample
                                        .add_amp(dasp::Sample::to_signed_sample(*in_sample));
                                }
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
                    .processor
                    .process_unchecked(node_input.as_ref(), output);

                return;
            } else {
                let mut node_output = self.buffer_pool.aquire(node.processor.output_channels());
                self.dag
                    .node_weight_mut(node_idx)
                    .expect("must be valid due to invariants")
                    .processor
                    .process_unchecked(node_input.as_ref(), &mut node_output);

                node_outputs.insert(node_idx, node_output);
            };
        }
    }

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
            increment(&mut buffers_required, node.processor.output_channels());
            increment(&mut buffers_required, node.processor.input_channels());
        }

        for (channels, amount) in buffers_required {
            self.buffer_pool.ensure_capacity(channels, amount);
        }
    }

    fn recompute_execution_order(&mut self) {
        self.execution_order = petgraph::algo::toposort(&self.dag, None)
            .expect("graph must be acyclic")
            .into_iter()
            .rev()
            .collect();
    }

    pub fn add_node(&mut self, weight: AudioNode<T, BLOCK_SIZE>) -> NodeIndex {
        self.update_buffer_pool();
        self.dag.add_node(weight)
    }

    pub fn get_input(&self) -> &AudioNode<T, BLOCK_SIZE> {
        self.dag
            .node_weight(self.input)
            .expect("invariant: self.input must always be valid")
    }

    pub fn get_output(&self) -> &AudioNode<T, BLOCK_SIZE> {
        self.dag
            .node_weight(self.output)
            .expect("invariant: self.output must always be valid")
    }

    pub fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}

impl<T: dasp::Sample + 'static, const BLOCK_SIZE: usize> AudioProcessor<T, BLOCK_SIZE>
    for AudioGraph<T, BLOCK_SIZE>
{
    fn process_unchecked(
        &mut self,
        input: &FixedFrameBuffer<T, BLOCK_SIZE>,
        output: &mut FixedFrameBuffer<T, BLOCK_SIZE>,
    ) {
        self.process_linear(input, output);
    }

    fn input_channels(&self) -> usize {
        self.get_input().processor.input_channels()
    }

    fn output_channels(&self) -> usize {
        self.get_output().processor.output_channels()
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
