// use itertools::Itertools;

use clap::Parser;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::Command;

pub mod args;
pub mod config;
pub mod logs;
pub mod graph;
pub mod parser;
pub mod version;

use args::*;
use config::CCVerConfig;
use logs::Logs;
use parser::Subject::*;
use version::Version;


use eyre::Result;

fn main() -> Result<()> {
    Command::new("git")
        .arg("-v")
        .output()
        .expect("git not installed");

    let args = CCVerArgs::parse();
    let config = CCVerConfig::default()?;

    let path = args.path.map_or(
        current_dir().expect("could not get current dir"),
        PathBuf::from,
    );

    let mut logs = Logs::new(path);
    let graph_cell = logs.get_graph();
    let graph = graph_cell.borrow();

    let versions = vec![Version::default()];

    graph.dfs_postorder_history().for_each(|(idx, commit)| {
        dbg!(commit.name);


        match &commit.subject {
            _=> {},
            Conventional(ccdata) => {
                
            },
        };

    });

    Ok(())
}
