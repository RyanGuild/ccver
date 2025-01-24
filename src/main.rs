use itertools::Itertools;
use pest::Parser;
use serde::Deserialize;
use std::env::args;
use std::env::current_dir;
use std::fs::read_to_string;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Output};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "reflog.pest"]
struct RefLogParser;

#[derive(Debug, Deserialize)]
struct Config {
    major: Vec<String>,
    minor: Vec<String>,
    patch: Vec<String>,
}

#[derive(Debug, Clone)]
struct RefLogLine {
    sha: String,
    ref_name: String,
    action: String,
    message: String,
}

fn main() {
    Command::new("git")
        .arg("-v")
        .output()
        .expect("git not installed");

    let mut args = args();
    let cwd = current_dir().unwrap();

    // start at the current_dir and walk up the fs and collect the paths of the config files in a vector
    let mut config_paths: Vec<PathBuf> = Vec::new();
    let mut current_path = cwd.clone();
    loop {
        let config_path = current_path.join(".ccver");
        if config_path.exists() {
            config_paths.push(config_path.clone());
        }
        if current_path.parent().is_none() {
            break;
        }

        if args.contains("--no-config-search") {
            break;
        }
        current_path = current_path.parent().unwrap().to_path_buf();
    }

    let config = config_paths
        .iter()
        .map(|path| {
            let contents = read_to_string(path).unwrap();
            let config: Config = serde_yaml::from_str(&contents).unwrap();
            config
        })
        .reduce(|a, b| Config {
            major: a
                .major
                .into_iter()
                .merge(b.major.into_iter())
                .unique()
                .collect(),
            minor: a
                .minor
                .into_iter()
                .merge(b.minor.into_iter())
                .unique()
                .collect(),
            patch: a
                .patch
                .into_iter()
                .merge(b.patch.into_iter())
                .unique()
                .collect(),
        })
        .unwrap();

    let git_reflog: Vec<RefLogLine> = Command::new("git")
        .arg("reflog")
        .output()
        .expect("pwd is not a git repository")
        .stdout
        .lines()
        .map(|line| {
            if let Ok(l) = line {
                let parts: Vec<Vec<&str>> = l
                    .trim()
                    .split(":")
                    .map(|part| part.trim().split_whitespace().collect())
                    .collect();
                RefLogLine {
                    sha: parts[0][0].to_string(),
                    ref_name: parts[0][1].to_string(),
                    action: parts[1].join(" ").to_string(),
                    message: parts[2].join(" ").to_string(),
                }
            } else {
                panic!("error reading line")
            }
        })
        .collect();

    println!("{:?}", git_reflog);
}
