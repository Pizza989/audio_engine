use std::marker::PhantomData;

use audio_buffer::core::io::mix_buffers;
use audio_buffer::dasp;
use audio_buffer::{buffers::interleaved_dynamic::InterleavedDynamicBuffer, core::Buffer};
use daggy::{Dag, EdgeIndex, NodeIndex, Walker};

use crate::error::{GraphError, ProcessingError};

pub mod error;

pub trait AudioProcessor<T: dasp::Sample> {
    fn process(
        &mut self,
        input: &InterleavedDynamicBuffer<T>,
        output: &mut InterleavedDynamicBuffer<T>,
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
        input: &InterleavedDynamicBuffer<T>,
        output: &mut InterleavedDynamicBuffer<T>,
    );

    fn input_channels(&self) -> usize;
    fn output_channels(&self) -> usize;
}

pub struct Connection {}

pub struct AudioGraph<T, N>
where
    T: dasp::Sample,
    N: AudioProcessor<T>,
{
    dag: Dag<N, Connection>,
    sample_rate: usize,
    block_size_frames: usize,
    input: NodeIndex,
    output: NodeIndex,
    _marker: PhantomData<T>,
}

impl<T: dasp::Sample + 'static, N: AudioProcessor<T>> AudioGraph<T, N> {
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

        if !(src_node.output_channels() == dst_node.input_channels()) {
            return Err(GraphError::InvalidConnection(
                src_node.output_channels(),
                dst_node.input_channels(),
            ));
        }

        Ok(self.dag.add_edge(src, dst, Connection {})?)
    }

    /// Edge Case:
    ///
    /// a (input) -> b -> c (output)
    ///              d ---^
    ///
    /// this graph is possible. with the current implementation
    /// d would just get an empty input buffer as it is not
    /// self.input and has no parents
    fn process_from(
        &mut self,
        input: &InterleavedDynamicBuffer<T>,
        output: &mut InterleavedDynamicBuffer<T>,
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

            (processor.input_channels(), processor.output_channels())
        };

        debug_assert_eq!(input.channels(), input_channels);
        debug_assert_eq!(output.channels(), output_channels);

        let input = if node == self.input {
            RefOrOwned::Ref(input)
        } else {
            let mut mixed_input =
                InterleavedDynamicBuffer::<T>::new(input_channels, self.sample_rate); // TODO: is this filled with T::EQUILIBRIUM?
            let mut inputs = self.dag.parents(node);

            while let Some((_audio_edge, audio_node)) = inputs.walk_next(&self.dag) {
                let mut temp_out = InterleavedDynamicBuffer::new(input_channels, self.sample_rate);

                self.process_from(input, &mut temp_out, audio_node)?;
                mix_buffers(&temp_out, &mut mixed_input);
            }
            RefOrOwned::Owned(mixed_input)
        };

        self.dag
            .node_weight_mut(node)
            .expect("invariant: `node` must always be valid")
            .process(input.as_ref(), output)?;
        Ok(())
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
}

impl<T: dasp::Sample + 'static, N: AudioProcessor<T>> AudioProcessor<T> for AudioGraph<T, N> {
    fn process_unchecked(
        &mut self,
        input: &InterleavedDynamicBuffer<T>,
        output: &mut InterleavedDynamicBuffer<T>,
    ) {
        self.process_from(input, output, self.output).unwrap()
    }

    fn input_channels(&self) -> usize {
        self.get_input().input_channels()
    }

    fn output_channels(&self) -> usize {
        self.get_output().output_channels()
    }
}
