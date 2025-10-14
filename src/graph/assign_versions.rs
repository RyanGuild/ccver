use crate::{
    graph::{
        GraphOps,
        commit::CommitExt,
        head::HasHead,
        node::CommitGraphNodeWeight,
        parents_and_children::HasParentsAndChildren,
        tail::HasTail,
        version::{ExistingVersionExt, SetVersionExt},
    },
    logs::LogEntry,
    version::Version,
    version_format::VersionFormat,
};
use petgraph::{
    Direction, EdgeType, Graph,
    csr::IndexType,
    graph::{EdgeIndex, NodeIndex},
    visit::{DfsPostOrder, Walker},
};
use tracing::debug;

pub struct WithCCVerVersions<T> {
    inner: T,
    version_format: VersionFormat,
}

pub trait AsLogEntry {
    fn as_log_entry(&self) -> LogEntry<'_>;
}

impl<'a> AsLogEntry for CommitGraphNodeWeight<'a> {
    fn as_log_entry(&self) -> LogEntry<'_> {
        let guard = self.lock().unwrap();
        guard.log_entry.clone()
    }
}

impl<'a> ExistingVersionExt for CommitGraphNodeWeight<'a> {
    fn as_existing_version(&self) -> Option<Version> {
        let data = self.lock().unwrap();
        data.as_existing_version()
    }
}

impl<T> WithCCVerVersions<T> {
    pub fn new<N, E, Ty, Ix>(mut inner: T, version_format: VersionFormat) -> Self
    where
        T: GraphOps<N, E, Ty, Ix>
            + HasParentsAndChildren<N, E, Ty, Ix>
            + HasTail<N, E, Ty, Ix>
            + HasHead<N, E, Ty, Ix>,
        N: ExistingVersionExt + AsLogEntry + SetVersionExt,
        Ix: IndexType,
        Ty: EdgeType,
    {
        let base = inner.base_graph();

        let mut last_version =
            version_format.as_default_version(&inner.head().unwrap().as_log_entry());
        // let reversed = Reversed(base);
        let node_count = base.node_count();
        let mut versions = Vec::with_capacity(node_count);
        for idx in DfsPostOrder::new(base, inner.head_idx().unwrap()).iter(base) {
            let weight = inner.node_weight(idx).unwrap();
            let log_entry = weight.as_log_entry();
            let version = inner
                .parents(idx)
                .iter()
                .filter_map(|p| p.as_existing_version())
                .max()
                .unwrap_or_else(|| last_version.clone())
                .next_version(&log_entry, &version_format);
            last_version = version.clone();
            versions.push((idx, version));
        }

        for (idx, version) in versions {
            let log_entry = inner.node_weight(idx).unwrap().as_log_entry();
            debug!(
                "Setting version {} for node: {:?} {} {:?}",
                version, idx, log_entry.commit_hash, log_entry.parent_hashes
            );
            inner.node_weight_mut(idx).unwrap().set_version(version);
        }

        Self {
            inner,
            version_format,
        }
    }
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for WithCCVerVersions<T>
where
    T: GraphOps<N, E, Ty, Ix> + HasParentsAndChildren<N, E, Ty, Ix>,
    N: ExistingVersionExt + AsLogEntry + SetVersionExt,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let idx = self.inner.add_node(weight);
        let weight = self.inner.node_weight(idx).unwrap();
        let log_entry = weight.as_log_entry();
        let max_parent = self
            .inner
            .parents(idx)
            .iter()
            .filter_map(|p| p.as_existing_version())
            .max()
            .unwrap();
        let version = max_parent.next_version(&log_entry, &self.version_format);
        self.inner
            .node_weight_mut(idx)
            .unwrap()
            .set_version(version);
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

impl<N, E, Ty, Ix, T> HasHead<N, E, Ty, Ix> for WithCCVerVersions<T>
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

impl<N, E, Ty, Ix, T> HasTail<N, E, Ty, Ix> for WithCCVerVersions<T>
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

impl<N, E, Ty, Ix, T> HasParentsAndChildren<N, E, Ty, Ix> for WithCCVerVersions<T>
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

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for WithCCVerVersions<T>
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
mod version_assignment_tests {
    use super::*;
    use crate::graph::{
        GraphOps, commit::CommitMemo, head::HeadMemo, node::CommitGraphNodeData,
        parents_and_children::WithParentsAndChildEdges, tail::TailMemo,
    };
    use crate::logs::{ConventionalSubject, Decoration, LogEntry, Subject};
    use petgraph::{Directed, Graph};
    use std::sync::Arc;
    use std::sync::Mutex;

