use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use petgraph::{
    Directed, Direction, EdgeType, Graph,
    csr::IndexType,
    data::{Build, DataMap, DataMapMut},
    graph::{DiGraph, EdgeIndex, NodeIndex},
    visit::{
        EdgeIndexable, EdgeRef as _, IntoEdgeReferences, IntoNeighbors, IntoNeighborsDirected,
        IntoNodeIdentifiers, IntoNodeReferences, NodeCount, NodeIndexable,
    },
};
use tracing::debug;
pub mod assign_versions;
pub mod branch;
pub mod branch_heads;
pub mod commit;
pub mod head;
pub mod node;
pub mod parents_and_children;
pub mod tag;
pub mod tail;
pub mod version;
use crate::{
    graph::{
        assign_versions::WithCCVerVersions,
        commit::{CommitExt, CommitMemo},
        head::{HasHead, HeadMemo},
        node::{CommitGraphNodeData, CommitGraphNodeWeight},
        parents_and_children::{HasParentsAndChildren, WithParentsAndChildEdges},
        tail::{HasTail, TailMemo},
    },
    logs::Logs,
    version_format::VersionFormat,
};

pub type CommitGraphT<'a> = DiGraph<CommitGraphNodeWeight<'a>, ()>;

pub trait CommitGraph<N, E, Ty, Ix>:
    GraphOps<N, E, Ty, Ix>
    + HasHead<N, E, Ty, Ix>
    + HasTail<N, E, Ty, Ix>
    + HasParentsAndChildren<N, E, Ty, Ix>
    + CommitExt<N, E, Ty, Ix>
{
}

impl<N, E, Ty, Ix, T> CommitGraph<N, E, Ty, Ix> for T where
    T: GraphOps<N, E, Ty, Ix>
        + HasHead<N, E, Ty, Ix>
        + HasTail<N, E, Ty, Ix>
        + HasParentsAndChildren<N, E, Ty, Ix>
        + CommitExt<N, E, Ty, Ix>
{
}

pub struct MemoizedCommitGraph<'a, N = CommitGraphNodeWeight<'a>, E = (), Ty = Directed, Ix = u32> {
    inner: Box<dyn CommitGraph<N, E, Ty, Ix> + 'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, N, E, Ty, Ix> Deref for MemoizedCommitGraph<'a, N, E, Ty, Ix> {
    type Target = Box<dyn CommitGraph<N, E, Ty, Ix> + 'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, N, E, Ty, Ix> DerefMut for MemoizedCommitGraph<'a, N, E, Ty, Ix> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, N, E, Ty, Ix> HasHead<N, E, Ty, Ix> for MemoizedCommitGraph<'a, N, E, Ty, Ix> {
    fn headidx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.headidx()
    }
    fn head(&self) -> Option<&N> {
        self.inner.head()
    }
}

impl<'a, N, E, Ty, Ix> HasTail<N, E, Ty, Ix> for MemoizedCommitGraph<'a, N, E, Ty, Ix> {
    fn tailidx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.tailidx()
    }
    fn tail(&self) -> Option<&N> {
        self.inner.tail()
    }
}

impl<'a, N, E, Ty, Ix> HasParentsAndChildren<N, E, Ty, Ix>
    for MemoizedCommitGraph<'a, N, E, Ty, Ix>
{
    fn parents(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        self.inner.parents(idx)
    }
    fn parentidxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner.parentidxs(idx)
    }
    fn children(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        self.inner.children(idx)
    }
    fn childidxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner.childidxs(idx)
    }
}

impl<'a, N, E, Ty, Ix> CommitExt<N, E, Ty, Ix> for MemoizedCommitGraph<'a, N, E, Ty, Ix> {
    fn commit_by_hash(&self, commit: &str) -> Option<&N> {
        self.inner.commit_by_hash(commit)
    }
    fn commitidx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>> {
        self.inner.commitidx_by_hash(commit)
    }
}

