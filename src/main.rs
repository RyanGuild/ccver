use std::process::Command;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    major: Vec<String>,
    minor: Vec<String>,
    patch: Vec<String>
}



fn main() {
    Command::new("git")
        .arg("-v")
        .output().expect("git not installed");

    let cwd = current_dir().unwrap();

    let mut config = Config {
        major: Vec::new(),
        minor: Vec::new(),
        patch: Vec::new()
    };

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
        current_path = current_path.parent().unwrap().to_path_buf();
    }

    let config = config_paths.iter().map(|path| {
        let contents = read_to_string(path).unwrap();
        let config: Config = serde_yaml::from_str(&contents).unwrap();
        config
    }).reduce(|a, b| {
        let mut config = Config {
            major: a.major.clone(),
            minor: a.minor.clone(),
            patch: a.patch.clone()
        };
        config.major.extend(b.major.clone());
        config.minor.extend(b.minor.clone());
        config.patch.extend(b.patch.clone());
        config
    }).unwrap();

    println!("{:?}", config);


    



}