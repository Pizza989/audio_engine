use std::ops::Range;

use audio_buffer::SharedSample;
use audio_graph::mix_graph::MixGraph;
use crossbeam_channel::Receiver;
use time::{FrameTime, MusicalTime, SampleRate};

use crate::{
    command::{AudioCommand, Response},
    track::Track,
};

pub struct AudioBackend<Sample: SharedSample> {
    receiver: Receiver<AudioCommand>,
    graph: MixGraph<Sample, Track<Sample>, ()>,

    block_size: FrameTime,
    block_duration_musical: MusicalTime,
    block_range: Range<MusicalTime>,
    bpm: f64,
    sample_rate: SampleRate,

    running: bool,
}

impl<Sample: SharedSample> AudioBackend<Sample> {
    pub fn new(
        receiver: Receiver<AudioCommand>,
        graph: MixGraph<Sample, Track<Sample>, ()>,
        block_size: FrameTime,
        bpm: f64,
        sample_rate: SampleRate,
    ) -> Self {
        Self {
            graph,

            block_size,
            block_duration_musical: block_size.to_musical_lossy(bpm, sample_rate),
            block_range: MusicalTime::ZERO..block_size.to_musical_lossy(bpm, sample_rate),
            bpm,
            sample_rate,
            running: false,
            receiver,
        }
    }

    pub fn process_commands(&mut self) {
        while let Ok(command) = self.receiver.try_recv() {
            command
                .response_sender
                .try_send(Response::Ok)
                .expect("logic error: could not send response with response_sender");
        }
    }

    pub fn process_block(&mut self, output: &mut [Sample]) {
        if !self.running {
            return;
        }
        self.block_range =
            self.block_range.end..(self.block_range.end + self.block_duration_musical)
    }
}
