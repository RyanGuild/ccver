use rustc_hash::FxHashMap;
use std::ops::{Deref, DerefMut};

use crate::{
    graph::{GraphOps, node::CommitGraphNodeWeight},
    logs::LogEntry,
};
use petgraph::{
    Direction, Graph,
    graph::{EdgeIndex, NodeIndex},
};

pub trait HasCommitHash {
    fn commit_hash(&self) -> &str;
}

impl HasCommitHash for LogEntry<'_> {
    #[inline]
    fn commit_hash(&self) -> &str {
        self.commit_hash
    }
}

impl HasCommitHash for CommitGraphNodeWeight<'_> {
    #[inline]
    fn commit_hash(&self) -> &str {
        let guard = self.lock().unwrap();
        guard.log_entry.commit_hash
    }
}

pub struct CommitMemo<T, Ix> {
    commit_memo: FxHashMap<String, NodeIndex<Ix>>,
    inner: T,
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for CommitMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasCommitHash,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let commit_hash = weight.commit_hash().to_string();
        let idx = self.inner.add_node(weight);
        self.commit_memo.insert(commit_hash, idx);
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
        let node_count = graph.node_count();
        let mut commit_memo =
            FxHashMap::with_capacity_and_hasher(node_count.max(64), Default::default());
        for idx in graph.node_identifiers().iter() {
            let hash = graph.node_weight(*idx).unwrap().commit_hash().to_string();
            commit_memo.insert(hash, *idx);
        }
        Self {
            commit_memo,
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
    fn commit_idx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>>;
}

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for CommitMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasCommitHash,
    Ix: Copy,
{
    fn commit_by_hash(&self, commit: &str) -> Option<&N> {
        self.inner.node_weight(self.commit_idx_by_hash(commit)?)
    }

    fn commit_idx_by_hash(&self, commit: &str) -> Option<NodeIndex<Ix>> {
        self.commit_memo.get(commit).cloned()
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

#[cfg(test)]
mod commit_tests {
    use super::*;
    use crate::logs::{LogEntry, Subject};
    use petgraph::{Directed, Graph};
    use std::sync::Arc;

    /// Helper to create a simple LogEntry for testing
    fn create_test_log_entry(hash: &'static str, parent: &'static str) -> LogEntry<'static> {
        LogEntry {
            name: "Test User",
            branch: "main",
            commit_hash: hash,
            commit_timezone: chrono::Utc,
            commit_datetime: chrono::Utc::now(),
            parent_hashes: Arc::from([parent]),
            decorations: Arc::from(vec![]),
            subject: Subject::Text("test commit"),
            footers: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_commit_memo_indexing() {
        use super::super::GraphOps;

        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let mut memo = CommitMemo::new(graph);

        let entry1 = create_test_log_entry("abc123", "");
        let entry2 = create_test_log_entry("def456", "abc123");
        let entry3 = create_test_log_entry("ghi789", "def456");

        let idx1 = memo.add_node(entry1);
        let idx2 = memo.add_node(entry2);
        let idx3 = memo.add_node(entry3);

        // Verify all commits are indexed
        assert_eq!(memo.commit_memo.len(), 3);

        // Verify we can look up by hash
        assert_eq!(memo.commit_idx_by_hash("abc123"), Some(idx1));
        assert_eq!(memo.commit_idx_by_hash("def456"), Some(idx2));
        assert_eq!(memo.commit_idx_by_hash("ghi789"), Some(idx3));
    }

    #[test]
    fn test_commit_lookup_by_hash() {
        use crate::graph::GraphOps;

        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let mut memo = CommitMemo::new(graph);

        let entry1 = create_test_log_entry("abc123", "");
        let entry2 = create_test_log_entry("def456", "abc123");

        memo.add_node(entry1);
        memo.add_node(entry2);

        // Test successful lookup
        let found = memo.commit_by_hash("abc123");
        assert!(found.is_some());
        assert_eq!(found.unwrap().commit_hash, "abc123");

        // Test another successful lookup
        let found2 = memo.commit_by_hash("def456");
        assert!(found2.is_some());
        assert_eq!(found2.unwrap().commit_hash, "def456");
    }

    #[test]
    fn test_commit_lookup_nonexistent() {
        use crate::graph::GraphOps;

        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let mut memo = CommitMemo::new(graph);

        let entry = create_test_log_entry("abc123", "");
        memo.add_node(entry);

        // Lookup non-existent hash
        assert!(memo.commit_by_hash("nonexistent").is_none());
        assert!(memo.commit_idx_by_hash("nonexistent").is_none());
    }

    #[test]
    fn test_commit_memo_construction_from_existing_graph() {
        let mut graph: Graph<LogEntry, (), Directed, u32> = Graph::new();

        let entry1 = create_test_log_entry("abc123", "");
        let entry2 = create_test_log_entry("def456", "abc123");

        let idx1 = graph.add_node(entry1);
        let idx2 = graph.add_node(entry2);

        // Create CommitMemo from existing graph
        let memo = CommitMemo::new(graph);

        // Verify all existing nodes are indexed
        assert_eq!(memo.commit_memo.len(), 2);
        assert_eq!(memo.commit_idx_by_hash("abc123"), Some(idx1));
        assert_eq!(memo.commit_idx_by_hash("def456"), Some(idx2));
    }

    #[test]
    fn test_commit_hash_trait() {
        let entry = create_test_log_entry("test_hash_123", "parent_hash");
        assert_eq!(entry.commit_hash(), "test_hash_123");
    }
}
