use chrono::naive;
use eyre::{OptionExt, Result, eyre};

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Bfs, DfsPostOrder, Reversed, Walker};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tracing::{instrument, warn};
pub mod assign_edges;
mod assign_versions;
pub mod branch_heads;
pub mod commit_map;
pub mod dfs_postorder_history;
pub mod parents_and_children;
pub mod tail;
pub mod version;
use crate::graph::branch_heads::AsBranchHeadCommits as _;
use crate::graph::commit_map::AsCommitMap as _;
use crate::logs::{Decoration, LogEntry, Logs, PeekLogEntry as _, Tag};
use crate::version::Version;
use crate::version_format::VersionFormat;

pub type CommitGraphT<'a> = DiGraph<CommitGraphNodeWeight<'a>, ()>;

#[derive(Debug)]
pub struct CommitGraphNodeData<'a> {
    pub log_entry: LogEntry<'a>,
    pub version: Option<Version>,
}

impl<'a> From<LogEntry<'a>> for CommitGraphNodeData<'a> {
    fn from(log_entry: LogEntry<'a>) -> Self {
        CommitGraphNodeData {
            log_entry,
            version: None,
        }
    }
}

pub type CommitGraphNodeWeight<'a> = Arc<Mutex<CommitGraphNodeData<'a>>>;

#[derive(Debug)]
pub struct CommitGraph<'a> {
    petgraph: CommitGraphT<'a>,
    head_index: NodeIndex,
    current_branch: &'a str,
    branch_heads: HashMap<&'a str, &'a str>,
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
    pub fn new(logs: &'a Logs<'a>, version_format: &VersionFormat) -> Result<Self> {
        let mut petgraph = CommitGraphT::new();
        let commit_to_index = logs.as_commit_map(&mut petgraph);
        let branch_heads = logs.as_branch_heads();
        let current_head = logs.as_current_head();
        let tail_index = tail::find_tail(&petgraph)?;

        assign_edges::assign_edges(&mut petgraph, &commit_to_index);

        if let Some(current_head) = current_head {
            let head_index = branch_heads[current_head];
            let head_index = commit_to_index
                .get(head_index)
                .cloned()
                .ok_or_eyre("No head index found in commit to index")?;
            let head_branch = branch_heads
                .keys()
                .find_map(|b| if *b == current_head { Some(*b) } else { None })
                .expect("No head branch found in logs");

            assign_versions::assign_versions(&mut petgraph, version_format, head_index)?;

            Ok(CommitGraph {
                petgraph,
                commit_to_index,
                branch_heads,
                current_branch: head_branch,
                head_index,
                tail_index,
            })
        } else {
            Err(eyre!("No current head found in logs"))
        }
    }
}

