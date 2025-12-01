use std::ops::Range;

use audio_buffer::AudioBuffer;
use time::{MusicalTime, SampleRate};

#[derive(Debug, Clone)]
pub struct ProcessingContext {
    pub sample_rate: SampleRate,
    pub block_range: Range<MusicalTime>,
    pub bpm: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfiguration {
    pub num_input_channels: usize,
    pub num_output_channels: usize,
}

pub trait AudioGenerator<Sample>
where
    Sample: audio_buffer::dasp::Sample,
{
    fn generate(&mut self, output: &mut AudioBuffer<Sample>, context: ProcessingContext);

    fn num_channels(&self) -> usize;
}

pub trait AudioTransformer<Sample>: Send
where
    Sample: audio_buffer::dasp::Sample,
{
    fn transform_in_place(&mut self, buffer: &mut AudioBuffer<Sample>, context: ProcessingContext);

    fn config(&self) -> ProcessorConfiguration;
}
