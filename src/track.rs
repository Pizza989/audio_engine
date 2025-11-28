use std::{collections::HashMap, num::NonZero};

use audio_buffer::{buffers::interleaved::InterleavedBuffer, core::io::mix_buffers};
use audio_graph::{
    AudioGraph,
    daggy::NodeIndex,
    processor::{AudioProcessor, PassThrough, ProcessingContext},
};
use time::{FrameTime, SampleRate};

use crate::playlist::Playlist;

pub struct Track<T>
where
    T: audio_buffer::dasp::Sample,
{
    graph: AudioGraph<T, Box<dyn AudioProcessor<T>>>,
    playlist: Playlist<T>,

    // TODO: upholding this will be difficult once reconfiguration
    // is implemented
    // INVARIANT: 'Buffer Validity'
    // This invariant guarantees that the buffer's size is always
    // sufficient for the block_size
    buffer: InterleavedBuffer<T>,

    // INVARIANT: 'Input Validity'
    // This invariant guarantees that `self.input` always references
    // a valid `Node` inside `self.graph`.
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

        let buffer = InterleavedBuffer::with_shape(NonZero::new(2).unwrap(), block_size);

        Self {
            graph,
            input,
            playlist: Playlist::empty(),
            buffer,
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
    T: audio_buffer::SharedSample,
{
    fn process_unchecked(
        &mut self,
        _input: Option<&audio_buffer::buffers::interleaved::InterleavedBuffer<T>>,
        output: &mut audio_buffer::buffers::interleaved::InterleavedBuffer<T>,
        processing_context: ProcessingContext,
    ) {
        let block_events = self.get_playlist().get_block_events(
            processing_context.block_range.clone(),
            processing_context.bpm,
            processing_context.sample_rate,
        );

        for block_event in block_events {
            mix_buffers(
                &block_event.event.buffer,
                &mut self.buffer,
                Some(block_event.block_offset.0 as usize),
            )
            .expect("precondition a");
        }

        let mut inputs = HashMap::new();
        inputs.insert(self.input, &self.buffer);
        self.graph
            .process_block(&inputs, output, processing_context);
    }

    fn config(&self) -> audio_graph::processor::ProcessorConfiguration {
        self.graph
            .get_node_config(self.input)
            .expect("invariant: 'Input Validity'")
    }
}
