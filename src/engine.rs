use audio_graph::{AudioGraph, daggy::NodeIndex, pin_matrix::PinMatrix};
use interavl::IntervalTree;
use time::SampleRate;

use crate::{timeline::Timeline, track::Track};

pub struct AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    graph: AudioGraph<T, Track<T>>,
    _timeline: Timeline,
    // master must always be valid
    master: NodeIndex,
    sample_rate: SampleRate,
    block_size: usize,
}

impl<T> AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    pub fn new(bpm: f64, sample_rate: SampleRate, block_size: usize) -> Self {
        let master_graph = Track::from_config(sample_rate, block_size);
        let (graph, master_idx) = AudioGraph::new(master_graph, sample_rate, block_size);

        Self {
            graph: graph,
            master: master_idx,
            sample_rate,
            block_size,
            _timeline: Timeline::new(bpm, sample_rate, IntervalTree::default()),
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

    pub fn run(&self) {}
}
