#![feature(decl_macro)]
use clap::Parser;
use petgraph::graph::NodeIndex;
use version_map::{VersionMap, VersionMapData};
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
pub mod version_map;

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

    match args.command {
        None => {
            if logs.is_dirty() {
                println!("{}", logs.get_uncommited_version())
            } else {
                println!("{}", logs.get_latest_version())
            }
        },
        Some(command) => {
            match command {
                CCVerSubCommand::Tag(tag_args) => {
                    
                },
                _ => todo!(),
            }
        }
    }

    Ok(())
}
