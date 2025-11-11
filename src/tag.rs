use crate::git::is_dirty;
use crate::graph::version::{ExistingVersionExt as _, TaggedVersionExt as _};
use crate::{args::CCVerArgs, graph::MemoizedCommitGraph, version_format::VersionFormat};
use eyre::{Result, eyre};
use petgraph::visit::{DfsPostOrder, Walker as _};
use tracing::Level;
use tracing::info;
use tracing::span;

pub fn tag_command_handler(
    graph: &MemoizedCommitGraph,
    parsed_args: &CCVerArgs,
    version_format: &VersionFormat,
) -> Result<String> {
    if is_dirty(&parsed_args.path)? {
        return Err(eyre!("Repo is dirty while tag is true"));
    }
    let _tag_span = span!(Level::INFO, "tag_command", all = parsed_args.tag_all).entered();
    info!("Tagging with all: {}", parsed_args.tag_all);
    let version =
        crate::current_version::current_version_handler(&graph, &parsed_args, &version_format)?;
    if !parsed_args.tag_all {
        let head_guard = graph.head().unwrap().lock().unwrap();
        crate::git::tag_commit_with_version(
            head_guard.log_entry.commit_hash,
            &version,
            &parsed_args.path,
        )?;
        Ok(version.to_string())
    } else {
        let new_versions = DfsPostOrder::new(graph.base_graph(), graph.head_idx().unwrap())
            .iter(graph.base_graph())
            .map(|idx| {
                let weight = graph.node_weight(idx).unwrap().lock().unwrap();
                let version = weight
                    .as_existing_version()
                    .expect("A version was not assigned to a node in the graph");

                let tagged_version = weight.log_entry.as_tagged_version();
                if tagged_version.is_none() {
                    let _ = crate::git::tag_commit_with_version(
                        weight.log_entry.commit_hash,
                        &version,
                        &parsed_args.path,
                    );
                }

                Ok(format!("{}", version))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(new_versions.join("\n"))
    }
}
