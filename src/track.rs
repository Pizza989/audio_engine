use audio_buffer::{CopyOptions, SharedSample};
use audio_graph::mix_graph::{
    processor::{AudioGenerator, AudioTransformer},
    transformer_chain::TransformerChain,
};

use crate::playlist::Playlist;

// TODO:
// - you could implement AudioProcessor for the track for handling
// a direct input stream from a cpal device, like a microphone
// - implement insert chains with dynamic channel numbers per transformer
pub struct Track<Sample>
where
    Sample: SharedSample,
{
    num_channels: usize,
    inserts: Vec<Box<dyn AudioTransformer<Sample>>>,
    playlist: Playlist<Sample>,
}

impl<T> Track<T>
where
    T: SharedSample,
{
    pub fn new(num_channels: usize, num_frames: usize) -> Self {
        Self {
            inserts: TransformerChain::new(num_channels, num_frames),
            playlist: Playlist::empty(),
            num_channels,
        }
    }

    pub fn get_playlist(&self) -> &Playlist<T> {
        &self.playlist
    }

    pub fn get_playlist_mut(&mut self) -> &mut Playlist<T> {
        &mut self.playlist
    }
}

impl<Sample> AudioGenerator<Sample> for Track<Sample>
where
    Sample: SharedSample,
{
    fn generate(
        &mut self,
        output: &mut audio_buffer::AudioBuffer<Sample>,
        context: audio_graph::mix_graph::processor::ProcessingContext,
    ) {
        let block_events = self.get_playlist().get_block_events(
            context.block_range.clone(),
            context.bpm,
            context.sample_rate,
        );

        for block_event in block_events {
            output
                .mix_from(
                    &block_event.event.buffer,
                    Sample::IDENTITY,
                    CopyOptions::default().with_src_frame_offset(block_event.block_offset),
                )
                .expect("the ChannelStrategy is not Strict");
        }

        for transformer in self.inserts.iter_mut() {
            transformer.transform_in_place(output, context);
        }
    }

    fn num_channels(&self) -> usize {
        self.num_channels
    }
}
