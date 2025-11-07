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
        let config = self.config();
        if config.num_input_channels != input.channels()
            || config.num_output_channels != output.channels()
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

    fn config(&self) -> ProcessorConfiguration;
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

    fn config(&self) -> ProcessorConfiguration {
        (**self).config()
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
    num_input_channels: usize,
    num_output_channels: usize,
}

impl PassThrough {
    pub fn new(input_channels: usize, output_channels: usize) -> Self {
        Self {
            num_input_channels: input_channels,
            num_output_channels: output_channels,
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
        mix_buffers(input, output, None).expect("this is the unchecked method");
    }

    fn config(&self) -> ProcessorConfiguration {
        ProcessorConfiguration {
            num_input_channels: self.num_input_channels,
            num_output_channels: self.num_output_channels,
        }
    }
}

pub struct ProcessorConfiguration {
    // If these will ever be reconfigurable they will
    // probably have to be stored here
    // pub sample_rate: SampleRate,
    // pub block_size: usize,
    pub num_input_channels: usize,
    pub num_output_channels: usize,
}