impl<'a, N, E, Ty, Ix> GraphOps<N, E, Ty, Ix> for MemoizedCommitGraph<'a, N, E, Ty, Ix> {
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        self.inner.add_node(weight)
    }
    fn node_weight(&self, idx: NodeIndex<Ix>) -> Option<&N> {
        self.inner.node_weight(idx)
    }
    fn node_weight_mut(&mut self, idx: NodeIndex<Ix>) -> Option<&mut N> {
        self.inner.node_weight_mut(idx)
    }
    fn edge_weight(&self, idx: EdgeIndex<Ix>) -> Option<&E> {
        self.inner.edge_weight(idx)
    }
    fn edge_weight_mut(&mut self, idx: EdgeIndex<Ix>) -> Option<&mut E> {
        self.inner.edge_weight_mut(idx)
    }
    fn add_edge(
        &mut self,
        from: NodeIndex<Ix>,
        to: NodeIndex<Ix>,
        weight: E,
    ) -> Option<EdgeIndex<Ix>> {
        self.inner.add_edge(from, to, weight)
    }
    fn contains_edge(&self, from: NodeIndex<Ix>, to: NodeIndex<Ix>) -> bool {
        self.inner.contains_edge(from, to)
    }
    fn node_identifiers(&self) -> Vec<NodeIndex<Ix>> {
        self.inner.node_identifiers()
    }
    fn node_references(&self) -> Vec<(NodeIndex<Ix>, &N)> {
        self.inner.node_references()
    }
    fn edge_identifiers(&self) -> Vec<EdgeIndex<Ix>> {
        self.inner.edge_identifiers()
    }
    fn edge_references(&self) -> Vec<(EdgeIndex<Ix>, &E)> {
        self.inner.edge_references()
    }
    fn node_count(&self) -> usize {
        self.inner.node_count()
    }
    fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }
    fn node_bound(&self) -> usize {
        self.inner.node_bound()
    }
    fn edge_bound(&self) -> usize {
        self.inner.edge_bound()
    }
    fn neighbors(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner.neighbors(idx)
    }
    fn neighbors_directed(&self, idx: NodeIndex<Ix>, dir: Direction) -> Vec<NodeIndex<Ix>> {
        self.inner.neighbors_directed(idx, dir)
    }
    fn base_graph(&self) -> &Graph<N, E, Ty, Ix> {
        self.inner.base_graph()
    }
    fn base_graph_mut(&mut self) -> &mut Graph<N, E, Ty, Ix> {
        self.inner.base_graph_mut()
    }
}

impl<'a> MemoizedCommitGraph<'a> {
    pub fn new(logs: Logs<'a>, version_format: &VersionFormat) -> MemoizedCommitGraph<'a> {
        let graph: Graph<Arc<Mutex<CommitGraphNodeData<'a>>>, ()> = CommitGraphT::new();
        debug!("CommitGraphT created");
        let graph = CommitMemo::new(graph);
        debug!("CommitMemo created");
        let mut graph = WithParentsAndChildEdges::new(graph);
        debug!("WithParentsAndChildEdges created");
        for log in logs {
            graph.add_node(Arc::new(Mutex::new(CommitGraphNodeData::from(log))));
        }
        debug!("Nodes added");
        let graph = TailMemo::new(graph).unwrap();
        debug!("TailMemo created");
        let graph = HeadMemo::new(graph);
        debug!("HeadMemo created");

        let graph = WithCCVerVersions::new(graph, version_format.clone());

        MemoizedCommitGraph {
            inner: Box::new(graph),
            _marker: PhantomData,
        }
    }
}

// ----------------------------------------------------------------------------
// Base-graph tracking for composable wrappers (Associated Type Pattern)
// ----------------------------------------------------------------------------

/// Tracks the concrete base graph type at the bottom of any wrapper chain.
/// This enables infinitely nestable wrappers without triggering recursive
/// trait evaluation, since all constructor bounds can be expressed in terms of
/// the concrete `BaseGraph` instead of the wrapper itself.
pub trait GraphOps<N, E, Ty, Ix> {
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix>;
    fn node_weight(&self, idx: NodeIndex<Ix>) -> Option<&N>;
    fn node_weight_mut(&mut self, idx: NodeIndex<Ix>) -> Option<&mut N>;
    fn edge_weight(&self, idx: EdgeIndex<Ix>) -> Option<&E>;
    fn edge_weight_mut(&mut self, idx: EdgeIndex<Ix>) -> Option<&mut E>;
    fn add_edge(
        &mut self,
        from: NodeIndex<Ix>,
        to: NodeIndex<Ix>,
        weight: E,
    ) -> Option<EdgeIndex<Ix>>;
    fn contains_edge(&self, from: NodeIndex<Ix>, to: NodeIndex<Ix>) -> bool;
    fn node_identifiers(&self) -> Vec<NodeIndex<Ix>>;
    fn node_references(&self) -> Vec<(NodeIndex<Ix>, &N)>;
    fn edge_identifiers(&self) -> Vec<EdgeIndex<Ix>>;
    fn edge_references(&self) -> Vec<(EdgeIndex<Ix>, &E)>;
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn node_bound(&self) -> usize;
    fn edge_bound(&self) -> usize;
    fn neighbors(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>>;
    fn neighbors_directed(&self, idx: NodeIndex<Ix>, dir: Direction) -> Vec<NodeIndex<Ix>>;
    fn base_graph(&self) -> &Graph<N, E, Ty, Ix>;
    fn base_graph_mut(&mut self) -> &mut Graph<N, E, Ty, Ix>;
}

impl<N, E, Ty, Ix> GraphOps<N, E, Ty, Ix> for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn base_graph(&self) -> &Graph<N, E, Ty, Ix> {
        self
    }

