use std::{collections::HashMap, rc::Rc};

use petgraph::graph::NodeIndex;

use crate::{graph::CommitGraphData, logs::{ConventionalSubject, Subject}, version::Version};
use eyre::Result;

pub type VersionMap = Rc<VersionMapData>;

#[derive(Debug)]
pub struct VersionMapData(HashMap<NodeIndex, Version>);

impl VersionMapData {
    pub fn new(graph: &CommitGraphData) -> Result<VersionMap> {
        let mut map = HashMap::new();

        let tailidx = graph.tailidx();
        let tail = graph.tail();
        let initial_version = tail.as_initial_version();
        map.insert(tailidx, initial_version);

        let results = graph
            .dfs_postorder_history()
            .map(|(idx, commit)| {
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
                        map
                            .get(parent)
                            .cloned()
                            .or(
                                graph
                                    .get(idx)
                                    .expect("Idx Comes from graph source")
                                    .tagged_version()
                            )
                    })
                    .max()
                    .unwrap_or_default();

                if let Some(existing) = existing {
                    if existing < max_parent {
                        return Err(eyre::eyre!("Existing version is less than its latest parent's version"));
                    } else {
                        map.insert(idx, existing);
                        return Ok(())
                    }
                } else {
                    let next_version = match (&commit.subject, commit.branch, commit.parent_hashes.len() == 2) {
                        (Subject::Conventional(ConventionalSubject{ breaking: true, .. }), "main" | "master", _) => max_parent.major(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "feat", .. }), "main" | "master", _) => max_parent.minor(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "fix", .. }), "main" | "master", _) => max_parent.patch(),
                        (Subject::Conventional(_), "main" | "master", _) => max_parent.build(),
                        (Subject::Conventional(ConventionalSubject{ breaking: true, .. }), "staging", _) => max_parent.major().rc(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "feat", .. }), "staging", _) => max_parent.minor().rc(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "fix", .. }), "staging", _) => max_parent.patch().rc(),
                        (Subject::Conventional(_), "staging", _) => max_parent.rc(),
                        (Subject::Conventional(ConventionalSubject{ breaking: true, .. }), "development", _) => max_parent.major().beta(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "feat", .. }), "development", _) => max_parent.minor().beta(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "fix", .. }), "development", _) => max_parent.patch().beta(),
                        (Subject::Conventional(_), "development", _) => max_parent.beta(),
                        (Subject::Conventional(ConventionalSubject{ breaking: true, .. }), "next", _) => max_parent.major().alpha(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "feat", .. }), "next", _) => max_parent.minor().alpha(),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "fix", .. }), "next", _) => max_parent.patch().alpha(),
                        (Subject::Conventional(_), "next", _) => max_parent.alpha(),
                        (Subject::Conventional(ConventionalSubject{ breaking: true, .. }), s, _) => max_parent.major().named(s),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "feat", .. }), s, _) => max_parent.minor().named(s),
                        (Subject::Conventional(ConventionalSubject{ commit_type: "fix", .. }), s, _) => max_parent.patch().named(s),
                        (Subject::Conventional(_), s, _) => max_parent.named(s),
                        (Subject::Text(_), "main" | "master", true) => max_parent.release(),
                        (Subject::Text(_), "main" | "master", _) => max_parent.build(),
                        (Subject::Text(_), "staging", _) => max_parent.rc(),
                        (Subject::Text(_), "development", _) => max_parent.beta(),
                        (Subject::Text(_), "next", _) => max_parent.alpha(),
                        (Subject::Text(_), s, _) => max_parent.named(s)
                    };


                    

                    map.insert(idx, next_version);
                    return Ok(());
                }
            }).collect::<Vec<Result<()>>>();

        results.iter().filter_map(|r| r.as_ref().err()).for_each(|e| eprintln!("{}", e));

        Ok(Rc::new(Self(map)))
    }

    pub fn get(&self, idx: NodeIndex) -> Option<&Version> {
        self.0.get(&idx)
    }

    pub fn set(&mut self, idx: NodeIndex, version: Version) {
        self.0.insert(idx, version);
    }
}
