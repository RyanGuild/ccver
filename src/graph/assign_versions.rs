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
    visit::{DfsPostOrder, Reversed, Walker},
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
        self.lock().unwrap().log_entry.clone()
    }
}

impl<'a> ExistingVersionExt for CommitGraphNodeWeight<'a> {
    fn as_existing_version(&self) -> Option<Version> {
        let data = self.lock().unwrap();
        data.as_existing_version()
    }
}

impl<'a, T> WithCCVerVersions<T> {
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
        let versions = DfsPostOrder::new(base, inner.headidx().unwrap())
            .iter(base)
            .map(|idx| {
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
                (idx, version)
            })
            .collect::<Vec<_>>();

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
    fn headidx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.headidx()
    }
    fn head(&self) -> Option<&N> {
        self.inner.head()
    }
}

impl<N, E, Ty, Ix, T> HasTail<N, E, Ty, Ix> for WithCCVerVersions<T>
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

impl<N, E, Ty, Ix, T> HasParentsAndChildren<N, E, Ty, Ix> for WithCCVerVersions<T>
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

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for WithCCVerVersions<T>
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
