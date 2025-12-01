use audio_buffer::{AudioBuffer, CopyOptions, SharedSample};
use audio_graph::mix_graph::{
    processor::{AudioTransformer, ProcessorConfiguration},
    transformer_chain::TransformerChain,
};

// TODO:
// you could implement AudioProcessor for the track for handling
// a direct input stream from a cpal device, like a microphone
pub struct Bus<Sample>
where
    Sample: SharedSample,
{
    num_input_channels: usize,
    num_output_channels: usize,
    inserts: Vec<Box<dyn AudioTransformer<Sample>>>,
}

impl<T> Bus<T>
where
    T: SharedSample,
{
    pub fn new(num_input_channels: usize, num_output_channels: usize, num_frames: usize) -> Self {
        Self {
            inserts: Vec::new(),
            num_input_channels,
            num_output_channels,
        }
    }
}

impl<Sample> AudioTransformer<Sample> for Bus<Sample>
where
    Sample: SharedSample,
{
    fn transform_in_place(
        &mut self,
        buffer: &mut AudioBuffer<Sample>,
        context: audio_graph::mix_graph::processor::ProcessingContext,
    ) {
        for transformer in self.inserts.iter_mut() {
            transformer.transform_in_place(buffer, context);
        }
    }

    fn config(&self) -> audio_graph::mix_graph::processor::ProcessorConfiguration {
        ProcessorConfiguration {
            num_input_channels: self.num_input_channels,
            num_output_channels: self.num_output_channels,
        }
    }
}
