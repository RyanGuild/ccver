#![feature(decl_macro, lock_value_accessors)]

/// The main entry point for the `ccver` application.
///
/// This function parses command-line arguments, initializes logging, and
/// executes the specified command. It supports version formatting and tagging
/// operations.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the operation is successful, otherwise
///   returns an error.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The current directory cannot be determined.
/// * The version format cannot be parsed.
/// * The logs cannot be retrieved or processed.
/// * The `git` command is not installed.
///
/// # Example
///
/// ```sh
/// ccver --path /path/to/repo --format "vYY.CC.CC-pre.<short-sha>" tag
/// ```
pub mod args;
pub mod changelog;
pub mod git;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod pattern_macros;
pub mod version;
pub mod version_format;
pub mod version_map;

use std::env::current_dir;
use std::io::Read as _;
use std::path::PathBuf;

use args::*;
use changelog::ChangeLogData;
use clap::Parser;
use eyre::*;
use git::{git_installed, is_dirty};
use graph::CommitGraph;
use logs::GIT_FORMAT_ARGS;
use logs::Logs;
use version_format::VersionFormat;
use version_map::VersionMap;

fn main() -> Result<()> {
    git_installed()?;

    let args = CCVerArgs::parse();
    let path = args.path.map_or(
        current_dir().expect("could not get current dir"),
        PathBuf::from,
    );
    let mut stdin_string = String::new();

    let logs = if args.raw {
        std::io::stdin().read_to_string(&mut stdin_string)?;
        Logs::from_str(&stdin_string)?
    } else {
        Logs::from_path(&path)?
    };

    let graph = CommitGraph::new(&logs)?;

    let version_format = if let Some(format_str) = args.format {
        parser::parse_version_format(&format_str, &graph)?
    } else {
        VersionFormat::default()
    };

    let version_map = VersionMap::new(&graph, &version_format)?;

    let stdout = match args.command {
        None => {
            let ver = match is_dirty(&path) {
                Err(e) => {
                    if args.ci {
                        Err(e)
                    } else {
                        Ok(version_map
                            .get(graph.headidx())
                            .ok_or_eyre(eyre!("No version found"))?
                            .build(graph.head(), &version_format))
                    }
                }
                Result::Ok(_) => version_map
                    .get(graph.headidx())
                    .ok_or_eyre(eyre!("No version found"))
                    .cloned(),
            }?;
            if args.no_pre {
                format!("{}", ver.release(graph.head(), &version_format))
            } else {
                format!("{}", ver)
            }
        }
        Some(command) => match command {
            CCVerSubCommand::ChangeLog => {
                let changelog = ChangeLogData::new(&graph)?;
                format!("{}", changelog)
            }
            CCVerSubCommand::GitFormat => {
                format!(
                    "{} {} {} {} {}",
                    GIT_FORMAT_ARGS[0],
                    GIT_FORMAT_ARGS[1],
                    GIT_FORMAT_ARGS[2],
                    GIT_FORMAT_ARGS[3],
                    GIT_FORMAT_ARGS[4]
                )
            }
            _ => todo!(),
        },
    };

    println!("{}", stdout);

    Ok(())
}
