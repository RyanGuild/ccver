use petgraph::{
    graph::NodeIndex,
    visit::{DfsPostOrder, Walker as _},
};

use crate::graph::{CommitGraphNodeWeight, CommitGraphT};

pub trait DfsPostorderHistoryExt<'a> {
    fn dfs_postorder_history(
        &self,
        headidx: NodeIndex,
    ) -> Vec<(NodeIndex, CommitGraphNodeWeight<'a>)>;
}

impl<'a> DfsPostorderHistoryExt<'a> for CommitGraphT<'a> {
    fn dfs_postorder_history(
        &self,
        headidx: NodeIndex,
    ) -> Vec<(NodeIndex, CommitGraphNodeWeight<'a>)> {
        DfsPostOrder::new(self, headidx)
            .iter(self)
            .map(move |idx| (idx, self[idx].clone()))
            .collect::<Vec<_>>()
    }
}
