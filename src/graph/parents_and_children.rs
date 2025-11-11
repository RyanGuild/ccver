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
};

pub trait Parented {
    fn parents(&self) -> Vec<&str>;
}

impl Parented for LogEntry<'_> {
    #[inline]
    fn parents(&self) -> Vec<&str> {
        self.parent_hashes.iter().map(|s| *s).collect()
    }
}

impl Parented for CommitGraphNodeWeight<'_> {
    #[inline]
    fn parents(&self) -> Vec<&str> {
        // For node weights, we need to store strings since the guard will be dropped
        // This is called infrequently (only during graph construction)
        let guard = self.lock().unwrap();
        guard.log_entry.parent_hashes.iter().map(|s| *s).collect()
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
            waiting_edges: Vec::with_capacity(64),
        }
    }
}

pub trait HasParentsAndChildren<N, E, Ty, Ix> {
    fn parents(&self, idx: NodeIndex<Ix>) -> Vec<&N>;
    fn parent_idxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>>;
    fn children(&self, idx: NodeIndex<Ix>) -> Vec<&N>;
    fn child_idxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>>;
}

impl<N, E, Ty, Ix, T> HasParentsAndChildren<N, E, Ty, Ix> for WithParentsAndChildEdges<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix> + CommitExt<N, E, Ty, Ix>,
    Ix: Copy,
{
    fn parents(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        let neighbors = self.inner.neighbors_directed(idx, Direction::Outgoing);
        let mut result = Vec::with_capacity(neighbors.len());
        for idx in neighbors.iter() {
            if let Some(weight) = self.inner.node_weight(*idx) {
                result.push(weight);
            }
        }
        result
    }

    fn parent_idxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner
            .neighbors_directed(idx, Direction::Outgoing)
            .to_vec()
    }

    fn children(&self, idx: NodeIndex<Ix>) -> Vec<&N> {
        let neighbors = self.inner.neighbors_directed(idx, Direction::Incoming);
        let mut result = Vec::with_capacity(neighbors.len());
        for idx in neighbors.iter() {
            if let Some(weight) = self.inner.node_weight(*idx) {
                result.push(weight);
            }
        }
        result
    }

    fn child_idxs(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
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
        // Extract data before moving weight
        let parents: Vec<String> = weight.parents().iter().map(|s| s.to_string()).collect();
        let commit_hash = weight.commit_hash().to_string();
        let idx = self.inner.add_node(weight);
        for (waiting_idx, waiting_parent) in &self.waiting_edges {
            if waiting_parent == &commit_hash {
                self.inner.add_edge(*waiting_idx, idx, E::default());
            }
        }
        for parent in parents {
            match self.inner.commit_idx_by_hash(&parent) {
                Some(parent_idx) => {
                    self.inner.add_edge(idx, parent_idx, E::default());
                }
                None => {
                    self.waiting_edges.push((idx, parent));
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
    fn tail_idx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.tail_idx()
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
    fn commit_idx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>> {
        self.inner.commit_idx_by_hash(commit)
    }
}

#[cfg(test)]
mod parents_and_children_tests {
    use super::*;
    use crate::graph::{GraphOps, commit::CommitMemo};
    use crate::logs::{LogEntry, Subject};
    use petgraph::Directed;
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
    fn test_linear_commit_chain() {
        // Create a linear chain: c1 <- c2 <- c3
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec!["c1"]);
        let c3 = create_log_entry("c3", vec!["c2"]);

        let idx1 = graph.add_node(c1);
        let idx2 = graph.add_node(c2);
        let idx3 = graph.add_node(c3);

        // Verify parent relationships
        assert_eq!(graph.parent_idxs(idx1).len(), 0); // c1 has no parents
        assert_eq!(graph.parent_idxs(idx2), vec![idx1]); // c2's parent is c1
        assert_eq!(graph.parent_idxs(idx3), vec![idx2]); // c3's parent is c2

        // Verify child relationships
        assert_eq!(graph.child_idxs(idx1), vec![idx2]); // c1's child is c2
        assert_eq!(graph.child_idxs(idx2), vec![idx3]); // c2's child is c3
        assert_eq!(graph.child_idxs(idx3).len(), 0); // c3 has no children
    }

    #[test]
    fn test_merge_commit_multiple_parents() {
        // Create a merge: c1, c2 (both parents) <- c3 (merge commit)
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec![]);
        let c3 = create_log_entry("c3", vec!["c1", "c2"]); // merge commit

        let idx1 = graph.add_node(c1);
        let idx2 = graph.add_node(c2);
        let idx3 = graph.add_node(c3);

        // Verify c3 has two parents
        let parents = graph.parent_idxs(idx3);
        assert_eq!(parents.len(), 2);
        assert!(parents.contains(&idx1));
        assert!(parents.contains(&idx2));

        // Verify both parents list c3 as a child
        assert!(graph.child_idxs(idx1).contains(&idx3));
        assert!(graph.child_idxs(idx2).contains(&idx3));
    }

    #[test]
    fn test_out_of_order_addition_waiting_edges() {
        // Add child before parent to test waiting edges
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c2 = create_log_entry("c2", vec!["c1"]);
        let c1 = create_log_entry("c1", vec![]);

        // Add c2 first (parent c1 doesn't exist yet)
        let idx2 = graph.add_node(c2);

        // Verify waiting edge was created
        assert_eq!(graph.waiting_edges.len(), 1);
        assert_eq!(graph.waiting_edges[0].1, "c1");

        // Now add c1
        let idx1 = graph.add_node(c1);

        // Verify edge was created when c1 was added
        assert_eq!(graph.parent_idxs(idx2), vec![idx1]);
        assert_eq!(graph.child_idxs(idx1), vec![idx2]);
    }

    #[test]
    fn test_orphan_commit_no_parents() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let orphan = create_log_entry("orphan", vec![]);
        let idx = graph.add_node(orphan);

        // Orphan should have no parents and no children
        assert_eq!(graph.parent_idxs(idx).len(), 0);
        assert_eq!(graph.child_idxs(idx).len(), 0);
    }

    #[test]
    fn test_parents_and_children_trait_methods() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec!["c1"]);

        let idx1 = graph.add_node(c1);
        let idx2 = graph.add_node(c2);

        // Test parents() returns node weights, not just indices
        let parents = graph.parents(idx2);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].commit_hash, "c1");

        // Test children() returns node weights
        let children = graph.children(idx1);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].commit_hash, "c2");
    }

    #[test]
    fn test_branching_history() {
        // Create branching: c1 <- c2 and c1 <- c3
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec!["c1"]);
        let c3 = create_log_entry("c3", vec!["c1"]);

        let idx1 = graph.add_node(c1);
        let idx2 = graph.add_node(c2);
        let idx3 = graph.add_node(c3);

        // c1 should have two children
        let children = graph.child_idxs(idx1);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&idx2));
        assert!(children.contains(&idx3));

        // Both c2 and c3 should have c1 as parent
        assert_eq!(graph.parent_idxs(idx2), vec![idx1]);
        assert_eq!(graph.parent_idxs(idx3), vec![idx1]);
    }

    #[test]
    fn test_parented_trait() {
        let entry = create_log_entry("c1", vec!["parent1", "parent2"]);
        let parents = entry.parents();
        assert_eq!(parents.len(), 2);
        assert_eq!(parents[0], "parent1");
        assert_eq!(parents[1], "parent2");
    }

    #[test]
    #[should_panic(expected = "Cannot add edge to graph with parents and children")]
    fn test_manual_edge_addition_panics() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![]);
        let c2 = create_log_entry("c2", vec![]);

        let idx1 = graph.add_node(c1);
        let idx2 = graph.add_node(c2);

        // This should panic - edges are implied by parents
        graph.add_edge(idx1, idx2, ());
    }
}