    fn base_graph_mut(&mut self) -> &mut Graph<N, E, Ty, Ix> {
        self
    }

    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        Build::add_node(self, weight)
    }

    fn node_weight(&self, idx: NodeIndex<Ix>) -> Option<&N> {
        DataMap::node_weight(self, idx)
    }

    fn node_weight_mut(&mut self, idx: NodeIndex<Ix>) -> Option<&mut N> {
        DataMapMut::node_weight_mut(self, idx)
    }

    fn edge_weight(&self, idx: EdgeIndex<Ix>) -> Option<&E> {
        DataMap::edge_weight(self, idx)
    }

    fn edge_weight_mut(&mut self, idx: EdgeIndex<Ix>) -> Option<&mut E> {
        DataMapMut::edge_weight_mut(self, idx)
    }

    fn add_edge(
        &mut self,
        from: NodeIndex<Ix>,
        to: NodeIndex<Ix>,
        weight: E,
    ) -> Option<EdgeIndex<Ix>> {
        Build::add_edge(self, from, to, weight)
    }

    fn contains_edge(&self, from: NodeIndex<Ix>, to: NodeIndex<Ix>) -> bool {
        Graph::<N, E, Ty, Ix>::contains_edge(self, from, to)
    }

    fn node_identifiers(&self) -> Vec<NodeIndex<Ix>> {
        IntoNodeIdentifiers::node_identifiers(self).collect()
    }

    fn node_references(&self) -> Vec<(NodeIndex<Ix>, &N)> {
        IntoNodeReferences::node_references(self).collect()
    }

    fn edge_identifiers(&self) -> Vec<EdgeIndex<Ix>> {
        IntoEdgeReferences::edge_references(self)
            .map(|f| f.id())
            .collect()
    }

    fn edge_references(&self) -> Vec<(EdgeIndex<Ix>, &E)> {
        IntoEdgeReferences::edge_references(self)
            .map(|f| (f.id(), f.weight()))
            .collect()
    }

    fn node_count(&self) -> usize {
        Graph::<N, E, Ty, Ix>::node_count(self)
    }

    fn edge_count(&self) -> usize {
        Graph::<N, E, Ty, Ix>::edge_count(self)
    }

    fn node_bound(&self) -> usize {
        NodeIndexable::node_bound(self)
    }

    fn edge_bound(&self) -> usize {
        EdgeIndexable::edge_bound(self)
    }

    fn neighbors(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        IntoNeighbors::neighbors(self, idx).collect()
    }

    fn neighbors_directed(&self, idx: NodeIndex<Ix>, dir: Direction) -> Vec<NodeIndex<Ix>> {
        IntoNeighborsDirected::neighbors_directed(self, idx, dir).collect()
    }
}

#[cfg(test)]
mod graph_tests {

    use std::sync::{Arc, Mutex};

    use crate::{
        graph::{
            CommitGraphT, GraphOps,
            commit::CommitMemo,
            head::{HasHead, HeadMemo},
            node::CommitGraphNodeData,
            parents_and_children::{HasParentsAndChildren, WithParentsAndChildEdges},
            tail::TailMemo,
        },
        logs::Logs,
        version_format::VersionFormat,
    };
    use eyre::*;
    use petgraph::{
        graph::{self, DiGraph},
        visit::{Bfs, Walker as _},
    };

    #[test]
    fn layered_graph_construction() -> Result<()> {
        let logs = Logs::default();
        let version_format = VersionFormat::default();
        let graph = super::MemoizedCommitGraph::new(logs, &version_format);

        let headidx = graph.headidx().unwrap();

        let parents = graph.parentidxs(headidx);

        for parent in parents {
            let parent = graph.node_weight(parent).unwrap();
            println!(
                "Parent: {:?}",
                parent.clone().lock().expect("failed to lock")
            );
        }

        Ok(())
    }

    #[test]
    fn test_graph_walk() -> Result<()> {
        let logs = Logs::default();
        let version_format = VersionFormat::default();
        let graph = super::MemoizedCommitGraph::new(logs.clone(), &version_format);

        assert_ne!(logs.len(), 0);

        let logs2: Vec<String> = Bfs::new(graph.base_graph(), graph.headidx().unwrap())
            .iter(graph.base_graph())
            .map(|idx| {
                graph
                    .node_weight(idx)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .log_entry
                    .commit_hash
                    .to_string()
            })
            .collect();
        assert_eq!(logs.len(), logs2.len());

        Ok(())
    }
}
