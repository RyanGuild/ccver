use eyre::{eyre, OptionExt, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Bfs, DfsPostOrder, Reversed, Walker};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::logs::{Decoration, LogEntry, Logs, Tag};

pub type PetGraph<'a> = DiGraph<&'a LogEntry<'a>, ()>;

#[derive(Debug)]
pub struct CommitGraph<'a> {
    petgraph: PetGraph<'a>,
    head_index: NodeIndex,
    tail_index: NodeIndex,
    commit_to_index: HashMap<&'a str, NodeIndex>,
}

pub enum Directions {
    Backward,
    Forward,
}

#[derive(Debug)]
pub enum Locations<'a> {
    Head,
    Initial,
    Sha(&'a str),
    Decoration(Decoration<'a>),
}

impl Locations<'_> {
    fn to_idx(&self, graph: &CommitGraph) -> Result<NodeIndex> {
        match self {
            Locations::Head => Ok(graph.headidx()),
            Locations::Initial => Ok(graph.tailidx()),
            Locations::Sha(sha) => graph.commitidx(sha),
            Locations::Decoration(d) => match d {
                Decoration::HeadIndicator(_) => Ok(graph.headidx()),
                Decoration::Tag(t) => graph.tagidx(
                    match t {
                        Tag::Text(t) => t.to_string(),
                        Tag::Version(v) => format!("{}", v),
                    }
                    .as_str(),
                ),
                Decoration::RemoteBranch((o, b)) => graph.remoteidx(o, b),
                Decoration::Branch(b) => graph.branchidx(b),
            },
        }
    }
}

