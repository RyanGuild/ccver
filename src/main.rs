#![feature(decl_macro)]

// use itertools::Itertools;

use clap::Parser;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::Command;

pub mod args;
pub mod config;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod version;
pub mod version_format;

use args::*;
use config::CCVerConfig;
use logs::Logs;
use version::Version;

use eyre::Result;

fn main() -> Result<()> {
    Command::new("git")
        .arg("-v")
        .output()
        .expect("git not installed");

    let args = CCVerArgs::parse();
    let _config = CCVerConfig::default()?;

    let path = args.path.map_or(
        current_dir().expect("could not get current dir"),
        PathBuf::from,
    );

    let mut logs = Logs::new(path);
    let graph_cell = logs.get_graph();
    let graph = graph_cell.borrow();
    graph.history_windowed_childeren().for_each(|(commit, children)| {
        println!("commit: {:?}", commit);
        children.iter().for_each(|child| {
            println!("child: {:?}", child);
        });
    });

    Ok(())
}
