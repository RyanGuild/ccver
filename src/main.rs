use std::io::BufRead;
use std::process::{Command, Output};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::env::current_dir;
use serde::Deserialize;
use itertools::Itertools;
use std::env::args;

#[derive(Debug, Deserialize)]
struct Config {
    major: Vec<String>,
    minor: Vec<String>,
    patch: Vec<String>
}


#[derive(Debug, Clone)]
struct RefLogLine {
    sha: String,
    ref_name: String,
    action: String,
    message: String
}


fn main() {
    Command::new("git")
        .arg("-v")
        .output().expect("git not installed");

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

    let config = config_paths.iter().map(|path| {
        let contents = read_to_string(path).unwrap();
        let config: Config = serde_yaml::from_str(&contents).unwrap();
        config
    }).reduce(|a, b| {
        Config {
            major: a.major.into_iter().merge(b.major.into_iter()).unique().collect(),
            minor: a.minor.into_iter().merge(b.minor.into_iter()).unique().collect(),
            patch: a.patch.into_iter().merge(b.patch.into_iter()).unique().collect()
        }
    }).unwrap();

    let git_reflog: Vec<RefLogLine> = Command::new("git")
        .arg("reflog")
        .output()
        .expect("pwd is not a git repository")
        .stdout
        .lines()
        .map(|line| line.expect("could not read line"))
        .map(|line| line.trim().to_string())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            RefLogLine {
                sha: parts[0].to_string(),
                ref_name: parts[1].to_string(),
                action: parts[2].to_string(),
                message: parts[3..].join(" ")
            }
        })
        .collect();

    println!("{:?}", git_reflog);








    



}