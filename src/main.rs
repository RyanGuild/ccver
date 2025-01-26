// use itertools::Itertools;

use std::process::Command;
use clap::Parser;


pub mod config;
pub mod parser;
pub mod args;
pub mod commands;

use config::*;
use args::*;
use args::CCVerSubCommand::*;
use commands::*;

fn main() {
    Command::new("git")
        .arg("-v")
        .output()
        .expect("git not installed");

    let args = CCVerArgs::parse();
    let config = CCVerConfig::default();

    match args.command {
        Init(args) => init::run(args, config),
        Install(args) => install::run(args, config),
        Tag(args) => tag::run(args, config),
    }
}
