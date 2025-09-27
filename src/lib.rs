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

use eyre::{OptionExt, Result, eyre};
use graph::CommitGraph;
use logs::Logs;
use tracing::{Level, span};
use version::Version;
use version_format::VersionFormat;

pub fn peek(
    repo_path: &Path,
    commit_message: &str,
    version_format: &VersionFormat,
) -> Result<Version, eyre::Error> {
    let _span = span!(Level::INFO, "peek", repo_path = ?repo_path, commit_message = %commit_message, version_format = %version_format).entered();
    let logs = Logs::from_path(repo_path)?;
    let graph = CommitGraph::new(&logs, version_format)?;
    let next_graph = graph.peek(commit_message, version_format)?;
    let version = next_graph
        .head()
        .lock()
        .unwrap()
        .version
        .clone()
        .ok_or_eyre(eyre!("Peek Head Was Not Assigned a Version"))?;
    Ok(version)
}
