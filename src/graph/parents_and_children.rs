use petgraph::graph::NodeIndex;

use crate::graph::CommitGraphT;

pub trait ParentChildExt {
    fn parents(&self, idx: NodeIndex) -> Vec<NodeIndex>;
    fn children(&self, idx: NodeIndex) -> Vec<NodeIndex>;
}

impl<'a> ParentChildExt for CommitGraphT<'a> {
    fn parents(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.neighbors_directed(idx, petgraph::Direction::Outgoing)
            .collect()
    }

    fn children(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.neighbors_directed(idx, petgraph::Direction::Incoming)
            .collect()
    }
}
