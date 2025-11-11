use std::{collections::HashMap, num::NonZero, ops::Range, path::Path, sync::Arc};

use audio_buffer::{
    buffers::interleaved::InterleavedBuffer,
    core::{BufferMut, io::mix_buffers},
    loader::error::LoadError,
};
use audio_graph::{AudioGraph, daggy::NodeIndex, pin_matrix::PinMatrix};
use symphonia::core::conv::ConvertibleSample;
use time::{FrameTime, MusicalTime, SampleRate};

use crate::track::Track;

pub struct AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    graph: AudioGraph<T, Track<T>>,
    track_buffers: HashMap<NodeIndex, Option<InterleavedBuffer<T>>>,
    master_buffer: Option<InterleavedBuffer<T>>,
    // master must always be valid
    master: NodeIndex,
    block_size: FrameTime,
    sample_rate: SampleRate,
    bpm: f64,
}

impl<T> AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    pub fn new(bpm: f64, sample_rate: SampleRate, block_size: FrameTime) -> Self {
        let master_track = Track::from_config(sample_rate, block_size);
        let (graph, master_idx) = AudioGraph::new(master_track, sample_rate, block_size);

        Self {
            graph: graph,
            master: master_idx,
            track_buffers: HashMap::new(),
            block_size,
            sample_rate,
            bpm,
            master_buffer: Some(InterleavedBuffer::with_shape(
                NonZero::new(2).unwrap(),
                sample_rate,
                block_size,
            )),
        }
    }

    pub fn add_track(&mut self) -> NodeIndex {
        let index = self
            .graph
            .add_node(Track::from_config(self.sample_rate, self.block_size));

        self.add_connection(index, self.master, PinMatrix::diagonal(2, 2))
            .expect("must be valid due to invariants");

        self.track_buffers.insert(
            index,
            Some(InterleavedBuffer::with_shape(
                NonZero::new(2).unwrap(),
                self.sample_rate,
                self.block_size,
            )),
        );
        index
    }

    pub fn get_track(&self, index: NodeIndex) -> Option<&Track<T>> {
        self.graph.get_node(index)
    }

    pub fn get_track_mut(&mut self, index: NodeIndex) -> Option<&mut Track<T>> {
        self.graph.get_node_mut(index)
    }

    pub fn load_audio_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Arc<InterleavedBuffer<T>>, LoadError>
    where
        T: ConvertibleSample,
    {
        audio_buffer::loader::load(path).map(|buffer| Arc::new(buffer))
    }

    pub fn add_connection(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
        pin_matrix: PinMatrix,
    ) -> Result<audio_graph::daggy::EdgeIndex, audio_graph::error::GraphError> {
        self.graph.add_connection(src, dst, pin_matrix)
    }

    /// Process one block of the timeline into the output_buffer
    ///
    /// # PRECONDITIONS
    /// - track_buffers must contain one buffer per track which:
    ///   - has as many channels as the track has input channels
    ///   - has a size equal to self.block_size
    ///   - is empty (otherwise its content will be mixed in)
    ///
    ///  - output_buffer must have as many channels as the master
    ///    track has output channels and a size equal to
    ///    self.block_size
    pub fn process_block(
        &mut self,
        block_range: Range<MusicalTime>,
        track_buffers: &mut HashMap<NodeIndex, InterleavedBuffer<T>>,
        master_buffer: &mut InterleavedBuffer<T>,
    ) {
        for track_index in self.graph.get_dag().graph().node_indices() {
            // TODO: self.master is a bus but busses don't exist yet
            if track_index == self.master {
                continue;
            }

            let track = self.graph.get_node(track_index).expect("is valid");

            let block_events = track.get_playlist().get_block_events(
                block_range.clone(),
                self.bpm,
                self.sample_rate,
            );

            let track_buffer = track_buffers.get_mut(&track_index).expect("preconditions");

            for block_event in block_events {
                mix_buffers(
                    &block_event.event.buffer,
                    track_buffer,
                    Some(block_event.block_offset.0 as usize),
                )
                .expect("preconditions");
            }
        }

        self.graph.process_block(
            &track_buffers.iter().map(|(&k, v)| (k, v)).collect(),
            master_buffer,
        );
    }

    // PRECONDITIONS:
    // - track_buffers must contain one buffer per track which:
    //   - has as much channels as the track has input channels
    //   - has a size equal to self.block_size
    //   - is empty
    //
    //  - master_buffer must be some and contain a valid buffer
    //   like track_buffers
    pub fn run(&mut self) {
        let mut track_buffers: HashMap<NodeIndex, InterleavedBuffer<T>> = HashMap::new();
        self.track_buffers
            .iter_mut()
            .map(|(index, option)| (index, option.take().expect("preconditions")))
            .for_each(|(index, buffer)| {
                track_buffers.insert(*index, buffer);
            });

        let mut master_buffer = self.master_buffer.take().expect("preconditions");

        let block_duration_musical = self.block_size.to_musical_lossy(self.bpm, self.sample_rate);
        let mut block_range = MusicalTime::ZERO..block_duration_musical;

        loop {
            // TODO: send master_buffer to some audio backend
            self.process_block(block_range.clone(), &mut track_buffers, &mut master_buffer);

            for buffer in track_buffers.values_mut() {
                buffer.set_to_equilibrium();
            }

            master_buffer.set_to_equilibrium();
            block_range = block_range.end..block_range.end + block_duration_musical
        }
    }
}
