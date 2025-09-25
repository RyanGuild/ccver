use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use tracing::{debug, info, info_span, instrument, warn};

use crate::{
    graph::CommitGraph,
    logs::Subject,
    pattern_macros::{
        alpha_branches, beta_branches, major_subject, minor_subject, patch_subject, rc_branches,
        release_branches,
    },
    version::Version,
    version_format::VersionFormat,
};
use eyre::Result;

#[derive(Debug)]
pub struct VersionMap(HashMap<NodeIndex, Version>);

impl VersionMap {
    #[instrument(skip(graph), name = "VersionMap::new")]
    pub fn new(graph: &CommitGraph, version_format: &VersionFormat) -> Result<VersionMap> {
        let start = std::time::Instant::now();
        let node_count = graph.node_count();
        info!(node_count = node_count, "Creating new version map");
        let mut map: HashMap<NodeIndex, Version> = HashMap::new();

        let tailidx = graph.tailidx();
        let tail = graph.tail();
        let initial_version = version_format.as_default_version(tail);
        debug!("Initial version: {}", initial_version);
        map.insert(tailidx, initial_version);

        let graph_len = graph.node_count();
        let mut seen = std::collections::HashSet::new();

        let results = graph
            .dfs_postorder_history()
            .map(|(idx, commit)| {
                let _debug_span = info_span!("dfs_postorder_history", idx = ?idx).entered();
                info!("Processing commit: {:?}", commit);
                seen.insert(idx);
                let tagged = commit.tagged_version();
                if let Some(tagged) = tagged {
                    map.insert(idx, tagged);
                    return Ok(());
                };

                let existing = tagged.or(map.get(&idx).cloned());

                let max_parent = graph
                    .all_parents(idx)
                    .iter()
                    .filter_map(|parent| {
                        map.get(parent).cloned().or(graph
                            .get(idx)
                            .expect("Idx Comes from graph source")
                            .tagged_version())
                    })
                    .max()
                    .unwrap_or(version_format.as_default_version(commit));

                if let Some(existing) = existing {
                    if existing < max_parent {
                        Err(eyre::eyre!(
                            "Existing version is less than its latest parent's version"
                        ))
                    } else {
                        map.insert(idx, existing);
                        Ok(())
                    }
                } else {
                    let next_version = match (
                        &commit.subject,
                        commit.branch,
                        commit.parent_hashes.len() == 2,
                    ) {
                        (major_subject!(), release_branches!(), _) => {
                            max_parent.major(commit, version_format)
                        }
                        (minor_subject!(), release_branches!(), _) => {
                            max_parent.minor(commit, version_format)
                        }
                        (patch_subject!(), release_branches!(), _) => {
                            max_parent.patch(commit, version_format)
                        }
                        (Subject::Conventional(_), release_branches!(), _) => {
                            max_parent.short_sha(commit, version_format)
                        }
                        (major_subject!(), rc_branches!(), _) => max_parent
                            .major(commit, version_format)
                            .rc(commit, version_format),
                        (minor_subject!(), rc_branches!(), _) => max_parent
                            .minor(commit, version_format)
                            .rc(commit, version_format),
                        (patch_subject!(), rc_branches!(), _) => max_parent
                            .patch(commit, version_format)
                            .rc(commit, version_format),
                        (Subject::Conventional(_), rc_branches!(), _) => {
                            max_parent.rc(commit, version_format)
                        }
                        (major_subject!(), beta_branches!(), _) => max_parent
                            .major(commit, version_format)
                            .beta(commit, version_format),
                        (minor_subject!(), beta_branches!(), _) => max_parent
                            .minor(commit, version_format)
                            .beta(commit, version_format),
                        (patch_subject!(), beta_branches!(), _) => max_parent
                            .patch(commit, version_format)
                            .beta(commit, version_format),
                        (Subject::Conventional(_), beta_branches!(), _) => {
                            max_parent.beta(commit, version_format)
                        }
                        (major_subject!(), alpha_branches!(), _) => max_parent
                            .major(commit, version_format)
                            .alpha(commit, version_format),
                        (minor_subject!(), alpha_branches!(), _) => max_parent
                            .minor(commit, version_format)
                            .alpha(commit, version_format),
                        (patch_subject!(), alpha_branches!(), _) => max_parent
                            .patch(commit, version_format)
                            .alpha(commit, version_format),
                        (Subject::Conventional(_), alpha_branches!(), _) => {
                            max_parent.alpha(commit, version_format)
                        }
                        (major_subject!(), _, _) => max_parent
                            .major(commit, version_format)
                            .named(commit, version_format),
                        (minor_subject!(), _, _) => max_parent
                            .minor(commit, version_format)
                            .named(commit, version_format),
                        (patch_subject!(), _, _) => max_parent
                            .patch(commit, version_format)
                            .named(commit, version_format),
                        (Subject::Conventional(_), _, _) => {
                            max_parent.named(commit, version_format)
                        }
                        (Subject::Text(_), release_branches!(), true) => {
                            max_parent.release(commit, version_format)
                        }
                        (Subject::Text(_), release_branches!(), _) => {
                            max_parent.short_sha(commit, version_format)
                        }
                        (Subject::Text(_), rc_branches!(), _) => {
                            max_parent.rc(commit, version_format)
                        }
                        (Subject::Text(_), beta_branches!(), _) => {
                            max_parent.beta(commit, version_format)
                        }
                        (Subject::Text(_), alpha_branches!(), _) => {
                            max_parent.alpha(commit, version_format)
                        }
                        (Subject::Text(_), _, _) => max_parent.named(commit, version_format),
                    };

                    // dbg!(&next_version);

                    map.insert(idx, next_version);
                    Ok(())
                }
            })
            .collect::<Vec<Result<()>>>();

        assert_eq!(
            seen.len(),
            graph_len,
            "Seen nodes should be equal to graph nodes"
        );

        let error_count = results.iter().filter_map(|r| r.as_ref().err()).count();

        if error_count > 0 {
            warn!("Version map creation had {} errors", error_count);
            results
                .iter()
                .filter_map(|r| r.as_ref().err())
                .for_each(|e| warn!("Version map error: {}", e));
        }

        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            map_entries = map.len(),
            "Version map created successfully"
        );
        debug!(
            "Version map created with {} entries in {:?}",
            map.len(),
            duration
        );
        Ok(Self(map))
    }

    #[instrument(skip(self))]
    pub fn get(&self, idx: NodeIndex) -> Option<&Version> {
        let version = self.0.get(&idx);
        debug!(idx = ?idx, version = ?version, "Retrieved version for index");
        version
    }

    #[instrument(skip(self))]
    pub fn set(&mut self, idx: NodeIndex, version: Version) {
        debug!(idx = ?idx, version = %version, "Setting version for index");
        self.0.insert(idx, version);
    }

    #[instrument(skip(self))]
    pub fn get_key(&self, version: &Version) -> Option<NodeIndex> {
        self.0
            .iter()
            .find_map(|(k, v)| if v == version { Some(*k) } else { None })
    }
}
