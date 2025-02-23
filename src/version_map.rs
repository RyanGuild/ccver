use std::{collections::HashMap, rc::Rc};

use petgraph::graph::NodeIndex;

use crate::{
    graph::CommitGraph,
    logs::{ConventionalSubject, Subject},
    version::Version, version_format::{self, VERSION_FORMAT},
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
                    .parents(idx)
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
                        return Err(eyre::eyre!(
                            "Existing version is less than its latest parent's version"
                        ));
                    } else {
                        map.insert(idx, existing);
                        return Ok(());
                    }
                } else {
                    let next_version = match (
                        &commit.subject,
                        commit.branch,
                        commit.parent_hashes.len() == 2,
                    ) {
                        (
                            Subject::Conventional(ConventionalSubject { breaking: true, .. }),
                            "main" | "master",
                            _,
                        ) => max_parent.major(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "feat",
                                ..
                            }),
                            "main" | "master",
                            _,
                        ) => max_parent.minor(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "fix", ..
                            }),
                            "main" | "master",
                            _,
                        ) => max_parent.patch(commit),
                        (Subject::Conventional(_), "main" | "master", _) => max_parent.build(commit),
                        (
                            Subject::Conventional(ConventionalSubject { breaking: true, .. }),
                            "staging",
                            _,
                        ) => max_parent.major(commit.clone()).rc(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "feat",
                                ..
                            }),
                            "staging",
                            _,
                        ) => max_parent.minor(commit.clone()).rc(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "fix", ..
                            }),
                            "staging",
                            _,
                        ) => max_parent.patch(commit.clone()).rc(commit),
                        (Subject::Conventional(_), "staging", _) => max_parent.rc(commit),
                        (
                            Subject::Conventional(ConventionalSubject { breaking: true, .. }),
                            "development",
                            _,
                        ) => max_parent.major(commit.clone()).beta(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "feat",
                                ..
                            }),
                            "development",
                            _,
                        ) => max_parent.minor(commit.clone()).beta(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "fix", ..
                            }),
                            "development",
                            _,
                        ) => max_parent.patch(commit.clone()).beta(commit),
                        (Subject::Conventional(_), "development", _) => max_parent.beta(commit),
                        (
                            Subject::Conventional(ConventionalSubject { breaking: true, .. }),
                            "next",
                            _,
                        ) => max_parent.major(commit.clone()).alpha(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "feat",
                                ..
                            }),
                            "next",
                            _,
                        ) => max_parent.minor(commit.clone()).alpha(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "fix", ..
                            }),
                            "next",
                            _,
                        ) => max_parent.patch(commit.clone()).alpha(commit),
                        (Subject::Conventional(_), "next", _) => max_parent.alpha(commit),
                        (
                            Subject::Conventional(ConventionalSubject { breaking: true, .. }),
                            _,
                            _,
                        ) => max_parent.major(commit.clone()).named(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "feat",
                                ..
                            }),
                            _,
                            _,
                        ) => max_parent.minor(commit.clone()).named(commit),
                        (
                            Subject::Conventional(ConventionalSubject {
                                commit_type: "fix", ..
                            }),
                            _,
                            _,
                        ) => max_parent.patch(commit.clone()).named(commit),
                        (Subject::Conventional(_), _, _) => max_parent.named(commit),
                        (Subject::Text(_), "main" | "master", true) => max_parent.release(commit),
                        (Subject::Text(_), "main" | "master", _) => max_parent.build(commit),
                        (Subject::Text(_), "staging", _) => max_parent.rc(commit),
                        (Subject::Text(_), "development", _) => max_parent.beta(commit),
                        (Subject::Text(_), "next", _) => max_parent.alpha(commit),
                        (Subject::Text(_), _, _) => max_parent.named(commit),
                    };

                    // dbg!(&next_version);

                    map.insert(idx, next_version);
                    return Ok(());
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
}
