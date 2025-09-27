use eyre::OptionExt as _;
use eyre::Result;
use petgraph::graph::NodeIndex;

use crate::graph::{CommitGraphT, parents_and_children::ParentChildExt as _};

pub fn find_tail(graph: &CommitGraphT) -> Result<NodeIndex> {
    graph
        .node_indices()
        .find(|idx| graph.parents(*idx).is_empty())
        .ok_or_eyre("No tail found in graph")
}
