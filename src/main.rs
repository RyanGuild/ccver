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
use clap::Parser;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::Command;
use version_format::{PRE_TAG_FORMAT, VERSION_FORMAT};

pub mod args;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod version;
pub mod version_format;
pub mod version_map;
pub mod changelog;
pub mod pattern_macros;

use args::*;
use eyre::Result;
use logs::Logs;

fn main() -> Result<()> {
    Command::new("git")
        .arg("-v")
        .output()
        .expect("git not installed");

    let args = CCVerArgs::parse();
    let path = args.path.map_or(
        current_dir().expect("could not get current dir"),
        PathBuf::from,
    );

    let mut logs = Logs::new(path);

    if let Some(format_str) = args.format {
        let format_string = format_str.to_string();
        let format = parser::parse_version_format(&format_string, logs.get_graph()?)?;
        VERSION_FORMAT.replace(format.clone())?;
        if let Some(pre_format) = format.prerelease {
            PRE_TAG_FORMAT.replace(pre_format)?;
        }
    }

    match args.command {
        None => {
            if logs.is_dirty() {
                println!("{}", logs.get_uncommited_version()?)
            } else {
                println!("{}", logs.get_latest_version()?)
            }
        }
        Some(command) => match command {
            CCVerSubCommand::Tag(_) => {}
            CCVerSubCommand::ChangeLog => {
                let changelog = logs.get_changelog()?;
                println!("{}", changelog)
            }
            _ => todo!(),
        },
    }

    Ok(())
}