impl<'a> CommitGraph<'a> {
    #[instrument]
    pub fn peek<'b>(
        &'a self,
        message: &'b str,
        version_format: &VersionFormat,
    ) -> Result<CommitGraph<'a>>
    where
        'b: 'a,
    {
        let head = self.head().clone();
        dbg!(&head);
        let next_log_entry = message.as_peek_log_entry(head.clone());
        dbg!(&next_log_entry);
        let head = head.lock().unwrap();
        let head_version = head
            .version
            .clone()
            .unwrap_or(version_format.as_default_version(&head.log_entry));
        dbg!(&head_version);

        // Clone the existing graph structure
        let mut petgraph = self.petgraph.clone();
        let mut commit_to_index = self.commit_to_index.clone();

        // Create the peek log entry - this will have lifetime 'b but we need 'a
        // We'll need to ensure the message data is copied/owned somehow

        let next_node_data = CommitGraphNodeData {
            log_entry: next_log_entry.clone(),
            version: Some(head_version.next_version(&next_log_entry, version_format)),
        };
        dbg!(&next_node_data);

        let next_index = petgraph.add_node(Arc::new(Mutex::new(next_node_data)));
        dbg!(&next_index);

        let mut next_branch_heads = self.branch_heads.clone();

        next_branch_heads.insert(self.current_branch, next_log_entry.commit_hash);

        petgraph.add_edge(next_index, self.head_index, ());
        commit_to_index.insert(next_log_entry.commit_hash, next_index);

        let result = CommitGraph {
            petgraph,
            branch_heads: next_branch_heads,
            current_branch: self.current_branch,
            head_index: next_index,
            tail_index: self.tail_index,
            commit_to_index,
        };

        Ok(result)
    }

    pub fn get(&self, idx: NodeIndex) -> Option<CommitGraphNodeWeight<'a>> {
        Some(self.petgraph[idx].clone())
    }

    pub fn commit(&self, index: &str) -> Result<CommitGraphNodeWeight<'a>> {
        Ok(self.petgraph[self.commitidx(index)?].clone())
    }

    pub fn head(&'_ self) -> CommitGraphNodeWeight<'a> {
        self.petgraph[self.head_index].clone()
    }

    pub fn tail(&'_ self) -> CommitGraphNodeWeight<'a> {
        self.petgraph[self.tail_index].clone()
    }

    pub fn tag(&'_ self, tag: &str) -> Result<CommitGraphNodeWeight<'a>> {
        Ok(self.petgraph[self.tagidx(tag)?].clone())
    }

    pub fn branch(&'_ self, branch: &str) -> Result<CommitGraphNodeWeight<'a>> {
        Ok(self.petgraph[self.branchidx(branch)?].clone())
    }

    pub fn remote(&self, remote: &str, branch: &str) -> Result<CommitGraphNodeWeight<'a>> {
        Ok(self.petgraph[self.remoteidx(remote, branch)?].clone())
    }

    pub fn iter_from(
        &'a self,
        location: Locations,
        direction: Directions,
    ) -> Result<Box<dyn Iterator<Item = (NodeIndex, CommitGraphNodeWeight<'a>)> + 'a>> {
        let start = location.to_idx(self)?;

        match direction {
            Directions::Backward => Ok(Box::new(
                Bfs::new(&self.petgraph, start)
                    .iter(&self.petgraph)
                    .map(|idx| (idx, self.petgraph[idx].clone())),
            )),

            Directions::Forward => {
                let graph = Reversed(&self.petgraph);
                Ok(Box::new(
                    Bfs::new(graph, start)
                        .iter(graph)
                        .map(|idx| (idx, self.petgraph[idx].clone())),
                ))
            }
        }
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = (NodeIndex, CommitGraphNodeWeight<'a>)> {
        self.petgraph
            .node_indices()
            .map(|idx| (idx, self.petgraph[idx].clone()))
    }

    #[instrument(skip(self))]
    pub fn dfs_postorder_history(
        &'_ self,
    ) -> impl Iterator<Item = (NodeIndex, CommitGraphNodeWeight<'a>)> {
        let start = self.headidx();
        DfsPostOrder::new(&self.petgraph, start)
            .iter(&self.petgraph)
            .map(|idx| (idx, self.petgraph[idx].clone()))
    }

    pub fn bfs_history(&'_ self) -> impl Iterator<Item = (NodeIndex, CommitGraphNodeWeight<'a>)> {
        let start = self.headidx();
        let graph = &self.petgraph;
        Bfs::new(graph, start)
            .iter(graph)
            .map(|idx| (idx, self.petgraph[idx].clone()))
    }

    pub fn history_windowed_childeren(
        &'_ self,
    ) -> impl Iterator<Item = (CommitGraphNodeWeight<'a>, Vec<CommitGraphNodeWeight<'a>>)> {
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
            let parent = self.petgraph[idx].clone();
            let children = children
                .into_iter()
                .map(|idx| self.petgraph[idx].clone())
                .collect();
            (parent, children)
        })
    }

    pub fn history_windowed_parents(
        &'_ self,
    ) -> impl Iterator<Item = (CommitGraphNodeWeight<'a>, Vec<CommitGraphNodeWeight<'a>>)> {
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
            let child = self.petgraph[idx].clone();
            let parents = parents
                .into_iter()
                .map(|idx| self.petgraph[idx].clone())
                .collect();
            (child, parents)
        })
    }
}

impl CommitGraph<'_> {
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

    pub fn commitidx(&self, index: &str) -> Result<NodeIndex> {
        self.commit_to_index
            .get(index)
            .copied()
            .ok_or_eyre("could not find commit in commit map")
    }

    pub fn headidx(&self) -> NodeIndex {
        self.head_index
    }

    pub fn tailidx(&self) -> NodeIndex {
        self.tail_index
    }

    fn tagidx(&self, tag: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx]
                .lock()
                .unwrap()
                .log_entry
                .decorations
                .iter()
            {
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

    fn branchidx(&self, branch: &str) -> Result<NodeIndex> {
        match self.branch_heads.get(branch) {
            Some(idx) => Ok(self.commitidx(idx)?),
            None => Err(eyre!("branch not found in history")),
        }
    }

    pub fn append_commit_to_head(&mut self, commit: LogEntry<'static>) -> Result<()> {
        let new_head_index = self
            .petgraph
            .add_node(Arc::new(Mutex::new(CommitGraphNodeData::from(commit))));
        let old_head_index = self.head_index;
        // set the head index to the new commit
        self.head_index = new_head_index;

        // set an edge from the old head to the new head
        self.petgraph.add_edge(old_head_index, new_head_index, ());

        Ok(())
    }

    fn remoteidx(&self, remote: &str, branch: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx]
                .lock()
                .unwrap()
                .log_entry
                .decorations
                .iter()
            {
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
}

#[cfg(test)]
mod graph_tests {

    use crate::{graph::CommitGraph, logs::Logs, version_format::VersionFormat};
    use eyre::*;

    use super::{Directions, Locations};

    #[test]
    fn test_iter_from() -> Result<()> {
        let logs = Logs::default();
        let graph = CommitGraph::new(&logs, &VersionFormat::default())?;

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
        let graph = CommitGraph::new(&logs, &VersionFormat::default())?;
        let logs: Vec<String> = graph
            .dfs_postorder_history()
            .map(|(_, n)| n.lock().unwrap().log_entry.commit_hash.to_string())
            .collect();
        assert_ne!(logs.len(), 0);

        let logs2: Vec<String> = graph
            .bfs_history()
            .map(|(_, n)| n.lock().unwrap().log_entry.commit_hash.to_string())
            .collect();
        assert_eq!(logs.len(), logs2.len());

        Ok(())
    }
}
