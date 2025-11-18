use std::{collections::HashMap, num::NonZero, ops::Range};

use audio_buffer::{
    SharedSample,
    buffers::interleaved::InterleavedBuffer,
    core::{Buffer, BufferMut, io::mix_buffers},
};
use audio_graph::{
    AudioGraph,
    daggy::{EdgeIndex, NodeIndex},
    pin_matrix::PinMatrix,
};
use log::error;
use ringbuf::{HeapCons, traits::Consumer};
use time::{FrameTime, MusicalTime, SampleRate};

use crate::{
    message::{AudioBackendCommand, AudioBackendMessage, Intent},
    track::Track,
};

pub struct AudioBackend<T: SharedSample> {
    pub(crate) command_consumer: HeapCons<AudioBackendMessage>,
    // pub(crate) status_producer: HeapProd<AudioEngineMessage>,
    pub(crate) graph: AudioGraph<T, Track<T>>,
    pub(crate) master: NodeIndex,
    pub(crate) master_buffer: InterleavedBuffer<T>,
    pub(crate) track_buffers: HashMap<NodeIndex, InterleavedBuffer<T>>,

    pub(crate) block_size: FrameTime,
    pub(crate) block_duration_musical: MusicalTime,
    pub(crate) block_range: Range<MusicalTime>,
    pub(crate) bpm: f64,
    pub(crate) sample_rate: SampleRate,

    pub(crate) running: bool,
}

impl<T: SharedSample> AudioBackend<T> {
    pub fn add_track(&mut self) {
        let index = self
            .graph
            .add_node(Track::from_config(self.sample_rate, self.block_size));

        self.graph
            .add_connection(index, self.master, PinMatrix::diagonal(2, 2))
            .expect("logic error");

        self.track_buffers.insert(
            index,
            InterleavedBuffer::with_shape(NonZero::new(2).unwrap(), self.block_size),
        );
    }
    pub fn add_connection(&mut self, source: NodeIndex, destination: NodeIndex, matrix: PinMatrix) {
        if let Err(e) = self.graph.add_connection(source, destination, matrix) {
            error!(
                "Error while adding a connection to the audio graph: {:?}",
                e
            );
        }
    }

    pub fn update_connection(&mut self, edge: EdgeIndex, matrix: PinMatrix) {
        if let None = self.graph.update_connection(edge, matrix) {
            error!("Error while updating connection");
        }
    }
}

impl<T: SharedSample> AudioBackend<T> {
    pub fn new(
        command_consumer: HeapCons<AudioBackendMessage>,
        // status_producer: HeapProd<AudioEngineMessage>,
        graph: AudioGraph<T, Track<T>>,
        master: NodeIndex,
        block_size: FrameTime,
        bpm: f64,
        sample_rate: SampleRate,
    ) -> Self {
        Self {
            command_consumer,
            // status_producer,
            graph,
            master,
            master_buffer: InterleavedBuffer::with_shape(NonZero::new(2).unwrap(), block_size),
            track_buffers: HashMap::new(),
            block_size,
            block_duration_musical: block_size.to_musical_lossy(bpm, sample_rate),
            block_range: MusicalTime::ZERO..block_size.to_musical_lossy(bpm, sample_rate),
            bpm,
            sample_rate,
            running: false,
        }
    }

    pub fn process_messages(&mut self) {
        while let Some(message) = self.command_consumer.try_pop() {
            match message.intent {
                Intent::Query(query) => (),
                Intent::Command(command) => match command {
                    AudioBackendCommand::Start => self.running = true,
                    AudioBackendCommand::Pause => self.running = false,
                    AudioBackendCommand::SetPlayhead(musical_time) => {
                        self.block_range = musical_time..musical_time + self.block_duration_musical
                    }
                    AudioBackendCommand::AddTrack => self.add_track(),
                    AudioBackendCommand::AddConnection {
                        source,
                        destination,
                        matrix,
                    } => self.add_connection(source, destination, matrix),
                    AudioBackendCommand::UpdateConnection { edge, matrix } => {
                        self.update_connection(edge, matrix)
                    }
                },
            }
        }
    }

    // PRECONDITIONS:
    // a) track_buffers must hold a valid buffer for each track that isn't self.master
    // b) master_buffer must be a valid buffer
    pub fn process_block(&mut self, output: &mut [T]) {
        if !self.running {
            return;
        }

        for track_index in self.graph.get_dag().graph().node_indices() {
            if track_index == self.master {
                continue;
            }

            let track = self.graph.get_node(track_index).expect("logic error");

            let block_events = track.get_playlist().get_block_events(
                self.block_range.clone(),
                self.bpm,
                self.sample_rate,
            );

            let track_buffer = self
                .track_buffers
                .get_mut(&track_index)
                .expect("precondition a");

            for block_event in block_events {
                mix_buffers(
                    &block_event.event.buffer,
                    track_buffer,
                    Some(block_event.block_offset.0 as usize),
                )
                .expect("precondition a");
            }
        }

        self.graph.process_block(
            &self.track_buffers.iter().map(|(&k, v)| (k, v)).collect(),
            &mut self.master_buffer,
        );

        // TODO: make clean adapter abstraction
        let channels = self.master_buffer.channels();
        for (i, sample) in output.iter_mut().enumerate() {
            let frame = i / channels;
            let channel = i % channels;
            *sample = *self
                .master_buffer
                .get_sample(channel, frame)
                .unwrap_or(&T::EQUILIBRIUM);
        }

        for buffer in self.track_buffers.values_mut() {
            buffer.set_to_equilibrium();
        }

        self.master_buffer.set_to_equilibrium();

        self.block_range =
            self.block_range.end..(self.block_range.end + self.block_duration_musical)
    }
}
