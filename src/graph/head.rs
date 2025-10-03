use std::ops::{Deref, DerefMut};

use crate::{
    graph::{
        GraphOps, commit::CommitExt, node::CommitGraphNodeWeight,
        parents_and_children::HasParentsAndChildren, tail::HasTail,
    },
    logs::{Decoration, LogEntry},
};
use petgraph::{
    Direction, Graph,
    graph::{EdgeIndex, NodeIndex},
};

pub struct HeadMemo<T, Ix> {
    head_idx: NodeIndex<Ix>,
    inner: T,
}

impl<T, Ix> HeadMemo<T, Ix> {
    pub fn new<N, E, Ty>(inner: T) -> Self
    where
        T: GraphOps<N, E, Ty, Ix>,
        N: Headed,
        Ix: Copy,
    {
        let head_idx = inner
            .node_identifiers()
            .into_iter()
            .find(|idx| inner.node_weight(*idx).unwrap().is_current_head())
            .unwrap();
        Self { head_idx, inner }
    }
}

impl<T, Ix> Deref for HeadMemo<T, Ix> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, Ix> DerefMut for HeadMemo<T, Ix> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait Headed {
    fn is_current_head(&self) -> bool;
    fn current_head_branch(&self) -> Option<String>;
}

impl Headed for LogEntry<'_> {
    fn is_current_head(&self) -> bool {
        self.decorations.iter().any(|d| match d {
            Decoration::HeadIndicator(_) => true,
            _ => false,
        })
    }
    fn current_head_branch(&self) -> Option<String> {
        self.decorations.iter().find_map(|d| match d {
            Decoration::HeadIndicator(b) => Some(b.to_string()),
            _ => None,
        })
    }
}

impl Headed for CommitGraphNodeWeight<'_> {
    fn is_current_head(&self) -> bool {
        self.lock().unwrap().log_entry.is_current_head()
    }
    fn current_head_branch(&self) -> Option<String> {
        self.lock().unwrap().log_entry.current_head_branch()
    }
}

impl<N, E, Ty, Ix, G> GraphOps<N, E, Ty, Ix> for HeadMemo<G, Ix>
where
    G: GraphOps<N, E, Ty, Ix>,
    N: Headed,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        if weight.is_current_head() {
            let head_idx = self.inner.add_node(weight);
            self.head_idx = head_idx;
            head_idx
        } else {
            self.inner.add_node(weight)
        }
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

pub trait HasHead<N, E, Ty, Ix> {
    fn head_idx(&self) -> Option<NodeIndex<Ix>>;
    fn head(&self) -> Option<&N>;
}

impl<N, E, Ty, Ix, T> HasHead<N, E, Ty, Ix> for HeadMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    Ix: Copy,
{
    fn head_idx(&self) -> Option<NodeIndex<Ix>> {
        Some(self.head_idx)
    }
    fn head(&self) -> Option<&N> {
        self.inner.node_weight(self.head_idx()?)
    }
}

impl<N, E, Ty, Ix, T> HasTail<N, E, Ty, Ix> for HeadMemo<T, Ix>
where
    T: HasTail<N, E, Ty, Ix>,
{
    fn tail_idx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.tail_idx()
    }
    fn tail(&self) -> Option<&N> {
        self.inner.tail()
    }
}

impl<N, E, Ty, Ix, T> HasParentsAndChildren<N, E, Ty, Ix> for HeadMemo<T, Ix>
where
    T: HasParentsAndChildren<N, E, Ty, Ix>,
{
    fn parents(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        self.inner.parents(idx)
    }
    fn parent_idxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner.parent_idxs(idx)
    }
    fn children(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        self.inner.children(idx)
    }
    fn child_idxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner.child_idxs(idx)
    }
}

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for HeadMemo<T, Ix>
where
    T: CommitExt<N, E, Ty, Ix>,
{
    fn commit_by_hash(&self, commit: &str) -> Option<&N> {
        self.inner.commit_by_hash(commit)
    }
    fn commit_idx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>> {
        self.inner.commit_idx_by_hash(commit)
    }
}
