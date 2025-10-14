use std::ops::{Deref, DerefMut};

use crate::graph::{
    GraphOps, commit::CommitExt, head::HasHead, parents_and_children::HasParentsAndChildren,
};
use petgraph::graph::NodeIndex;

pub struct TailMemo<T, Ix> {
    tail_idx: NodeIndex<Ix>,
    inner: T,
}

pub trait HasTail<N, E, Ty, Ix> {
    fn tail_idx(&self) -> Option<NodeIndex<Ix>>;
    fn tail(&self) -> Option<&N>;
}

impl<T, Ix> TailMemo<T, Ix> {
    pub fn new<N, E, Ty>(graph: T) -> Option<Self>
    where
        T: GraphOps<N, E, Ty, Ix> + HasParentsAndChildren<N, E, Ty, Ix>,
        Ix: Copy,
    {
        Some(TailMemo {
            tail_idx: graph
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
        self.inner.node_weight(self.tail_idx()?)
    }

    fn tail_idx(&self) -> Option<NodeIndex<Ix>> {
        Some(self.tail_idx)
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

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for TailMemo<T, Ix>
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

impl<N, E, Ty, Ix, T> HasHead<N, E, Ty, Ix> for TailMemo<T, Ix>
where
    T: HasHead<N, E, Ty, Ix>,
{
    fn head_idx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.head_idx()
    }
    fn head(&self) -> Option<&N> {
        self.inner.head()
    }
}

#[cfg(test)]
mod tail_tests {
    use super::*;
    use crate::graph::{
        GraphOps, commit::CommitMemo, parents_and_children::WithParentsAndChildEdges,
    };
    use crate::logs::{LogEntry, Subject};
    use petgraph::{Directed, Graph};
    use std::sync::Arc;

    /// Helper to create a test LogEntry
    fn create_log_entry(hash: &'static str, parents: Vec<&'static str>) -> LogEntry<'static> {
        LogEntry {
            name: "Test User",
            branch: "main",
            commit_hash: hash,
            commit_timezone: chrono::Utc,
            commit_datetime: chrono::Utc::now(),
            parent_hashes: Arc::from(parents),
            decorations: Arc::from(vec![]),
            subject: Subject::Text("test commit"),
            footers: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_tail_detection_linear_chain() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        // Create a linear chain: c1 <- c2 <- c3
        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec!["c1"]);
        let c3 = create_log_entry("c3", vec!["c2"]);

        let idx1 = graph.add_node(c1);
        let _idx2 = graph.add_node(c2);
        let _idx3 = graph.add_node(c3);

        // Create TailMemo
        let tail_graph = TailMemo::new(graph).unwrap();

        // c1 should be the tail (no parents)
        assert_eq!(tail_graph.tail_idx(), Some(idx1));
        let tail = tail_graph.tail().unwrap();
        assert_eq!(tail.commit_hash, "c1");
    }

    #[test]
    fn test_tail_has_no_parents() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec!["c1"]);

        let idx1 = graph.add_node(c1);
        graph.add_node(c2);

        let tail_graph = TailMemo::new(graph).unwrap();

        // Tail should have no parents
        let tail_idx = tail_graph.tail_idx().unwrap();
        assert_eq!(tail_idx, idx1);
        let parents = tail_graph.parent_idxs(tail_idx);
        assert_eq!(parents.len(), 0);
    }

    #[test]
    fn test_tail_memoization() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let idx1 = graph.add_node(c1);

        let tail_graph = TailMemo::new(graph).unwrap();

        // Multiple calls should return the same memoized value
        assert_eq!(tail_graph.tail_idx(), Some(idx1));
        assert_eq!(tail_graph.tail_idx(), Some(idx1));
        assert!(tail_graph.tail().is_some());
    }

    #[test]
    fn test_tail_with_branching() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        // Create branching from root: c1 <- c2 and c1 <- c3
        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec!["c1"]);
        let c3 = create_log_entry("c3", vec!["c1"]);

        let idx1 = graph.add_node(c1);
        graph.add_node(c2);
        graph.add_node(c3);

        let tail_graph = TailMemo::new(graph).unwrap();

        // c1 is still the tail even with branching
        assert_eq!(tail_graph.tail_idx(), Some(idx1));
    }

    #[test]
    fn test_tail_with_merge_commit() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        // Create a merge scenario: c1, c2 (roots) <- c3 (merge)
        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec![]);
        let c3 = create_log_entry("c3", vec!["c1", "c2"]);

        let idx1 = graph.add_node(c1);
        graph.add_node(c2);
        graph.add_node(c3);

        let tail_graph = TailMemo::new(graph).unwrap();

        // Should find one of the roots (c1 or c2)
        let tail_idx = tail_graph.tail_idx().unwrap();
        assert!(tail_idx == idx1 || tail_graph.parent_idxs(tail_idx).is_empty());
    }

    #[test]
    fn test_tail_none_when_no_root() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let graph = WithParentsAndChildEdges::new(memo);

        // Empty graph has no tail
        let tail_graph = TailMemo::new(graph);
        assert!(tail_graph.is_none());
    }

    #[test]
    fn test_tail_forwarding_traits() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec!["c1"]);

        let idx1 = graph.add_node(c1);
        let idx2 = graph.add_node(c2);

        let tail_graph = TailMemo::new(graph).unwrap();

        // Verify HasParentsAndChildren is forwarded correctly
        let parents = tail_graph.parent_idxs(idx2);
        assert_eq!(parents, vec![idx1]);

        // Verify CommitExt is forwarded correctly
        assert!(tail_graph.commit_by_hash("c1").is_some());
        assert_eq!(tail_graph.commit_idx_by_hash("c2"), Some(idx2));
    }

    #[test]
    fn test_single_commit_is_tail() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let idx1 = graph.add_node(c1);

        let tail_graph = TailMemo::new(graph).unwrap();

        // Single commit should be the tail
        assert_eq!(tail_graph.tail_idx(), Some(idx1));
    }
}
