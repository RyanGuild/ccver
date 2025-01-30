// use itertools::Itertools;

use std::process::Command;
use clap::Parser;



pub mod parser;
pub mod args;
pub mod config;
pub mod commands;

use args::*;
use commands::*;
use args::CCVerSubCommand::*;
use config::CCVerConfig;

use eyre::Result;

fn main() -> Result<()> {
    Command::new("git")
        .arg("-v")
        .output()
        .expect("git not installed");

    let args = CCVerArgs::parse();
    let config = CCVerConfig::default()?;

    match args.command {
        Init(_) => init::run(args, config),
        Install(_) => install::run(args, config),
        Tag(_) => tag::run(args, config),
    }
}
