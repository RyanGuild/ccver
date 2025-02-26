use std::{cmp::Ordering, collections::HashMap, fmt::Display, rc::Rc, vec};

use eyre::*;
use petgraph::graph::NodeIndex;

use crate::{
    graph::CommitGraph,
    logs::{ConventionalSubject, LogEntry, Subject},
    pattern_macros::{major_commit_types, minor_commit_types, patch_commit_types, semver_advancing_subject},
    version::Version,
    version_map::VersionMap,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ChangeLogData(Rc<[ChangeScoped]>);
pub type ChangeLog = Rc<ChangeLogData>;

impl Ord for ChangeScoped {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ChangeScoped::All(a), ChangeScoped::All(b))
            | (ChangeScoped::Scoped(_, a), ChangeScoped::Scoped(_, b))
            | (ChangeScoped::All(a), ChangeScoped::Scoped(_, b))
            | (ChangeScoped::Scoped(_, a), ChangeScoped::All(b)) => a.cmp(b),
        }
    }
}

impl Display for ChangeLogData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.0.is_sorted());
        writeln!(f, "# ChangeLog", )?;
        
        let mut current_scope: Option<String> = None;
        let mut last_level: Option<String> = None;
        for change in self.0.iter() {
            match change {
                ChangeScoped::All(change) => {
                    match change {
                        Change::Breaking(desc, date) => {
                            if last_level != "Breaking Changes".to_string().into() {
                                writeln!(f, "## Breaking Changes")?;
                                last_level = Some("Breaking Changes".to_string());
                            };


                            writeln!(f, "- ({}): {}", date, desc)?;
                        }
                        Change::Feature(desc, date) => {
                            if last_level != "Features".to_string().into() {
                                writeln!(f, "## Features")?;
                                last_level = Some("Features".to_string());
                            };

                            writeln!(f, "- ({}): {}", date, desc)?;
                        }
                        Change::Fix(desc, date) => {
                            if last_level != "Fixes".to_string().into() {
                                writeln!(f, "## Fixes")?;
                                last_level = Some("Fixes".to_string());
                            };
                            writeln!(f, "- ({}): {}", date, desc)?;
                        }
                        Change::Named(name, desc, date) => {
                            if last_level != name.to_string().into() {
                                writeln!(f, "## {}", name)?;
                                last_level = Some(name.to_string());
                            };

                            writeln!(f, "- ({}): {}", date, desc)?;
                        }
                        Change::Misc(desc, date) => {
                            if last_level != "Misc".to_string().into() {
                                writeln!(f, "## Misc")?;
                                last_level = Some("Misc".to_string());
                            }
                            
                            writeln!(f, "- ({}): {}", date, desc)?;
                        }
                    }
                },
                ChangeScoped::Scoped(scope, change) => {
                  match change {
                    Change::Breaking(desc, date) => {
                        if last_level != Some("Breaking Changes".to_string()) {
                            writeln!(f, "## Breaking Changes")?;
                            last_level = Some("Breaking Changes".to_string());
                        };
                        
                        if current_scope != Some(scope.clone()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.clone());
                        };
                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Feature(desc, date) => {
                        if last_level != Some("Features".to_string()) {
                            writeln!(f, "## Features")?;
                            last_level = "Features".to_string().into();
                        };

                        if current_scope != Some(scope.clone()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.clone());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Fix(desc, date) => {
                        if last_level != Some("Fixes".to_string()) {
                            writeln!(f, "## Fixes")?;
                            last_level = Some("Fixes".to_string());
                        };

                        if current_scope != Some(scope.clone()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.clone());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Named(name, desc, date) => {
                        if last_level != Some(name.clone()) {
                            writeln!(f, "## {}", name)?;
                            last_level = Some(name.to_string());
                        };

                        if current_scope != Some(scope.clone()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.clone());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Misc(desc, date) => {
                        if last_level != Some("Misc".to_string()) {
                            writeln!(f, "## Misc")?;
                            last_level = Some("Misc".to_string());
                        }

                        if current_scope != Some(scope.clone()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.clone());
                        };
                        
                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                  }  
                },
            }
        };


        std::fmt::Result::Ok(())
    }
}

impl PartialOrd for ChangeScoped {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ChangeScoped {
    All(Change),
    Scoped(String, Change),
}

#[derive(Debug, PartialEq, Eq)]
enum Change {
    Breaking(String, chrono::DateTime<chrono::Utc>),
    Feature(String, chrono::DateTime<chrono::Utc>),
    Fix(String, chrono::DateTime<chrono::Utc>),
    Named(String, String, chrono::DateTime<chrono::Utc>),
    Misc(String, chrono::DateTime<chrono::Utc>),
}

impl ChangeLogData {
    pub fn new(graph: CommitGraph, version_map: VersionMap) -> Result<ChangeLog> {
        let root = graph.headidx();
        Self::from_index(graph, version_map, root)
    }

    pub fn from_index(graph: CommitGraph, version_map: VersionMap, from: NodeIndex) -> Result<ChangeLog> {
        let versions = {
            let mut stack = graph.parents(from);
            let current_ver = graph.get(from).ok_or_eyre("Foreign NodeIndex detected please only input ")?;
            let mut versions = vec![current_ver];
            while let Some(parent) = stack.pop() {
                let parent_ver = version_map
                    .get(parent)
                    .ok_or_eyre(eyre!("Could not find version for commit"))?;
                let parent_commit = graph.get(parent).expect("Idx Comes from graph source");

                match parent_commit.subject {
                    semver_advancing_subject!() => {},
                    _ => {
                        stack.extend(graph.parents(parent));
                        versions.push(parent_commit);
                    }
                };
            }
            versions
        };

        let mut changes = versions.iter().map(|commit| {
            match &commit.subject {
                Subject::Conventional(ConventionalSubject {
                    commit_type,
                    scope: None,
                    description,
                    ..
                }) => match *commit_type {
                    major_commit_types!() => ChangeScoped::All(Change::Breaking(
                        description.to_string(),
                        commit.commit_datetime,
                    )),
                    minor_commit_types!() => ChangeScoped::All(Change::Feature(
                        description.to_string(),
                        commit.commit_datetime,
                    )),
                    patch_commit_types!() => ChangeScoped::All(Change::Fix(
                        description.to_string(),
                        commit.commit_datetime,
                    )),
                    _ => ChangeScoped::All(Change::Named(
                        commit_type.to_string(),
                        description.to_string(),
                        commit.commit_datetime,
                    )),
                },
                Subject::Conventional(ConventionalSubject {
                    commit_type,
                    scope: Some(scope),
                    description,
                    ..
                }) => match *commit_type {
                    major_commit_types!() => ChangeScoped::Scoped(
                        scope.to_string(),
                        Change::Breaking(description.to_string(), commit.commit_datetime),
                    ),
                    minor_commit_types!() => ChangeScoped::Scoped(
                        scope.to_string(),
                        Change::Feature(description.to_string(), commit.commit_datetime),
                    ),
                    patch_commit_types!() => ChangeScoped::Scoped(
                        scope.to_string(),
                        Change::Fix(description.to_string(), commit.commit_datetime),
                    ),
                    _ => ChangeScoped::Scoped(
                        scope.to_string(),
                        Change::Named(
                            commit_type.to_string(),
                            description.to_string(),
                            commit.commit_datetime,
                        )
                    )
                },
                Subject::Text(t) => {
                    ChangeScoped::All(Change::Misc(t.to_string(), commit.commit_datetime))
                }
            }
        }).collect::<Vec<_>>();

        changes.sort();

        Ok(Rc::new(Self(changes.into())))
    }
}

impl Ord for Change {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Change::Breaking(_, a), Change::Breaking(_, b)) => a.cmp(b),
            (Change::Feature(_, a), Change::Feature(_, b)) => a.cmp(b),
            (Change::Fix(_, a), Change::Fix(_, b)) => a.cmp(b),
            (Change::Named(_, _, a), Change::Named(_, _, b)) => a.cmp(b),
            (Change::Misc(_, a), Change::Misc(_, b)) => a.cmp(b),
            (Change::Breaking(_, _), _) => Ordering::Less,
            (Change::Feature(_, _), Change::Breaking(_, _)) => Ordering::Greater,
            (Change::Feature(_, _), _) => Ordering::Less,
            (Change::Fix(_, _), Change::Breaking(_, _)) => Ordering::Greater,
            (Change::Fix(_, _), Change::Feature(_, _)) => Ordering::Greater,
            (Change::Fix(_, _), _) => Ordering::Less,
            (Change::Named(_, _, _), Change::Breaking(_, _)) => Ordering::Greater,
            (Change::Named(_, _, _), Change::Feature(_, _)) => Ordering::Greater,
            (Change::Named(_, _, _), Change::Fix(_, _)) => Ordering::Greater,
            (Change::Named(_, _, _), _) => Ordering::Less,
            (Change::Misc(_, _), _) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Change {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[cfg(test)]
mod changelog_tests {
    use std::rc::Rc;

    use chrono::Timelike;
    use indoc::*;

    use crate::changelog::{Change, ChangeLogData, ChangeScoped};

    #[test]
    fn test_display_changelog() {
        let dummy_date = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let cl = ChangeLogData(Rc::new([
            ChangeScoped::All(Change::Breaking("Added Emojis".to_string(), dummy_date.with_hour(1).unwrap())),
            ChangeScoped::All(Change::Feature("Temp Removed Emojis".to_string(), dummy_date.with_hour(2).unwrap())),
            ChangeScoped::All(Change::Fix("Fixed Emojis".to_string(), dummy_date.with_hour(3).unwrap())),
            ChangeScoped::Scoped("./src/emoji.rs".to_string(), Change::Named("docs".to_string(),"Documented Emojis".to_string(), dummy_date.with_hour(4).unwrap())),
        ]));

        assert_eq!(format!("{}", cl), indoc! {
           "
            # ChangeLog
            ## Breaking Changes
            - (1970-01-01 01:00:00 UTC): Added Emojis
            ## Features
            - (1970-01-01 02:00:00 UTC): Temp Removed Emojis
            ## Fixes
            - (1970-01-01 03:00:00 UTC): Fixed Emojis
            ## docs
            ### ./src/emoji.rs
            - (1970-01-01 04:00:00 UTC): Documented Emojis
            "
        });
    }
}