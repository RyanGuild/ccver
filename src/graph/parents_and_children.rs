use core::panic;
use std::ops::{Deref, DerefMut};

use crate::{
    graph::{
        GraphOps,
        commit::{CommitExt, HasCommitHash},
        node::CommitGraphNodeWeight,
        tail::HasTail,
    },
    logs::LogEntry,
};
use petgraph::{
    Direction, Graph,
    graph::{EdgeIndex, NodeIndex},
    visit::{IntoNeighborsDirected, IntoNodeIdentifiers},
};

pub trait Parented {
    fn parents(&self) -> Vec<String>;
}

impl Parented for LogEntry<'_> {
    fn parents(&self) -> Vec<String> {
        self.parent_hashes
            .clone()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Parented for CommitGraphNodeWeight<'_> {
    fn parents(&self) -> Vec<String> {
        self.lock().unwrap().log_entry.parents()
    }
}

pub struct WithParentsAndChildEdges<T, Ix> {
    inner: T,
    waiting_edges: Vec<(NodeIndex<Ix>, String)>,
}

impl<T, Ix> WithParentsAndChildEdges<T, Ix> {
    pub fn new<N, E, Ty>(inner: T) -> Self
    where
        T: GraphOps<N, E, Ty, Ix> + CommitExt<N, E, Ty, Ix>,
    {
        Self {
            inner,
            waiting_edges: vec![],
        }
    }
}

pub trait HasParentsAndChildren<N, E, Ty, Ix> {
    fn parents(&self, idx: NodeIndex<Ix>) -> Vec<&N>;
    fn parentidxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>>;
    fn children(&self, idx: NodeIndex<Ix>) -> Vec<&N>;
    fn childidxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>>;
}

impl<N, E, Ty, Ix, T> HasParentsAndChildren<N, E, Ty, Ix> for WithParentsAndChildEdges<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix> + CommitExt<N, E, Ty, Ix>,
    Ix: Copy,
{
    fn parents(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        self.inner
            .neighbors_directed(idx, Direction::Outgoing)
            .iter()
            .filter_map(|idx| self.inner.node_weight(*idx))
            .collect()
    }

    fn parentidxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner
            .neighbors_directed(idx, Direction::Outgoing)
            .to_vec()
    }

    fn children(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        self.inner
            .neighbors_directed(idx, Direction::Incoming)
            .iter()
            .filter_map(|idx| self.inner.node_weight(*idx))
            .collect()
    }

    fn childidxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner
            .neighbors_directed(idx, Direction::Incoming)
            .to_vec()
    }
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for WithParentsAndChildEdges<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix> + CommitExt<N, E, Ty, Ix>,
    N: Parented + HasCommitHash,
    E: Default,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let parents = weight.parents();
        let commit_hash = weight.commit_hash();
        let idx = self.inner.add_node(weight);
        for (waiting_idx, waiting_parent) in &self.waiting_edges {
            if waiting_parent == &commit_hash {
                self.inner.add_edge(*waiting_idx, idx, E::default());
            }
        }
        for parent in parents {
            match self.inner.commitidx_by_hash(&parent) {
                Some(parent_idx) => {
                    self.inner.add_edge(idx, parent_idx, E::default());
                }
                None => {
                    self.waiting_edges.push((idx, parent.clone()));
                }
            };
        }

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
        _from: NodeIndex<Ix>,
        _to: NodeIndex<Ix>,
        _weight: E,
    ) -> Option<EdgeIndex<Ix>> {
        panic!(
            "Cannot add edge to graph with parents and children; edges are implied by the parents"
        );
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

impl<T, Ix> Deref for WithParentsAndChildEdges<T, Ix> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, Ix> DerefMut for WithParentsAndChildEdges<T, Ix> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<N, E, Ty, Ix, T> HasTail<N, E, Ty, Ix> for WithParentsAndChildEdges<T, Ix>
where
    T: HasTail<N, E, Ty, Ix>,
{
    fn tailidx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.tailidx()
    }
    fn tail(&self) -> Option<&N> {
        self.inner.tail()
    }
}

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for WithParentsAndChildEdges<T, Ix>
where
    T: CommitExt<N, E, Ty, Ix>,
{
    fn commit_by_hash(&self, commit: &str) -> Option<&N> {
        self.inner.commit_by_hash(commit)
    }
    fn commitidx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>> {
        self.inner.commitidx_by_hash(commit)
    }
}
