#![feature(decl_macro)]
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use std::{collections::HashMap, env::current_dir};
use version::{Version, VersionNumber};

pub mod args;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod version;
pub mod version_format;

use args::*;
use eyre::Result;
use logs::{Decoration, Logs, Subject};

fn main() -> Result<()> {
    Command::new("git")
        .arg("-v")
        .output()
        .expect("git not installed");

    let args = CCVerArgs::parse();
    // let _config = CCVerConfig::default()?;

    let path = args.path.map_or(
        current_dir().expect("could not get current dir"),
        PathBuf::from,
    );

    let mut logs = Logs::new(path);
    let graph_cell = logs.get_graph();
    let graph = graph_cell.borrow();
    let mut current_version = Version {
        major: VersionNumber::CCVer(0),
        minor: VersionNumber::CCVer(0),
        patch: VersionNumber::CCVer(0),
        prerelease: None,
    };
    let node_version_map: HashMap<_, _> = graph
        .history_windowed_parents()
        .map(|(commit, _parents)| {
            let commitidx = graph
                .commitidx(commit.commit_hash)
                .expect("hash not in graph");
            let existing_version_tag = commit
                .decorations
                .iter()
                .filter_map(|dec| {
                    if let Decoration::Tag(tag) = dec {
                        match tag {
                            logs::Tag::Text(_) => None,
                            logs::Tag::Version(version) => Some(version),
                        }
                    } else {
                        None
                    }
                })
                .next();

            if let Some(version) = existing_version_tag {
                current_version = version.clone();

                (commitidx, current_version.clone())
            } else if let Subject::Conventional(subject) = &commit.subject {
                current_version = match (subject.commit_type, subject.breaking, commit.branch) {
                    (_, breaking, _) if breaking => current_version.major(),
                    ("feat", _, _) => current_version.minor(),
                    ("fix", _, _) => current_version.patch(),
                    (_, _, "staging") => current_version.rc(),
                    (_, _, "development") => current_version.beta(),
                    (_, _, "next") => current_version.alpha(),
                    _ => current_version.build(),
                };
                (commitidx, current_version.clone())
            } else {
                current_version = current_version.build();
                (commitidx, current_version.clone())
            }
        })
        .collect();

    let mut versions = node_version_map.values().collect::<Vec<_>>();
    versions.sort();

    for version in versions {
        println!("{}", version);
    }

    Ok(())
}
