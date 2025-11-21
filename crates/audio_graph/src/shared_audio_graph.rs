use daggy::{Dag, NodeIndex};
use left_right::{Absorb, ReadHandle, WriteHandle};
use time::FrameTime;

use crate::{pin_matrix::PinMatrix, processor::ProcessorKey};

pub enum Operation {
    AddConnection {
        source: NodeIndex,
        destination: NodeIndex,
        matrix: PinMatrix,
    },
}

impl Absorb<Operation> for SharedAudioGraph {
    fn absorb_first(&mut self, operation: &mut Operation, other: &Self) {
        match operation {
            Operation::AddConnection {
                source,
                destination,
                matrix,
            } => self.add_connection(*source, *destination, matrix.clone()),
        }
    }

    fn sync_with(&mut self, first: &Self) {
        todo!()
    }
}

pub struct Connection {
    matrix: PinMatrix,
}

pub struct SharedAudioGraph {
    dag: Dag<ProcessorKey, Connection>,
    processing_order: Vec<NodeIndex>,
    block_size: FrameTime,
    output: NodeIndex,
}

impl SharedAudioGraph {
    pub fn new(
        node: ProcessorKey,
        processing_order: Vec<NodeIndex>,
        block_size: FrameTime,
        output: NodeIndex,
    ) -> Self {
        let mut graph = Self {
            dag: Dag::new(),
            processing_order,
            block_size,
            output,
        };

        graph
    }

    pub fn add_connection(&mut self, source: NodeIndex, destination: NodeIndex, matrix: PinMatrix) {
        self.dag
            .add_edge(source, destination, Connection { matrix })
            .unwrap();
    }
}

// pub struct Engine {
//     graph: WriteHandle<SharedAudioGraph, Operation>,
// }

// impl Engine {
//     pub fn test(&mut self) {
//         self.graph.append();
//         self.graph.publish();
//     }
// }

// pub struct Backend {
//     graph: ReadHandle<SharedAudioGraph>,
// }

// impl Backend {
//     pub fn test(&self) {
//         let shared_graph = self.graph.enter().unwrap();
//     }
// }