impl CommitGraph<'_> {
    pub fn new<'a, 'b>(logs: &'a Logs<'b>) -> Result<CommitGraph<'b>>
    where
        'a: 'b,
    {
        let mut petgraph: petgraph::Graph<&'b LogEntry<'b>, ()> = DiGraph::new();
        let commit_to_index: HashMap<&str, NodeIndex> = logs
            .iter()
            .map(|l| (l.commit_hash, petgraph.add_node(l)))
            .collect();

        let indexes = commit_to_index.values();

        let edges: Vec<(NodeIndex, NodeIndex)> = indexes
            .flat_map(|i| {
                let ref1 = petgraph[*i];
                let parents = ref1.parent_hashes.clone();
                let parents_iter = parents.iter();
                let parent_index_iter = parents_iter.map(|p| commit_to_index[*p]);
                let current_index_iter = std::iter::repeat_n(*i, parent_index_iter.len());
                current_index_iter
                    .zip(parent_index_iter)
                    .collect::<Vec<(NodeIndex, NodeIndex)>>()
            })
            .collect();

        for (child, parent) in edges {
            petgraph.add_edge(child, parent, ());
        }

        let head_index = petgraph
            .node_indices()
            .filter(|i| {
                let node = petgraph[*i];
                for dec in node.decorations.iter() {
                    match dec {
                        Decoration::HeadIndicator(_) => return true,
                        _ => continue,
                    };
                }
                false
            })
            .next()
            .ok_or_eyre("could not find HEAD in graph")?;

        let tail_index = petgraph
            .node_indices()
            .filter(|i| {
                let node = petgraph[*i];
                node.parent_hashes.is_empty()
            })
            .next()
            .ok_or_eyre("could not find initial commit circular history or no commits detected")?;

        Ok(CommitGraph {
            petgraph,
            head_index,
            tail_index,
            commit_to_index,
        })
    }

    pub fn get(&self, idx: NodeIndex) -> Option<&LogEntry> {
        Some(self.petgraph[idx])
    }

    pub fn parents(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.petgraph
            .neighbors_directed(idx, petgraph::Direction::Outgoing)
            .collect()
    }

    pub fn children(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.petgraph
            .neighbors_directed(idx, petgraph::Direction::Incoming)
            .collect()
    }

    pub fn commit(&self, index: &str) -> Result<&LogEntry<'_>> {
        Ok(self.petgraph[self.commitidx(index)?])
    }

    pub fn commitidx(&self, index: &str) -> Result<NodeIndex> {
        self.commit_to_index
            .get(index)
            .copied()
            .ok_or_eyre("could not find commit in commit map")
    }

    pub fn head(&self) -> &LogEntry {
        self.petgraph[self.head_index]
    }

    pub fn headidx(&self) -> NodeIndex {
        self.head_index
    }

    pub fn tail(&self) -> &LogEntry {
        self.petgraph[self.tail_index]
    }

    pub fn tailidx(&self) -> NodeIndex {
        self.tail_index
    }

    pub fn tag(&self, tag: &str) -> Result<&LogEntry> {
        Ok(self.petgraph[self.tagidx(tag)?])
    }

    fn tagidx(&self, tag: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx].decorations.iter() {
                match dec {
                    Decoration::Tag(t) => {
                        if t == &Tag::Text(tag) {
                            return Ok(idx);
                        } else {
                            continue;
                        };
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }
        Err(eyre!("tag not found in history"))
    }

    pub fn branch(&self, branch: &str) -> Result<&LogEntry> {
        Ok(self.petgraph[self.branchidx(branch)?])
    }

    fn branchidx(&self, branch: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx].decorations.iter() {
                match dec {
                    Decoration::Branch(b) => {
                        if *b == branch {
                            return Ok(idx);
                        } else {
                            continue;
                        };
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }
        Err(eyre!("branch not found in history"))
    }

    pub fn remote(&self, remote: &str, branch: &str) -> Result<&LogEntry> {
        Ok(self.petgraph[self.remoteidx(remote, branch)?])
    }

    fn remoteidx(&self, remote: &str, branch: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx].decorations.iter() {
                match dec {
                    Decoration::RemoteBranch((o, b)) => {
                        if *o == remote && *b == branch {
                            return Ok(idx);
                        } else {
                            continue;
                        };
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }
        Err(eyre!("remote not found in history"))
    }

    pub fn iter_from<'a>(
        &'a self,
        location: Locations,
        direction: Directions,
    ) -> Result<Box<dyn Iterator<Item = (NodeIndex, &'a LogEntry<'a>)> + 'a>> {
        let start = location.to_idx(self)?;

        match direction {
            Directions::Backward => Ok(Box::new(
                Bfs::new(&self.petgraph, start)
                    .iter(&self.petgraph)
                    .map(|idx| (idx, self.petgraph[idx])),
            )),

            Directions::Forward => {
                let graph = Reversed(&self.petgraph);
                Ok(Box::new(
                    Bfs::new(graph, start)
                        .iter(graph)
                        .map(|idx| (idx, self.petgraph[idx])),
                ))
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeIndex, &LogEntry)> {
        self.petgraph
            .node_indices()
            .map(|idx| (idx, self.petgraph[idx]))
    }

    pub fn dfs_postorder_history(&self) -> impl Iterator<Item = (NodeIndex, &LogEntry)> {
        let start = self.headidx();
        DfsPostOrder::new(&self.petgraph, start)
            .iter(&self.petgraph)
            .map(|idx| (idx, self.petgraph[idx]))
    }

    pub fn bfs_history(&self) -> impl Iterator<Item = (NodeIndex, &LogEntry)> {
        let start = self.headidx();
        let graph = &self.petgraph;
        Bfs::new(graph, start)
            .iter(graph)
            .map(|idx| (idx, self.petgraph[idx]))
    }

    pub fn history_windowed_childeren(&self) -> impl Iterator<Item = (&LogEntry, Vec<&LogEntry>)> {
        let history = self.dfs_postorder_history();
        let mut windows: Vec<(NodeIndex, Vec<NodeIndex>)> = vec![];

        for (idx, _) in history {
            let children: Vec<NodeIndex> = self
                .petgraph
                // note that children point to parents in the DiGraph so we need to reverse the direction
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .collect();
            windows.push((idx, children));
        }

        windows.into_iter().map(|(idx, children)| {
            let parent = self.petgraph[idx];
            let children = children.into_iter().map(|idx| self.petgraph[idx]).collect();
            (parent, children)
        })
    }

    pub fn history_windowed_parents(&self) -> impl Iterator<Item = (&LogEntry, Vec<&LogEntry>)> {
        let history = self.dfs_postorder_history();
        let mut windows: Vec<(NodeIndex, Vec<NodeIndex>)> = vec![];

        for (idx, _) in history {
            let parents: Vec<NodeIndex> = self
                .petgraph
                .neighbors_directed(idx, petgraph::Direction::Outgoing)
                .collect();
            windows.push((idx, parents));
        }

        windows.into_iter().map(|(idx, parents)| {
            let child = self.petgraph[idx];
            let parents = parents.into_iter().map(|idx| self.petgraph[idx]).collect();
            (child, parents)
        })
    }

    pub fn all_parents(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        let mut parents = vec![];
        let mut stack = vec![idx];
        while let Some(node) = stack.pop() {
            let node_parents = self.parents(node);
            stack.extend(node_parents.clone());
            parents.push(node);
            parents.extend(node_parents);
        }
        parents
    }
}

#[cfg(test)]
mod graph_tests {

    use crate::{graph::CommitGraph, logs::Logs};
    use eyre::*;

    use super::{Directions, Locations};

    #[test]
    fn test_iter_from() -> Result<()> {
        let logs = Logs::default();
        let graph = CommitGraph::new(&logs)?;

        // Second commit
        let second_commit = "40f8bef8e7c290ebe0e52b91fa84fee30b4a162d";

        let iter1: Vec<_> = graph
            .iter_from(Locations::Sha(second_commit), Directions::Backward)?
            .collect();

        assert_eq!(iter1.len(), 2);

        let iter2: Vec<_> = graph
            .iter_from(Locations::Sha(second_commit), Directions::Forward)?
            .collect();

        assert_ne!(iter2.len(), 2);

        // -1 on account of the second_commit appears in both iters
        assert_eq!(
            iter1.len() + iter2.len() - 1,
            graph.dfs_postorder_history().count()
        );
        assert_eq!(iter1.len() + iter2.len() - 1, graph.bfs_history().count());

        Ok(())
    }

    #[test]
    fn test_graph_walk() -> Result<()> {
        let logs = Logs::default();
        let graph = CommitGraph::new(&logs)?;
        let logs: Vec<String> = graph
            .dfs_postorder_history()
            .map(|(_, n)| n.commit_hash.to_string())
            .collect();
        assert_ne!(logs.len(), 0);

        let logs2: Vec<String> = graph
            .bfs_history()
            .map(|(_, n)| n.commit_hash.to_string())
            .collect();
        assert_eq!(logs.len(), logs2.len());

        Ok(())
    }
}
