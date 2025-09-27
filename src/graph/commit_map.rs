use eyre::{OptionExt, Result};
use petgraph::graph::NodeIndex;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    graph::{CommitGraphNodeData, CommitGraphT},
    logs::Logs,
};

pub type CommitMap<'a> = HashMap<&'a str, NodeIndex>;

pub trait CallWithCommitHashMappedToNodeIndex {
    fn call_with_commit<T>(&self, commit_hash: &str, f: impl FnOnce(NodeIndex) -> T) -> Result<T>;
}

impl CallWithCommitHashMappedToNodeIndex for HashMap<&str, NodeIndex> {
    fn call_with_commit<T>(&self, commit_hash: &str, f: impl FnOnce(NodeIndex) -> T) -> Result<T> {
        self.get(commit_hash)
            .copied()
            .map(f)
            .ok_or_eyre("commit hash not found in commit map")
    }
}

pub trait AsCommitMap<'a> {
    fn as_commit_map(&'_ self, petgraph: &mut CommitGraphT<'a>) -> CommitMap<'a>;
}

impl<'a, 'b> AsCommitMap<'a> for &'b Logs<'a>
where
    'b: 'a,
{
    fn as_commit_map(&'_ self, petgraph: &mut CommitGraphT<'a>) -> CommitMap<'a> {
        self.iter()
            .map(|l| {
                (
                    l.commit_hash,
                    petgraph.add_node(Arc::new(Mutex::new(CommitGraphNodeData::from(l.clone())))),
                )
            })
            .collect()
    }
}
