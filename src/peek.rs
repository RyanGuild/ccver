use crate::graph::version::ExistingVersionExt as _;
use crate::logs::PeekLogEntry;
use crate::{args::CCVerArgs, graph::MemoizedCommitGraph, version_format::VersionFormat};
use eyre::Result;
use tracing::Level;
use tracing::{debug, span};

pub fn peek_command_handler<'a>(
    graph: &MemoizedCommitGraph<'a>,
    parsed_args: &CCVerArgs,
    version_format: &VersionFormat,
) -> Result<String> {
    let message = parsed_args.message.as_ref().unwrap();
    let _peek_span = span!(Level::INFO, "peek_command", message = %message).entered();
    let head_guard = graph.head().unwrap().lock().unwrap();
    let parent_commit = head_guard.log_entry.commit_hash;
    let branch = head_guard.log_entry.branch;
    let next_entry = message.as_str();
    let next_entry = next_entry.as_peek_log_entry(parent_commit, branch);
    debug!(commit_hash = %next_entry.commit_hash, branch = %next_entry.branch, "Next entry");
    let next_version = head_guard
        .as_existing_version()
        .map(|v| v.next_version(&next_entry, &version_format))
        .unwrap_or_else(|| version_format.as_default_version(&next_entry));

    debug!(next_version = %next_version);
    if parsed_args.no_pre {
        Ok(format!("{}", next_version.no_pre()))
    } else {
        Ok(format!("{}", next_version))
    }
}
