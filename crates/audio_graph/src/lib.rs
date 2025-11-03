use std::marker::PhantomData;

use audio_buffer::core::io::mix_buffers;
use audio_buffer::dasp;
use audio_buffer::{buffers::fixed_frames::FixedFrameBuffer, core::Buffer};
use daggy::{Dag, EdgeIndex, NodeIndex, Walker};

use crate::error::{GraphError, ProcessingError};

pub mod error;

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

pub struct Connection {}

pub struct AudioGraph<T, const BLOCK_SIZE: usize>
where
    T: dasp::Sample,
{
    dag: Dag<AudioNode<T, BLOCK_SIZE>, Connection>,
    sample_rate: usize,
    input: NodeIndex,
    output: NodeIndex,
    _marker: PhantomData<T>,
}

impl<T: dasp::Sample + 'static, const BLOCK_SIZE: usize> AudioGraph<T, BLOCK_SIZE> {
    // INVARIANT: src.output_channels == dst.input_channels
    pub fn add_connection(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
    ) -> Result<EdgeIndex, GraphError> {
        let src_node = self
            .dag
            .node_weight(src)
            .ok_or(GraphError::InvalidNode(src))?;
        let dst_node = self
            .dag
            .node_weight(dst)
            .ok_or(GraphError::InvalidNode(dst))?;

        if !(src_node.processor.output_channels() == dst_node.processor.input_channels()) {
            return Err(GraphError::InvalidConnection(
                src_node.processor.output_channels(),
                dst_node.processor.input_channels(),
            ));
        }

        Ok(self.dag.add_edge(src, dst, Connection {})?)
    }

    /// This method computes the entire processed output from a node
    /// by recursively looking for input buffers. The stop conditions
    /// either that the node is `self.input` or that it doesn't have
    /// any parents. In any other case the function will recurse to
    /// find the input buffer of the node's parents. What is passed
    /// as the input parameter of the method will be reached through
    /// recursive calls and be the input for `self.input` if it is
    /// ever found.
    //
    // Edge Case:
    //
    // a (input) -> b -> c (output)
    //              d ---^
    //
    // this graph is possible. with the current implementation
    // d would just get an empty input buffer as it is not
    // self.input and has no parents
    fn process_from(
        &mut self,
        input: &FixedFrameBuffer<T, BLOCK_SIZE>,
        output: &mut FixedFrameBuffer<T, BLOCK_SIZE>,
        node: NodeIndex,
    ) -> Result<(), ProcessingError> {
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

        let (input_channels, output_channels) = {
            let processor = &self
                .dag
                .node_weight(node)
                .expect("invariant: `node` must always be valid");

            (
                processor.processor.input_channels(),
                processor.processor.output_channels(),
            )
        };

        debug_assert_eq!(input.channels(), input_channels);
        debug_assert_eq!(output.channels(), output_channels);

        let input = if node == self.input {
            RefOrOwned::Ref(input)
        } else {
            let mut mixed_input =
                FixedFrameBuffer::<T, BLOCK_SIZE>::with_capacity(input_channels, self.sample_rate);
            let mut inputs = self.dag.parents(node);

            while let Some((_audio_edge, audio_node)) = inputs.walk_next(&self.dag) {
                let mut temp_out = FixedFrameBuffer::<T, BLOCK_SIZE>::with_capacity(
                    input_channels,
                    self.sample_rate,
                );

                self.process_from(input, &mut temp_out, audio_node)?;
                mix_buffers(&temp_out, &mut mixed_input).expect("the channels don't mismatch");
            }
            RefOrOwned::Owned(mixed_input)
        };

        self.dag
            .node_weight_mut(node)
            .expect("invariant: `node` must always be valid")
            .processor
            .process(input.as_ref(), output)?;
        Ok(())
    }

    pub fn add_node(&mut self, weight: AudioNode<T, BLOCK_SIZE>) -> NodeIndex {
        self.dag.add_node(weight)
    }

    pub fn remove_node(
        &mut self,
        index: NodeIndex,
    ) -> Result<Option<AudioNode<T, BLOCK_SIZE>>, GraphError> {
        if index == self.output || index == self.input {
            return Err(GraphError::OutputInputValidity);
        }

        Ok(self.dag.remove_node(index))
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

    pub fn set_input(&mut self, index: NodeIndex) -> Result<(), GraphError> {
        if self.dag.node_weight(index).is_some() {
            self.input = index;
            Ok(())
        } else {
            Err(GraphError::OutputInputValidity)
        }
    }

    pub fn set_output(&mut self, index: NodeIndex) -> Result<(), GraphError> {
        if self.dag.node_weight(index).is_some() {
            self.output = index;
            Ok(())
        } else {
            Err(GraphError::OutputInputValidity)
        }
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
        self.process_from(input, output, self.output).unwrap()
    }

    fn input_channels(&self) -> usize {
        self.get_input().processor.input_channels()
    }

    fn output_channels(&self) -> usize {
        self.get_output().processor.output_channels()
    }
}
