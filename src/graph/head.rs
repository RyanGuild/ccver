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
        self.decorations
            .iter()
            .any(|d| matches!(d, Decoration::HeadIndicator(_)))
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
        let guard = self.lock().unwrap();
        guard.log_entry.is_current_head()
    }
    fn current_head_branch(&self) -> Option<String> {
        let guard = self.lock().unwrap();
        guard.log_entry.current_head_branch()
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

#[cfg(test)]
mod head_tests {
    use super::*;
    use crate::graph::{
        GraphOps, commit::CommitMemo, parents_and_children::WithParentsAndChildEdges,
    };
    use crate::logs::{Decoration, LogEntry, Subject};
    use petgraph::Directed;
    use std::sync::Arc;

    /// Helper to create a test LogEntry
    fn create_log_entry(
        hash: &'static str,
        parents: Vec<&'static str>,
        decorations: Vec<Decoration<'static>>,
    ) -> LogEntry<'static> {
        LogEntry {
            name: "Test User",
            branch: "main",
            commit_hash: hash,
            commit_timezone: chrono::Utc,
            commit_datetime: chrono::Utc::now(),
            parent_hashes: Arc::from(parents),
            decorations: Arc::from(decorations),
            subject: Subject::Text("test commit"),
            footers: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_head_detection() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        // Add commits, last one with HEAD decoration
        let c1 = create_log_entry("c1", vec![], vec![]);
        let c2 = create_log_entry("c2", vec!["c1"], vec![]);
        let c3 = create_log_entry("c3", vec!["c2"], vec![Decoration::HeadIndicator("main")]);

        let _idx1 = graph.add_node(c1);
        let _idx2 = graph.add_node(c2);
        let idx3 = graph.add_node(c3);

        // Create HeadMemo
        let head_graph = HeadMemo::new(graph);

        // Verify HEAD is correctly identified
        assert_eq!(head_graph.head_idx(), Some(idx3));
        let head = head_graph.head().unwrap();
        assert_eq!(head.commit_hash, "c3");
    }

    #[test]
    fn test_head_memoization() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![], vec![Decoration::HeadIndicator("main")]);
        let idx1 = graph.add_node(c1);

        let head_graph = HeadMemo::new(graph);

        // Multiple calls should return the same memoized value
        assert_eq!(head_graph.head_idx(), Some(idx1));
        assert_eq!(head_graph.head_idx(), Some(idx1));
        assert!(head_graph.head().is_some());
    }

    #[test]
    fn test_dynamic_head_update() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![], vec![Decoration::HeadIndicator("main")]);
        let idx1 = graph.add_node(c1);

        let mut head_graph = HeadMemo::new(graph);
        assert_eq!(head_graph.head_idx(), Some(idx1));

        // Add a new HEAD commit
        let c2 = create_log_entry("c2", vec!["c1"], vec![Decoration::HeadIndicator("main")]);
        let idx2 = head_graph.add_node(c2);

        // HEAD should be updated
        assert_eq!(head_graph.head_idx(), Some(idx2));
    }

    #[test]
    fn test_head_branch_extraction() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![], vec![Decoration::HeadIndicator("develop")]);
        graph.add_node(c1);

        let head_graph = HeadMemo::new(graph);
        let head = head_graph.head().unwrap();

        assert!(head.is_current_head());
        assert_eq!(head.current_head_branch(), Some("develop".to_string()));
    }

    #[test]
    fn test_headed_trait() {
        // Test on LogEntry
        let entry_with_head =
            create_log_entry("c1", vec![], vec![Decoration::HeadIndicator("main")]);
        assert!(entry_with_head.is_current_head());
        assert_eq!(
            entry_with_head.current_head_branch(),
            Some("main".to_string())
        );

        // Test on LogEntry without HEAD
        let entry_without_head = create_log_entry("c2", vec![], vec![]);
        assert!(!entry_without_head.is_current_head());
        assert_eq!(entry_without_head.current_head_branch(), None);
    }

    #[test]
    fn test_head_with_multiple_decorations() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry(
            "c1",
            vec![],
            vec![
                Decoration::Branch("main"),
                Decoration::HeadIndicator("main"),
                Decoration::RemoteBranch(("origin", "main")),
            ],
        );
        let idx1 = graph.add_node(c1);

        let head_graph = HeadMemo::new(graph);

        // Should still find HEAD even with multiple decorations
        assert_eq!(head_graph.head_idx(), Some(idx1));
        let head = head_graph.head().unwrap();
        assert!(head.is_current_head());
    }

    #[test]
    fn test_head_forwarding_traits() {
        let graph: Graph<LogEntry, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_log_entry("c1", vec![], vec![]);
        let c2 = create_log_entry("c2", vec!["c1"], vec![Decoration::HeadIndicator("main")]);

        let idx1 = graph.add_node(c1);
        let idx2 = graph.add_node(c2);

        let head_graph = HeadMemo::new(graph);

        // Verify HasParentsAndChildren is forwarded correctly
        let parents = head_graph.parent_idxs(idx2);
        assert_eq!(parents, vec![idx1]);

        // Verify CommitExt is forwarded correctly
        assert!(head_graph.commit_by_hash("c1").is_some());
        assert_eq!(head_graph.commit_idx_by_hash("c2"), Some(idx2));
    }
}
