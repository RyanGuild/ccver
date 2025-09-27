use crate::{
    graph::{
        CommitGraphT, dfs_postorder_history::DfsPostorderHistoryExt as _,
        parents_and_children::ParentChildExt as _,
    },
    version_format::VersionFormat,
};
use eyre::{Ok, Result};
use petgraph::graph::NodeIndex;
use tracing::{info, info_span};

pub fn assign_versions<'a>(
    graph: &mut CommitGraphT<'a>,
    version_format: &VersionFormat,
    headidx: NodeIndex,
) -> Result<()> {
    let history_walk = graph.dfs_postorder_history(headidx);
    history_walk
        .into_iter()
        .map(|(idx, commit)| {
            let _debug_span = info_span!("dfs_postorder_history", idx = ?idx).entered();
            info!("Processing commit: {:?}", commit);

            let binding = version_format.as_default_version(&commit.lock().unwrap().log_entry);
            let max_parent = graph
                .parents(idx)
                .iter()
                .filter_map(|parent| {
                    let parent = &mut graph[*parent].lock().unwrap();
                    parent.version.clone()
                })
                .max()
                .unwrap_or(binding);

            let next_version =
                max_parent.next_version(&commit.lock().unwrap().log_entry, version_format);

            commit.lock().unwrap().version = Some(next_version);

            Ok(())
        })
        .try_collect::<Vec<_>>()?;
    Ok(())
}
