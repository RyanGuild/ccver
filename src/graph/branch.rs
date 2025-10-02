use std::collections::HashMap;

use petgraph::{
    Direction, Graph,
    graph::{EdgeIndex, NodeIndex},
};

use crate::{
    graph::GraphOps,
    logs::{Decoration, LogEntry},
};

pub trait HasBranchs {
    fn branch(&self) -> Vec<&str>;
}

impl HasBranchs for LogEntry<'_> {
    fn branch(&self) -> Vec<&str> {
        self.decorations
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

pub trait HasLocalBranchs {
    fn local_branch(&self) -> Vec<&str>;
}

impl HasLocalBranchs for LogEntry<'_> {
    fn local_branch(&self) -> Vec<&str> {
        self.decorations
            .iter()
            .filter_map(|d| match d {
                Decoration::Branch(b) => Some(*b),
                _ => None,
            })
            .collect()
    }
}

pub trait HasRemoteBranchs {
    fn remote_branch(&self) -> Vec<(&str, &str)>;
}

impl HasRemoteBranchs for LogEntry<'_> {
    fn remote_branch(&self) -> Vec<(&str, &str)> {
        self.decorations
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
    fn branchidx(&self, branch: &str) -> Vec<NodeIndex<Ix>>;

    fn local_branch(&self, branch: &str) -> Vec<N>;
    fn local_branchidx(&self, branch: &str) -> Vec<NodeIndex<Ix>>;

    fn remote_branch(&self, remote: &str, branch: &str) -> Vec<N>;
    fn remote_branchidx(&self, remote: &str, branch: &str) -> Vec<NodeIndex<Ix>>;
}
struct BranchMemo<T, Ix> {
    branchmemo: HashMap<String, NodeIndex<Ix>>,
    inner: T,
    local_branchmemo: HashMap<String, NodeIndex<Ix>>,
    remote_branchmemo: HashMap<(String, String), NodeIndex<Ix>>,
}

impl<T, Ix> BranchMemo<T, Ix> {
    pub fn new<N, E, Ty>(graph: T) -> Self
    where
        T: GraphOps<N, E, Ty, Ix>,
        N: HasBranchs + HasLocalBranchs + HasRemoteBranchs,
    {
        Self {
            branchmemo: HashMap::new(),
            local_branchmemo: HashMap::new(),
            remote_branchmemo: HashMap::new(),
            inner: graph,
        }
    }
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for BranchMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasBranchs + HasLocalBranchs + HasRemoteBranchs + Clone,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let clone = weight.clone();
        let idx = self.inner.add_node(weight);

        clone
            .branch()
            .iter()
            .map(|branch| self.branchmemo.insert(branch.to_string(), idx));
        clone
            .local_branch()
            .iter()
            .map(|local_branch| self.local_branchmemo.insert(local_branch.to_string(), idx));
        clone.remote_branch().iter().map(|(o, b)| {
            self.remote_branchmemo
                .insert((o.to_string(), b.to_string()), idx)
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
