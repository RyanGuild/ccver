#![feature(decl_macro)]
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use std::env::current_dir;
use version_format::{PRE_TAG_FORMAT, VERSION_FORMAT};

pub mod args;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod version;
pub mod version_format;
pub mod version_map;

use args::*;
use eyre::Result;
use logs::Logs;

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

    if let Some(format_str) = args.format {
        let format_string = format_str.to_string();
        let format = parser::parse_version_format(&format_string, logs.get_graph()?)?;
        VERSION_FORMAT.set(format.clone());
        if let Some(pre_format) = format.prerelease {
            PRE_TAG_FORMAT.set(pre_format);
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
            _ => todo!(),
        },
    }

    Ok(())
}
