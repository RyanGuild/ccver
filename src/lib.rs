#![feature(decl_macro, lock_value_accessors, iterator_try_collect)]

use std::path::Path;
pub mod args;
pub mod changelog;
pub mod git;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod pattern_macros;
pub mod version;
pub mod version_format;

use eyre::Result;
use logs::Logs;
use tracing::debug;
use version::Version;
use version_format::VersionFormat;

use crate::{
    graph::{MemoizedCommitGraph, version::ExistingVersionExt as _},
    logs::PeekLogEntry as _,
};

pub fn peek(
    repo_path: &Path,
    commit_message: String,
    version_format: &VersionFormat,
) -> Result<(Version, Version), eyre::Error> {
    let logs = Logs::from_path(repo_path)?;
    let graph = MemoizedCommitGraph::new(logs, version_format);

    let parent_commit = graph.head().unwrap().lock().unwrap().log_entry.clone();
    let next_entry = commit_message
        .leak()
        .into_peek_log_entry(parent_commit.commit_hash, parent_commit.branch);
    let last_version = graph
        .head()
        .unwrap()
        .as_existing_version()
        .unwrap_or_else(|| version_format.as_default_version(&parent_commit).clone());
    let next_version = last_version.next_version(&next_entry, &version_format);

    debug!(version = %next_version, "Peek result");

    Ok((last_version, next_version))
}
