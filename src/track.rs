use std::collections::HashMap;

use audio_graph::{
    AudioGraph,
    daggy::NodeIndex,
    processor::{AudioProcessor, PassThrough},
};
use time::{FrameTime, SampleRate};

use crate::playlist::Playlist;

pub struct Track<T: audio_buffer::dasp::Sample> {
    graph: AudioGraph<T, Box<dyn AudioProcessor<T>>>,
    playlist: Playlist,
    // INVARIANT: `input` must never dangle
    input: NodeIndex,
}

impl<T: audio_buffer::dasp::Sample + 'static> Track<T> {
    /// Convinience constructor to create a stereo track from its configuration
    pub fn from_config(sample_rate: SampleRate, block_size: FrameTime) -> Self {
        let (graph, input) = AudioGraph::<T, Box<dyn AudioProcessor<T>>>::new(
            Box::new(PassThrough::new(2, 2)),
            sample_rate,
            block_size,
        );

        Self {
            graph,
            input,
            playlist: Playlist::empty(),
        }
    }

    pub fn from_graph(graph: AudioGraph<T, Box<dyn AudioProcessor<T>>>, input: NodeIndex) -> Self {
        Self {
            graph,
            input,
            playlist: Playlist::empty(),
        }
    }

    pub fn playlist(&self) -> &Playlist {
        &self.playlist
    }
}

impl<T: audio_buffer::dasp::Sample + 'static> AudioProcessor<T> for Track<T> {
    fn process_unchecked(
        &mut self,
        input: &audio_buffer::buffers::interleaved::InterleavedBuffer<T>,
        output: &mut audio_buffer::buffers::interleaved::InterleavedBuffer<T>,
    ) {
        let mut inputs = HashMap::new();
        inputs.insert(self.input, input);
        self.graph.process(inputs, output);
    }

    fn config(&self) -> audio_graph::processor::ProcessorConfiguration {
        self.graph
            .get_node_config(self.input)
            .expect("invariant: input must always be valid")
    }
}
