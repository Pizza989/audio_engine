use audio_buffer::{
    buffers::interleaved::InterleavedBuffer,
    core::{Buffer, io::mix_buffers},
    dasp,
};

use crate::error::ProcessingError;

pub trait AudioProcessor<T: dasp::Sample> {
    fn process(
        &mut self,
        input: &InterleavedBuffer<T>,
        output: &mut InterleavedBuffer<T>,
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
        input: &InterleavedBuffer<T>,
        output: &mut InterleavedBuffer<T>,
    );

    fn input_channels(&self) -> usize;
    fn output_channels(&self) -> usize;
}

impl<T, S> AudioProcessor<S> for Box<T>
where
    T: AudioProcessor<S> + ?Sized,
    S: dasp::Sample,
{
    fn process_unchecked(
        &mut self,
        input: &InterleavedBuffer<S>,
        output: &mut InterleavedBuffer<S>,
    ) {
        (**self).process_unchecked(input, output);
    }

    fn input_channels(&self) -> usize {
        (**self).input_channels()
    }

    fn output_channels(&self) -> usize {
        (**self).output_channels()
    }
}

pub struct AudioNode<T>
where
    T: audio_buffer::dasp::Sample,
{
    processor: Box<dyn AudioProcessor<T>>,
}

impl<T> AudioNode<T>
where
    T: audio_buffer::dasp::Sample,
{
    pub fn new(processor: Box<dyn AudioProcessor<T>>) -> Self {
        Self { processor }
    }

    pub fn get_processor(&self) -> &Box<(dyn AudioProcessor<T> + 'static)> {
        &self.processor
    }

    pub fn get_processor_mut(&mut self) -> &mut Box<(dyn AudioProcessor<T> + 'static)> {
        &mut self.processor
    }
}

pub struct PassThrough {
    input_channels: usize,
    output_channels: usize,
}

impl PassThrough {
    pub fn new(input_channels: usize, output_channels: usize) -> Self {
        Self {
            input_channels,
            output_channels,
        }
    }
}

impl<T> AudioProcessor<T> for PassThrough
where
    T: audio_buffer::dasp::Sample + 'static,
{
    fn process_unchecked(
        &mut self,
        input: &InterleavedBuffer<T>,
        output: &mut InterleavedBuffer<T>,
    ) {
        mix_buffers(input, output).expect("this is the unchecked method");
    }

    fn input_channels(&self) -> usize {
        self.input_channels
    }

    fn output_channels(&self) -> usize {
        self.output_channels
    }
}
