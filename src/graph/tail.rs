use std::ops::{Deref, DerefMut};

use crate::graph::{
    GraphOps, commit::CommitExt, head::HasHead, parents_and_children::HasParentsAndChildren,
};
use petgraph::graph::NodeIndex;

pub struct TailMemo<T, Ix> {
    tailidx: NodeIndex<Ix>,
    inner: T,
}

pub trait HasTail<N, E, Ty, Ix> {
    fn tailidx(&self) -> Option<NodeIndex<Ix>>;
    fn tail(&self) -> Option<&N>;
}

impl<T, Ix> TailMemo<T, Ix> {
    pub fn new<N, E, Ty>(graph: T) -> Option<Self>
    where
        T: GraphOps<N, E, Ty, Ix> + HasParentsAndChildren<N, E, Ty, Ix>,
        Ix: Copy,
    {
        Some(TailMemo {
            tailidx: graph
                .node_identifiers()
                .into_iter()
                .find(|idx| graph.parents(*idx).is_empty())?,
            inner: graph,
        })
    }
}

impl<N, E, Ty, Ix, T> HasTail<N, E, Ty, Ix> for TailMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    Ix: Copy,
{
    fn tail(&self) -> Option<&N> {
        self.inner.node_weight(self.tailidx()?)
    }

    fn tailidx(&self) -> Option<NodeIndex<Ix>> {
        Some(self.tailidx)
    }
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for TailMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    Ix: Copy,
{
    fn node_weight(&self, idx: NodeIndex<Ix>) -> Option<&N> {
        self.inner.node_weight(idx)
    }

    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        self.inner.add_node(weight)
    }

    fn node_weight_mut(&mut self, idx: NodeIndex<Ix>) -> Option<&mut N> {
        self.inner.node_weight_mut(idx)
    }

    fn edge_weight(&self, idx: petgraph::prelude::EdgeIndex<Ix>) -> Option<&E> {
        self.inner.edge_weight(idx)
    }

    fn edge_weight_mut(&mut self, idx: petgraph::prelude::EdgeIndex<Ix>) -> Option<&mut E> {
        self.inner.edge_weight_mut(idx)
    }

    fn add_edge(
        &mut self,
        from: NodeIndex<Ix>,
        to: NodeIndex<Ix>,
        weight: E,
    ) -> Option<petgraph::prelude::EdgeIndex<Ix>> {
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

    fn edge_identifiers(&self) -> Vec<petgraph::prelude::EdgeIndex<Ix>> {
        self.inner.edge_identifiers()
    }

    fn edge_references(&self) -> Vec<(petgraph::prelude::EdgeIndex<Ix>, &E)> {
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

    fn neighbors_directed(
        &self,
        idx: NodeIndex<Ix>,
        dir: petgraph::Direction,
    ) -> Vec<NodeIndex<Ix>> {
        self.inner.neighbors_directed(idx, dir)
    }

    fn base_graph(&self) -> &petgraph::Graph<N, E, Ty, Ix> {
        self.inner.base_graph()
    }

    fn base_graph_mut(&mut self) -> &mut petgraph::Graph<N, E, Ty, Ix> {
        self.inner.base_graph_mut()
    }
}

// ParentChildExt is now implemented via blanket impl using HasBaseGraph

impl<T, Ix> Deref for TailMemo<T, Ix> {
    fn deref(&self) -> &T {
        &self.inner
    }

    type Target = T;
}

impl<T, Ix> DerefMut for TailMemo<T, Ix> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<N, E, Ty, Ix, T> HasParentsAndChildren<N, E, Ty, Ix> for TailMemo<T, Ix>
where
    T: HasParentsAndChildren<N, E, Ty, Ix>,
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

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for TailMemo<T, Ix>
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

impl<N, E, Ty, Ix, T> HasHead<N, E, Ty, Ix> for TailMemo<T, Ix>
where
    T: HasHead<N, E, Ty, Ix>,
{
    fn headidx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.headidx()
    }
    fn head(&self) -> Option<&N> {
        self.inner.head()
    }
}
