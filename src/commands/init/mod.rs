use std::{env::current_dir, path::PathBuf, process::Command, str::FromStr};

use crate::{config::CCVerConfig, args::CCVerArgs};

use eyre::Result;

pub fn run(args: CCVerArgs, config: CCVerConfig) -> Result<String> {
    let working_dir = if let Some(path) = args.path.as_ref() {
        PathBuf::from_str(path).expect("Invalid path")
    } else {
        current_dir().expect("Failed to get current directory")
    };

    Command::new("git")
        .arg("status")
        .current_dir(&working_dir)
        .output()
        .expect("not a git repository");

    let log_str = String::from_utf8(
        Command::new("git")
            .arg("log")
            .arg("--full-history")
            .arg("--pretty=raw")
            .arg("--decorate=full")
            .arg("--all")
            .current_dir(&working_dir)
            .output()
            .expect("Failed to get git log")
            .stdout,
    )
    .expect("Failed to convert git log to string");


    panic!("Not implemented");
}
