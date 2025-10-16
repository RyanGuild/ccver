use std::{cmp::Ordering, fmt::Display, rc::Rc};

use eyre::*;
use petgraph::graph::NodeIndex;

use crate::{
    graph::{
        GraphOps, assign_versions::AsLogEntry, head::HasHead,
        parents_and_children::HasParentsAndChildren, version::ExistingVersionExt,
    },
    logs::{ConventionalSubject, LogEntry, Subject},
    major_commit_types, major_conventional_subject, minor_commit_types, minor_conventional_subject,
    patch_commit_types, patch_conventional_subject, semver_advancing_conventional_subject,
    semver_advancing_subject,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ChangeLogData<'a>(Rc<[ChangeScoped<'a>]>);
pub type ChangeLog<'a> = Rc<ChangeLogData<'a>>;

impl Ord for ChangeScoped<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ChangeScoped::All(a), ChangeScoped::All(b))
            | (ChangeScoped::Scoped(_, a), ChangeScoped::Scoped(_, b))
            | (ChangeScoped::All(a), ChangeScoped::Scoped(_, b))
            | (ChangeScoped::Scoped(_, a), ChangeScoped::All(b)) => a.cmp(b),
        }
    }
}

impl Display for ChangeLogData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.0.is_sorted());
        writeln!(f, "# ChangeLog",)?;

        let mut current_scope: Option<&str> = None;
        let mut last_level: Option<&str> = None;
        for change in self.0.iter() {
            match change {
                ChangeScoped::All(change) => match change {
                    Change::Breaking(desc, date) => {
                        if last_level != Some("Breaking Changes") {
                            writeln!(f, "## Breaking Changes")?;
                            last_level = Some("Breaking Changes");
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Feature(desc, date) => {
                        if last_level != Some("Features") {
                            writeln!(f, "## Features")?;
                            last_level = Some("Features");
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Fix(desc, date) => {
                        if last_level != Some("Fixes") {
                            writeln!(f, "## Fixes")?;
                            last_level = Some("Fixes");
                        };
                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Named(name, desc, date) => {
                        if last_level != Some(name.as_ref()) {
                            writeln!(f, "## {}", name)?;
                            last_level = Some(name.as_ref());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Misc(desc, date) => {
                        if last_level != Some("Misc") {
                            writeln!(f, "## Misc")?;
                            last_level = Some("Misc");
                        }

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                },
                ChangeScoped::Scoped(scope, change) => match change {
                    Change::Breaking(desc, date) => {
                        if last_level != Some("Breaking Changes") {
                            writeln!(f, "## Breaking Changes")?;
                            last_level = Some("Breaking Changes");
                        };

                        if current_scope != Some(scope.as_ref()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.as_ref());
                        };
                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Feature(desc, date) => {
                        if last_level != Some("Features") {
                            writeln!(f, "## Features")?;
                            last_level = Some("Features");
                        };

                        if current_scope != Some(scope.as_ref()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.as_ref());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Fix(desc, date) => {
                        if last_level != Some("Fixes") {
                            writeln!(f, "## Fixes")?;
                            last_level = Some("Fixes");
                        };

                        if current_scope != Some(scope.as_ref()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.as_ref());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Named(name, desc, date) => {
                        if last_level != Some(name.as_ref()) {
                            writeln!(f, "## {}", name)?;
                            last_level = Some(name.as_ref());
                        };

                        if current_scope != Some(scope.as_ref()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.as_ref());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                    Change::Misc(desc, date) => {
                        if last_level != Some("Misc") {
                            writeln!(f, "## Misc")?;
                            last_level = Some("Misc");
                        }

                        if current_scope != Some(scope.as_ref()) {
                            writeln!(f, "### {}", scope)?;
                            current_scope = Some(scope.as_ref());
                        };

                        writeln!(f, "- ({}): {}", date, desc)?;
                    }
                },
            }
        }

        std::fmt::Result::Ok(())
    }
}

impl PartialOrd for ChangeScoped<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ChangeScoped<'a> {
    All(Change<'a>),
    Scoped(&'a str, Change<'a>),
}

#[derive(Debug, PartialEq, Eq)]
enum Change<'a> {
    Breaking(&'a str, chrono::DateTime<chrono::Utc>),
    Feature(&'a str, chrono::DateTime<chrono::Utc>),
    Fix(&'a str, chrono::DateTime<chrono::Utc>),
    Named(&'a str, &'a str, chrono::DateTime<chrono::Utc>),
    Misc(&'a str, chrono::DateTime<chrono::Utc>),
}

impl<'a> From<LogEntry<'a>> for ChangeScoped<'a> {
    fn from(log_entry: LogEntry<'a>) -> Self {
        match log_entry.subject {
            Subject::Text(text) => ChangeScoped::All(Change::Misc(text, log_entry.commit_datetime)),
            Subject::Conventional(ConventionalSubject {
                description,
                commit_type,
                scope,
                breaking,
                ..
            }) => {
                let change = if breaking {
                    Change::Breaking(description, log_entry.commit_datetime)
                } else {
                    match commit_type {
                        major_commit_types!() => {
                            Change::Breaking(description, log_entry.commit_datetime)
                        }
                        minor_commit_types!() => {
                            Change::Feature(description, log_entry.commit_datetime)
                        }
                        patch_commit_types!() => {
                            Change::Fix(description, log_entry.commit_datetime)
                        }
                        _ => Change::Named(commit_type, description, log_entry.commit_datetime),
                    }
                };

                match scope {
                    Some(scope) => ChangeScoped::Scoped(scope, change),
                    None => ChangeScoped::All(change),
                }
            }
        }
    }
}

impl<'a> ChangeLogData<'a> {
    pub fn new<N, E, Ty, Ix, T>(graph: &'a T) -> Result<ChangeLog<'a>>
    where
        T: GraphOps<N, E, Ty, Ix> + HasHead<N, E, Ty, Ix> + HasParentsAndChildren<N, E, Ty, Ix>,
        N: AsLogEntry + ExistingVersionExt + 'a,
        Ix: Copy,
    {
        let root = graph
            .head_idx()
            .ok_or_eyre("No head index found in graph")?;
        Self::from_index(graph, root)
    }

    pub fn from_index<N, E, Ty, Ix, T>(graph: &'a T, from: NodeIndex<Ix>) -> Result<ChangeLog<'a>>
    where
        T: GraphOps<N, E, Ty, Ix> + HasHead<N, E, Ty, Ix> + HasParentsAndChildren<N, E, Ty, Ix>,
        N: AsLogEntry + ExistingVersionExt + 'a,
        Ix: Copy,
    {
        let mut stack = graph.parent_idxs(from);
        let mut versions = Vec::new();
        while let Some(parent_idx) = stack.pop() {
            let parent_version: LogEntry<'a> = graph
                .node_weight(parent_idx)
                .ok_or_eyre("No node weight found in graph")?
                .as_log_entry()
                .clone();
            if matches!(parent_version.subject, semver_advancing_subject!()) {
                versions.push(parent_version);
                break;
            } else {
                versions.push(parent_version);
                stack.extend(graph.parent_idxs(parent_idx));
            }
        }

        let mut changes: Vec<ChangeScoped<'a>> = versions
            .into_iter()
            .map(|commit| commit.clone().into())
            .collect();

        changes.sort();

        Ok(Rc::new(Self(changes.into())))
    }
}

impl Ord for Change<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Change::Breaking(_, a), Change::Breaking(_, b)) => a.cmp(&b),
            (Change::Feature(_, a), Change::Feature(_, b)) => a.cmp(&b),
            (Change::Fix(_, a), Change::Fix(_, b)) => a.cmp(&b),
            (Change::Named(_, _, a), Change::Named(_, _, b)) => a.cmp(&b),
            (Change::Misc(_, a), Change::Misc(_, b)) => a.cmp(&b),
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

impl PartialOrd for Change<'_> {
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
            ChangeScoped::All(Change::Breaking(
                "Added Emojis",
                dummy_date.with_hour(1).unwrap(),
            )),
            ChangeScoped::All(Change::Feature(
                "Temp Removed Emojis",
                dummy_date.with_hour(2).unwrap(),
            )),
            ChangeScoped::All(Change::Fix(
                "Fixed Emojis",
                dummy_date.with_hour(3).unwrap(),
            )),
            ChangeScoped::Scoped(
                "./src/emoji.rs",
                Change::Named(
                    "docs",
                    "Documented Emojis",
                    dummy_date.with_hour(4).unwrap(),
                ),
            ),
        ]));

        assert_eq!(
            format!("{}", cl),
            indoc! {
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
            }
        );
    }
}
