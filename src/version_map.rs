use std::{collections::HashMap, rc::Rc};

use petgraph::graph::NodeIndex;

use crate::{
    graph::CommitGraph,
    logs::Subject,
    pattern_macros::{
        alpha_branches, beta_branches, major_subject, minor_subject, patch_subject, rc_branches,
        release_branches,
    },
    version::Version,
    version_format::VERSION_FORMAT,
};
use eyre::Result;

pub type VersionMap = Rc<VersionMapData>;

#[derive(Debug)]
pub struct VersionMapData(HashMap<NodeIndex, Version>);

impl VersionMapData {
    pub fn new(graph: CommitGraph) -> Result<VersionMap> {
        let graph = graph.clone();
        let mut map: HashMap<NodeIndex, Version> = HashMap::new();

        let tailidx = graph.tailidx();
        let tail = graph.tail();

        let version_format = VERSION_FORMAT.lock().unwrap().clone();
        let initial_version = version_format.as_default_version(tail.clone());
        map.insert(tailidx, initial_version);

        let results = graph
            .dfs_postorder_history()
            .map(|(idx, commit)| {
                let graph = graph.clone();
                let tagged = commit.tagged_version();
                if tagged.is_some() {
                    map.insert(idx, tagged.unwrap());
                    return Ok(());
                };

                let existing = tagged.or(map.get(&idx).cloned());

                let max_parent = graph
                    .all_parents(idx)
                    .iter()
                    .filter_map(|parent| {
                        let ver = map.get(parent).cloned().or(graph
                            .get(idx)
                            .expect("Idx Comes from graph source")
                            .tagged_version());

                        ver
                    })
                    .max()
                    .unwrap_or(version_format.as_default_version(commit.clone()));

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
                        (major_subject!(), release_branches!(), _) => max_parent.major(commit),
                        (minor_subject!(), release_branches!(), _) => max_parent.minor(commit),
                        (patch_subject!(), release_branches!(), _) => max_parent.patch(commit),
                        (Subject::Conventional(_), release_branches!(), _) => {
                            max_parent.short_sha(commit)
                        }
                        (major_subject!(), rc_branches!(), _) => {
                            max_parent.major(commit.clone()).rc(commit)
                        }
                        (minor_subject!(), rc_branches!(), _) => {
                            max_parent.minor(commit.clone()).rc(commit)
                        }
                        (patch_subject!(), rc_branches!(), _) => {
                            max_parent.patch(commit.clone()).rc(commit)
                        }
                        (Subject::Conventional(_), rc_branches!(), _) => max_parent.rc(commit),
                        (major_subject!(), beta_branches!(), _) => {
                            max_parent.major(commit.clone()).beta(commit)
                        }
                        (minor_subject!(), beta_branches!(), _) => {
                            max_parent.minor(commit.clone()).beta(commit)
                        }
                        (patch_subject!(), beta_branches!(), _) => {
                            max_parent.patch(commit.clone()).beta(commit)
                        }
                        (Subject::Conventional(_), beta_branches!(), _) => max_parent.beta(commit),
                        (major_subject!(), alpha_branches!(), _) => {
                            max_parent.major(commit.clone()).alpha(commit)
                        }
                        (minor_subject!(), alpha_branches!(), _) => {
                            max_parent.minor(commit.clone()).alpha(commit)
                        }
                        (patch_subject!(), alpha_branches!(), _) => {
                            max_parent.patch(commit.clone()).alpha(commit)
                        }
                        (Subject::Conventional(_), alpha_branches!(), _) => {
                            max_parent.alpha(commit)
                        }
                        (major_subject!(), _, _) => max_parent.major(commit.clone()).named(commit),
                        (minor_subject!(), _, _) => max_parent.minor(commit.clone()).named(commit),
                        (patch_subject!(), _, _) => max_parent.patch(commit.clone()).named(commit),
                        (Subject::Conventional(_), _, _) => max_parent.named(commit),
                        (Subject::Text(_), release_branches!(), true) => max_parent.release(commit),
                        (Subject::Text(_), release_branches!(), _) => max_parent.short_sha(commit),
                        (Subject::Text(_), rc_branches!(), _) => max_parent.rc(commit),
                        (Subject::Text(_), beta_branches!(), _) => max_parent.beta(commit),
                        (Subject::Text(_), alpha_branches!(), _) => max_parent.alpha(commit),
                        (Subject::Text(_), _, _) => max_parent.named(commit),
                    };

                    // dbg!(&next_version);

                    map.insert(idx, next_version);
                    Ok(())
                }
            })
            .collect::<Vec<Result<()>>>();

        results
            .iter()
            .filter_map(|r| r.as_ref().err())
            .for_each(|e| eprintln!("{}", e));

        Ok(Rc::new(Self(map)))
    }

    pub fn get(&self, idx: NodeIndex) -> Option<&Version> {
        self.0.get(&idx)
    }

    pub fn set(&mut self, idx: NodeIndex, version: Version) {
        self.0.insert(idx, version);
    }

    pub fn get_key(&self, version: &Version) -> Option<NodeIndex> {
        self.0
            .iter()
            .find_map(|(k, v)| if v == version { Some(k.clone()) } else { None })
    }
}
