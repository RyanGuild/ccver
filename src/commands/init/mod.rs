use std::{env::current_dir, path::PathBuf, process::Command, str::FromStr};

use crate::{args::InitArgs, config::CCVerConfig};

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


    let log_str = Command::new("git").arg("log").arg("--full-history").arg("--pretty=raw");



}
        