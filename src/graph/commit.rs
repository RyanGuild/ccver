use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    graph::{GraphOps, node::CommitGraphNodeWeight},
    logs::LogEntry,
};
use petgraph::{
    Direction, Graph,
    graph::{EdgeIndex, NodeIndex},
};

pub trait HasCommitHash {
    fn commit_hash(&self) -> String;
}

impl HasCommitHash for LogEntry<'_> {
    fn commit_hash(&self) -> String {
        self.commit_hash.to_string()
    }
}

impl HasCommitHash for CommitGraphNodeWeight<'_> {
    fn commit_hash(&self) -> String {
        self.lock().unwrap().log_entry.commit_hash.to_string()
    }
}

pub struct CommitMemo<T, Ix> {
    commitmemo: HashMap<String, NodeIndex<Ix>>,
    inner: T,
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for CommitMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasCommitHash,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let commit_hash = weight.commit_hash();
        let idx = self.inner.add_node(weight);
        self.commitmemo.insert(commit_hash, idx);
        idx
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

impl<Ix, T> CommitMemo<T, Ix> {
    pub fn new<N, E, Ty>(graph: T) -> CommitMemo<T, Ix>
    where
        T: GraphOps<N, E, Ty, Ix>,
        N: HasCommitHash,
        Ix: Copy,
    {
        let commitmemo = graph
            .node_identifiers()
            .iter()
            .map(|idx| (graph.node_weight(*idx).unwrap().commit_hash(), *idx))
            .collect();
        Self {
            commitmemo,
            inner: graph,
        }
    }
}

impl<Ix, T> Deref for CommitMemo<T, Ix> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Ix, T> DerefMut for CommitMemo<T, Ix> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait CommitExt<N, E, Ty, Ix> {
    fn commit_by_hash(&self, commit: &str) -> Option<&N>;
    fn commitidx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>>;
}

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for CommitMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasCommitHash,
    Ix: Copy,
{
    fn commit_by_hash(&self, commit: &str) -> Option<&N> {
        self.inner.node_weight(self.commitidx_by_hash(commit)?)
    }

    fn commitidx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>> {
        self.commitmemo.get(commit).cloned()
    }
}

// Intentionally do NOT implement Data for CommitMemo to avoid constraining T: Data,
// which would prevent layering wrappers that don't implement Data.

// Intentionally do NOT implement DataMap for CommitMemo for the same reason.

// Do not implement IntoEdgeReferences for CommitMemo; prefer algorithms that
// use neighbors/neighbors_directed to avoid recursive trait bounds.

// impl<T> IntoEdgeReferences for &CommitMemo<T>
// where
//     T: GraphBase<NodeId = NodeIndex, EdgeId = EdgeIndex> + IntoEdgeReferences + Data,
//     for<'c> &'c T: GraphBase<NodeId = NodeIndex, EdgeId = EdgeIndex> + IntoEdgeReferences,
// {
//     fn edge_references(self) -> Self::EdgeReferences {
//         self.inner.edge_references()
//     }

//     type EdgeReferences = <T as IntoEdgeReferences>::EdgeReferences;

//     type EdgeRef = <<T as IntoEdgeReferences>::EdgeRef as EdgeRef>::Weight;
// }
