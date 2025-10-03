use std::collections::HashMap;

use petgraph::{
    Direction, Graph,
    graph::{EdgeIndex, NodeIndex},
};

use crate::{
    graph::{
        GraphOps, assign_versions::AsLogEntry, commit::CommitExt, head::HasHead,
        parents_and_children::HasParentsAndChildren, tail::HasTail,
    },
    logs::Decoration,
};

pub trait HasBranches {
    fn branch(&self) -> Vec<&str>;
}

impl<T> HasBranches for T
where
    T: AsLogEntry,
{
    fn branch(&self) -> Vec<&str> {
        self.as_log_entry()
            .decorations
            .iter()
            .filter_map(|d| match d {
                Decoration::Branch(b) => Some(*b),
                Decoration::RemoteBranch((_, b)) => Some(*b),
                Decoration::HeadIndicator(b) => Some(*b),
                _ => None,
            })
            .collect()
    }
}

pub trait HasLocalBranches {
    fn local_branch(&self) -> Vec<&str>;
}

impl<T> HasLocalBranches for T
where
    T: AsLogEntry,
{
    fn local_branch(&self) -> Vec<&str> {
        self.as_log_entry()
            .decorations
            .iter()
            .filter_map(|d| match d {
                Decoration::Branch(b) => Some(*b),
                _ => None,
            })
            .collect()
    }
}

pub trait HasRemoteBranches {
    fn remote_branch(&self) -> Vec<(&str, &str)>;
}

impl<T> HasRemoteBranches for T
where
    T: AsLogEntry,
{
    fn remote_branch(&self) -> Vec<(&str, &str)> {
        self.as_log_entry()
            .decorations
            .iter()
            .filter_map(|d| match d {
                Decoration::RemoteBranch((o, b)) => Some((*o, *b)),
                _ => None,
            })
            .collect()
    }
}

pub trait BranchExt<N, E, Ty, Ix> {
    fn branch(&self, branch: &str) -> Vec<N>;
    fn branch_idx(&self, branch: &str) -> Vec<NodeIndex<Ix>>;

    fn local_branch(&self, branch: &str) -> Vec<N>;
    fn local_branch_idx(&self, branch: &str) -> Vec<NodeIndex<Ix>>;

    fn remote_branch(&self, remote: &str, branch: &str) -> Vec<N>;
    fn remote_branch_idx(&self, remote: &str, branch: &str) -> Vec<NodeIndex<Ix>>;
}
pub struct BranchMemo<T, Ix> {
    branch_memo: HashMap<String, NodeIndex<Ix>>,
    inner: T,
    local_branch_memo: HashMap<String, NodeIndex<Ix>>,
    remote_branch_memo: HashMap<(String, String), NodeIndex<Ix>>,
}

impl<T, Ix> BranchMemo<T, Ix> {
    pub fn new<N, E, Ty>(graph: T) -> Self
    where
        T: GraphOps<N, E, Ty, Ix>,
        N: HasBranches + HasLocalBranches + HasRemoteBranches,
    {
        Self {
            branch_memo: HashMap::new(),
            local_branch_memo: HashMap::new(),
            remote_branch_memo: HashMap::new(),
            inner: graph,
        }
    }
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for BranchMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasBranches + HasLocalBranches + HasRemoteBranches + Clone,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let clone = weight.clone();
        let idx = self.inner.add_node(weight);

        clone.branch().iter().for_each(|branch| {
            self.branch_memo.insert(branch.to_string(), idx);
        });
        clone.local_branch().iter().for_each(|local_branch| {
            self.local_branch_memo.insert(local_branch.to_string(), idx);
        });
        clone.remote_branch().iter().for_each(|(o, b)| {
            self.remote_branch_memo
                .insert((o.to_string(), b.to_string()), idx);
        });
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

impl<N, E, Ty, Ix, T> HasHead<N, E, Ty, Ix> for BranchMemo<T, Ix>
where
    T: HasHead<N, E, Ty, Ix>,
{
    fn head(&self) -> Option<&N> {
        self.inner.head()
    }

    fn head_idx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.head_idx()
    }
}

impl<N, E, Ty, Ix, T> HasTail<N, E, Ty, Ix> for BranchMemo<T, Ix>
where
    T: HasTail<N, E, Ty, Ix>,
{
    fn tail(&self) -> Option<&N> {
        self.inner.tail()
    }

    fn tail_idx(&self) -> Option<NodeIndex<Ix>> {
        self.inner.tail_idx()
    }
}

impl<N, E, Ty, Ix, T> HasParentsAndChildren<N, E, Ty, Ix> for BranchMemo<T, Ix>
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

impl<N, E, Ty, Ix, T> CommitExt<N, E, Ty, Ix> for BranchMemo<T, Ix>
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
