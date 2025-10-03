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
use tracing::{Level, debug, instrument, span};
use version::Version;
use version_format::VersionFormat;

use crate::{
    graph::{MemoizedCommitGraph, version::ExistingVersionExt as _},
    logs::PeekLogEntry as _,
};

#[instrument]
pub fn peek(
    repo_path: &Path,
    commit_message: String,
    version_format: &VersionFormat,
) -> Result<Version, eyre::Error> {
    let logs = Logs::from_path(repo_path)?;
    let graph = MemoizedCommitGraph::new(logs, version_format);

    let parent_commit = graph.head().unwrap().lock().unwrap().log_entry.commit_hash;
    let branch = graph.head().unwrap().lock().unwrap().log_entry.branch;
    let next_entry = commit_message
        .leak()
        .into_peek_log_entry(parent_commit, branch);
    let next_version = graph
        .head()
        .unwrap()
        .as_existing_version()
        .map(|v| v.next_version(&next_entry, &version_format))
        .unwrap_or_else(|| version_format.as_default_version(&next_entry));

    debug!(version = %next_version, "Peek result");
    if version_format.prerelease.is_none() {
        Ok(next_version.no_pre())
    } else {
        Ok(next_version)
    }
}
