use audio_graph::{AudioGraph, daggy::NodeIndex, pin_matrix::PinMatrix};
use time::{FrameTime, SampleRate};

use crate::{
    playlist::{BlockEvent, BlockIterator},
    track::Track,
};

pub struct AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    graph: AudioGraph<T, Track<T>>,
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
        let master_graph = Track::from_config(sample_rate, block_size);
        let (graph, master_idx) = AudioGraph::new(master_graph, sample_rate, block_size);

        Self {
            graph: graph,
            master: master_idx,
            block_size,
            sample_rate,
            bpm,
        }
    }

    pub fn add_track(&mut self) -> NodeIndex {
        let index = self
            .graph
            .add_node(Track::from_config(self.sample_rate, self.block_size));

        self.add_connection(index, self.master, PinMatrix::diagonal(2, 2))
            .expect("must be valid due to invariants");
        index
    }

    pub fn get_track(&mut self) {}

    pub fn add_connection(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
        pin_matrix: PinMatrix,
    ) -> Result<audio_graph::daggy::EdgeIndex, audio_graph::error::GraphError> {
        self.graph.add_connection(src, dst, pin_matrix)
    }

    fn iter_all_tracks(&self) -> AllTracksIterator<'_> {
        AllTracksIterator {
            track_iterators: self
                .graph
                .get_dag()
                .graph()
                .node_indices()
                .map(|track_index| {
                    (
                        track_index,
                        self.graph
                            .get_dag()
                            .node_weight(track_index)
                            .expect("must be valid")
                            .playlist()
                            .iter_blocks(self.block_size, self.sample_rate, self.bpm),
                    )
                })
                .collect(),
        }
    }

    pub fn run(&self) {
        for all_block_events in self.iter_all_tracks() {
            for (_track_index, _block_events) in all_block_events {}
        }
    }
}

struct AllTracksIterator<'a> {
    track_iterators: Vec<(NodeIndex, BlockIterator<'a>)>,
}

impl Iterator for AllTracksIterator<'_> {
    type Item = Vec<(NodeIndex, Vec<BlockEvent>)>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.track_iterators
                .iter_mut()
                .map(|(track_id, iter)| (*track_id, iter.next().unwrap()))
                .collect(),
        )
    }
}
