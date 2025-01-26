use std::{env::current_dir, path::PathBuf, process::Command, str::FromStr};

use crate::{args::InitArgs, config::CCVerConfig, parser::git::parse_git_log};

pub fn run(args: InitArgs, config: CCVerConfig) {
    let working_dir = if let Some(path) = args.path {
        PathBuf::from_str(&path).expect("Invalid path")
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

    let log = parse_git_log(&log_str);

    println!("{:#?}", log);
}
