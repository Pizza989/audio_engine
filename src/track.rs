use std::collections::HashMap;

use audio_graph::{
    AudioGraph,
    daggy::NodeIndex,
    processor::{AudioProcessor, PassThrough},
};
use time::{FrameTime, SampleRate};

use crate::playlist::Playlist;

pub struct Track<T>
where
    T: audio_buffer::dasp::Sample,
{
    graph: AudioGraph<T, Box<dyn AudioProcessor<T>>>,
    playlist: Playlist<T>,
    // INVARIANT: `input` must never dangle
    input: NodeIndex,
}

impl<T> Track<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
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

    pub fn get_playlist(&self) -> &Playlist<T> {
        &self.playlist
    }

    pub fn get_playlist_mut(&mut self) -> &mut Playlist<T> {
        &mut self.playlist
    }
}

impl<T> AudioProcessor<T> for Track<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    fn process_unchecked(
        &mut self,
        input: &audio_buffer::buffers::interleaved::InterleavedBuffer<T>,
        output: &mut audio_buffer::buffers::interleaved::InterleavedBuffer<T>,
    ) {
        let mut inputs = HashMap::new();
        inputs.insert(self.input, input);
        self.graph.process_block(&inputs, output);
    }

    fn config(&self) -> audio_graph::processor::ProcessorConfiguration {
        self.graph
            .get_node_config(self.input)
            .expect("invariant: input must always be valid")
    }
}
