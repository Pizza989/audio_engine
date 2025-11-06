use audio_graph::{
    AudioGraph,
    processor::{AudioProcessor, PassThrough},
};
use time::SampleRate;

pub struct Track<T: audio_buffer::dasp::Sample> {
    graph: AudioGraph<T, Box<dyn AudioProcessor<T>>>,
}

impl<T: audio_buffer::dasp::Sample + 'static> Track<T> {
    /// Convinience method to create a stereo track from its configuration
    pub fn from_config(sample_rate: SampleRate, block_size: usize) -> Self {
        Self {
            graph: AudioGraph::new(Box::new(PassThrough::new(2, 2)), sample_rate, block_size),
        }
    }

    pub fn from_graph(graph: AudioGraph<T, Box<dyn AudioProcessor<T>>>) -> Self {
        Self { graph }
    }
}

impl<T: audio_buffer::dasp::Sample + 'static> AudioProcessor<T> for Track<T> {
    fn process_unchecked(
        &mut self,
        input: &audio_buffer::buffers::interleaved::InterleavedBuffer<T>,
        output: &mut audio_buffer::buffers::interleaved::InterleavedBuffer<T>,
    ) {
        self.graph.process_unchecked(input, output);
    }

    fn input_channels(&self) -> usize {
        self.graph.input_channels()
    }

    fn output_channels(&self) -> usize {
        self.graph.output_channels()
    }
}
