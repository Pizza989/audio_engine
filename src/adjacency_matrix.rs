use std::collections::{HashMap, HashSet};

use audio_graph::daggy::{EdgeIndex, NodeIndex};

pub struct AdjacencyMatrix {
    nodes: HashSet<NodeIndex>,
    edges: HashMap<EdgeIndex, (NodeIndex, NodeIndex)>,
}

impl AdjacencyMatrix {
    pub fn empty() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashMap::new(),
        }
    }

    pub fn is_adjacent(&self, src: NodeIndex, dst: NodeIndex) -> bool {
        for (edge_src, edge_dst) in self.edges.values() {
            if *edge_src == src && *edge_dst == dst {
                return true;
            }
        }
        false
    }

    pub fn edges(&self) -> &HashMap<EdgeIndex, (NodeIndex, NodeIndex)> {
        &self.edges
    }

    pub fn nodes(&self) -> &HashSet<NodeIndex> {
        &self.nodes
    }
}

impl AdjacencyMatrix {
    /// This can insert edges where the endpoints are dangling
    pub fn add_edge_unchecked(&mut self, index: EdgeIndex, src: NodeIndex, dst: NodeIndex) {
        self.edges.insert(index, (src, dst));
    }

    pub fn add_node(&mut self, index: NodeIndex) {
        self.nodes.insert(index);
    }

    /// This can remove nodes that are in edges, therefore leaving edge endpoints dangling
    pub fn remove_node_unchecked(&mut self, index: NodeIndex) {
        self.nodes.remove(&index);
    }

    pub fn remove_edge(&mut self, index: EdgeIndex) {
        self.edges.remove(&index);
    }
}
