use eyre::{OptionExt, Result, eyre};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Bfs, DfsPostOrder, Reversed, Walker};
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::{debug, info, instrument, warn};

use crate::logs::{Decoration, LogEntry, Logs, Tag};
use crate::parser;

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

impl<'a> CommitGraph<'a> {
    #[instrument]
    pub fn peek(&'a self, message: &'a str) -> Result<CommitGraph<'a>> {
        let first_line = message.split('\n').next().unwrap();
        let subject = parser::parse_subject(first_line)?;
        let head = self.head();

        let mut petgraph: petgraph::Graph<&LogEntry<'a>, ()> = self.petgraph.clone();
        let mut commit_to_index: HashMap<&str, NodeIndex> = self.commit_to_index.clone();
        // Intentionally leaking this reference as this code path is deterministic and will end affter the peak
        let next_index = petgraph.add_node(Box::<LogEntry<'_>>::leak(Box::new(LogEntry {
            name: "peek-next-commit",
            branch: head.branch,
            commit_hash: "0000000000000000000000000000000000000000",
            commit_timezone: chrono::Utc,
            commit_datetime: chrono::Utc::now(),
            parent_hashes: vec![head.commit_hash].into(),
            decorations: head.decorations.clone(),
            subject,
            footers: head.footers.clone(),
        })));
        petgraph.add_edge(next_index, self.head_index, ());
        commit_to_index.insert("0000000000000000000000000000000000000000", next_index);

        let result = CommitGraph {
            petgraph,
            head_index: next_index,
            tail_index: self.tail_index,
            commit_to_index,
        };

        Ok(result)
    }
}

impl CommitGraph<'_> {
    #[instrument(skip(logs))]
    pub fn new<'a, 'b>(logs: &'a Logs<'b>) -> Result<CommitGraph<'b>>
    where
        'a: 'b,
    {
        let start = std::time::Instant::now();
        let log_count = logs.iter().count();
        info!(log_count = log_count, "Building commit graph from logs");
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

        let graph = CommitGraph {
            petgraph,
            head_index,
            tail_index,
            commit_to_index,
        };

        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            node_count = graph.node_count(),
            edge_count = graph.edge_count(),
            "Commit graph built successfully"
        );
        debug!(
            "Commit graph built with {} nodes and {} edges in {:?}",
            graph.node_count(),
            graph.edge_count(),
            duration
        );
        Ok(graph)
    }

    pub fn get(&'_ self, idx: NodeIndex) -> Option<&'_ LogEntry<'_>> {
        Some(self.petgraph[idx])
    }

    pub fn node_count(&self) -> usize {
        self.petgraph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.petgraph.edge_count()
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

    pub fn head(&'_ self) -> &'_ LogEntry<'_> {
        self.petgraph[self.head_index]
    }

    pub fn headidx(&self) -> NodeIndex {
        self.head_index
    }

    pub fn tail(&'_ self) -> &'_ LogEntry<'_> {
        self.petgraph[self.tail_index]
    }

    pub fn tailidx(&self) -> NodeIndex {
        self.tail_index
    }

    pub fn tag(&'_ self, tag: &str) -> Result<&'_ LogEntry<'_>> {
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

    pub fn branch(&'_ self, branch: &str) -> Result<&'_ LogEntry<'_>> {
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

    pub fn append_commit_to_head(&mut self, commit: &'static LogEntry<'static>) -> Result<()> {
        let new_head_index = self.petgraph.add_node(commit);
        let old_head_index = self.head_index;
        // set the head index to the new commit
        self.head_index = new_head_index;

        // set an edge from the old head to the new head
        self.petgraph.add_edge(old_head_index, new_head_index, ());

        Ok(())
    }

    pub fn remote(&'_ self, remote: &str, branch: &str) -> Result<&'_ LogEntry<'_>> {
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

    pub fn iter(&'_ self) -> impl Iterator<Item = (NodeIndex, &'_ LogEntry<'_>)> {
        self.petgraph
            .node_indices()
            .map(|idx| (idx, self.petgraph[idx]))
    }

    #[instrument(skip(self))]
    pub fn dfs_postorder_history(&'_ self) -> impl Iterator<Item = (NodeIndex, &'_ LogEntry<'_>)> {
        let start = self.headidx();
        DfsPostOrder::new(&self.petgraph, start)
            .iter(&self.petgraph)
            .map(|idx| (idx, self.petgraph[idx]))
    }

    pub fn bfs_history(&'_ self) -> impl Iterator<Item = (NodeIndex, &'_ LogEntry<'_>)> {
        let start = self.headidx();
        let graph = &self.petgraph;
        Bfs::new(graph, start)
            .iter(graph)
            .map(|idx| (idx, self.petgraph[idx]))
    }

    pub fn history_windowed_childeren(
        &'_ self,
    ) -> impl Iterator<Item = (&'_ LogEntry<'_>, Vec<&'_ LogEntry<'_>>)> {
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

    pub fn history_windowed_parents(
        &'_ self,
    ) -> impl Iterator<Item = (&'_ LogEntry<'_>, Vec<&'_ LogEntry<'_>>)> {
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