    type TestNodeWeight = Arc<Mutex<CommitGraphNodeData<'static>>>;

    /// Helper to create a test LogEntry with conventional commit
    fn create_conventional_entry(
        hash: &'static str,
        parents: Vec<&'static str>,
        commit_type: &'static str,
        breaking: bool,
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
            subject: Subject::Conventional(ConventionalSubject {
                commit_type,
                breaking,
                scope: None,
                description: "test commit",
            }),
            footers: std::collections::HashMap::new(),
        }
    }

    /// Helper to create a test LogEntry with text subject
    fn create_text_entry(
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
    fn test_initial_version_assignment() {
        let graph: Graph<TestNodeWeight, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_text_entry("abcdef1", vec![], vec![Decoration::HeadIndicator("main")]);
        let data1 = Arc::new(Mutex::new(CommitGraphNodeData::from(c1)));

        let _idx1 = graph.add_node(data1.clone());

        let tail_graph = TailMemo::new(graph).unwrap();
        let head_graph = HeadMemo::new(tail_graph);
        let version_format = VersionFormat::default();
        let _versioned_graph = WithCCVerVersions::new(head_graph, version_format);

        // Verify initial version was assigned
        let node_data = data1.lock().unwrap();
        assert!(node_data.version.is_some());
    }

    #[test]
    fn test_patch_version_bump() {
        let graph: Graph<TestNodeWeight, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_text_entry("abcdef1", vec![], vec![]);
        let c2 = create_conventional_entry(
            "abcdef2",
            vec!["abcdef1"],
            "fix",
            false,
            vec![Decoration::HeadIndicator("main")],
        );

        let data1 = Arc::new(Mutex::new(CommitGraphNodeData::from(c1)));
        let data2 = Arc::new(Mutex::new(CommitGraphNodeData::from(c2)));

        graph.add_node(data1.clone());
        graph.add_node(data2.clone());

        let tail_graph = TailMemo::new(graph).unwrap();
        let head_graph = HeadMemo::new(tail_graph);
        let version_format = VersionFormat::default();
        let _versioned_graph = WithCCVerVersions::new(head_graph, version_format);

        // Verify versions were assigned
        let v1 = data1.lock().unwrap().version.clone();
        let v2 = data2.lock().unwrap().version.clone();

        assert!(v1.is_some());
        assert!(v2.is_some());

        // v2 should be greater than v1 (patch bump)
        assert!(v2.unwrap() > v1.unwrap());
    }

    #[test]
    fn test_minor_version_bump() {
        let graph: Graph<TestNodeWeight, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_text_entry("abcdef1", vec![], vec![]);
        let c2 = create_conventional_entry(
            "abcdef2",
            vec!["abcdef1"],
            "feat",
            false,
            vec![Decoration::HeadIndicator("main")],
        );

        let data1 = Arc::new(Mutex::new(CommitGraphNodeData::from(c1)));
        let data2 = Arc::new(Mutex::new(CommitGraphNodeData::from(c2)));

        graph.add_node(data1.clone());
        graph.add_node(data2.clone());

        let tail_graph = TailMemo::new(graph).unwrap();
        let head_graph = HeadMemo::new(tail_graph);
        let version_format = VersionFormat::default();
        let _versioned_graph = WithCCVerVersions::new(head_graph, version_format);

        // Verify feature bump occurred
        let v1 = data1.lock().unwrap().version.clone();
        let v2 = data2.lock().unwrap().version.clone();

        assert!(v1.is_some());
        assert!(v2.is_some());
        assert!(v2.unwrap() > v1.unwrap());
    }

    #[test]
    fn test_major_version_bump_breaking() {
        let graph: Graph<TestNodeWeight, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        let c1 = create_text_entry("abcdef1", vec![], vec![]);
        let c2 = create_conventional_entry(
            "abcdef2",
            vec!["abcdef1"],
            "feat",
            true,
            vec![Decoration::HeadIndicator("main")],
        );

        let data1 = Arc::new(Mutex::new(CommitGraphNodeData::from(c1)));
        let data2 = Arc::new(Mutex::new(CommitGraphNodeData::from(c2)));

        graph.add_node(data1.clone());
        graph.add_node(data2.clone());

        let tail_graph = TailMemo::new(graph).unwrap();
        let head_graph = HeadMemo::new(tail_graph);
        let version_format = VersionFormat::default();
        let _versioned_graph = WithCCVerVersions::new(head_graph, version_format);

        // Verify breaking change caused version bump
        let v1 = data1.lock().unwrap().version.clone();
        let v2 = data2.lock().unwrap().version.clone();

        assert!(v1.is_some());
        assert!(v2.is_some());
        assert!(v2.unwrap() > v1.unwrap());
    }

    #[test]
    fn test_version_inheritance_linear_chain() {
        let graph: Graph<TestNodeWeight, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        // Create a linear chain: c1 <- c2 <- c3
        let c1 = create_text_entry("abcdef1", vec![], vec![]);
        let c2 = create_conventional_entry("abcdef2", vec!["abcdef1"], "fix", false, vec![]);
        let c3 = create_conventional_entry(
            "abcdef3",
            vec!["abcdef2"],
            "feat",
            false,
            vec![Decoration::HeadIndicator("main")],
        );

        let data1 = Arc::new(Mutex::new(CommitGraphNodeData::from(c1)));
        let data2 = Arc::new(Mutex::new(CommitGraphNodeData::from(c2)));
        let data3 = Arc::new(Mutex::new(CommitGraphNodeData::from(c3)));

        graph.add_node(data1.clone());
        graph.add_node(data2.clone());
        graph.add_node(data3.clone());

        let tail_graph = TailMemo::new(graph).unwrap();
        let head_graph = HeadMemo::new(tail_graph);
        let version_format = VersionFormat::default();
        let _versioned_graph = WithCCVerVersions::new(head_graph, version_format);

        // Verify versions increase monotonically
        let v1 = data1.lock().unwrap().version.clone().unwrap();
        let v2 = data2.lock().unwrap().version.clone().unwrap();
        let v3 = data3.lock().unwrap().version.clone().unwrap();

        assert!(v2 > v1);
        assert!(v3 > v2);
    }

    #[test]
    fn test_merge_commit_max_parent_version() {
        let graph: Graph<TestNodeWeight, (), Directed, u32> = Graph::new();
        let memo = CommitMemo::new(graph);
        let mut graph = WithParentsAndChildEdges::new(memo);

        // Create a merge: c1, c2 <- c3 (merge)
        let c1 = create_conventional_entry("abcdef1", vec![], "feat", false, vec![]);
        let c2 = create_conventional_entry("abcdef2", vec![], "fix", false, vec![]);
        let c3 = create_text_entry(
            "abcdef3",
            vec!["abcdef1", "abcdef2"],
            vec![Decoration::HeadIndicator("main")],
        );

        let data1 = Arc::new(Mutex::new(CommitGraphNodeData::from(c1)));
        let data2 = Arc::new(Mutex::new(CommitGraphNodeData::from(c2)));
        let data3 = Arc::new(Mutex::new(CommitGraphNodeData::from(c3)));

        graph.add_node(data1.clone());
        graph.add_node(data2.clone());
        graph.add_node(data3.clone());

        let tail_graph = TailMemo::new(graph).unwrap();
        let head_graph = HeadMemo::new(tail_graph);
        let version_format = VersionFormat::default();
        let _versioned_graph = WithCCVerVersions::new(head_graph, version_format);

        // Merge should inherit max of parent versions
        let v1 = data1.lock().unwrap().version.clone().unwrap();
        let v2 = data2.lock().unwrap().version.clone().unwrap();
        let v3 = data3.lock().unwrap().version.clone().unwrap();

        let max_parent = v1.max(v2);
        assert!(v3 >= max_parent);
    }

    #[test]
    fn test_as_log_entry_trait() {
        let entry = create_text_entry("abcdef1", vec![], vec![]);
        let data = Arc::new(Mutex::new(CommitGraphNodeData::from(entry.clone())));

        let log_entry = data.as_log_entry();
        assert_eq!(log_entry.commit_hash, "abcdef1");
    }
}
