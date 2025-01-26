use serde::Deserialize;
use std::{env::current_dir, fs::read_to_string, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct CCVerConfig {
    major: Vec<String>,
    minor: Vec<String>,
    patch: Vec<String>,
}

impl CCVerConfig {
    pub fn new(override_config: CCVerConfig) -> CCVerConfig {
        CCVerConfig::default().extend(override_config)
    }

    pub fn default() -> CCVerConfig {
        CCVerConfig {
            major: vec!["breaking".to_string()],
            minor: vec!["feat".to_string()],
            patch: vec!["fix".to_string()],
        }
        .extend(CCVerConfig::from_fs())
    }

    pub fn from_fs() -> CCVerConfig {
        ConfigPaths(current_dir().expect("Could not get current directory"))
            .map(CCVerConfig::from_path)
            .reduce(CCVerConfig::extend)
            .expect("No config found")
    }

    pub fn from_path(path: PathBuf) -> CCVerConfig {
        serde_yaml::from_str(&read_to_string(path).expect("Could not read config file"))
            .expect("Could not parse config file")
    }

    pub fn extend(mut self, source: CCVerConfig) -> Self {
        self.major.extend(source.major);
        self.minor.extend(source.minor);
        self.patch.extend(source.patch);
        self
    }
    
    pub fn with_args(mut self, args: &crate::args::CCVerArgs) -> Self {
        match args.command {
           _ => {} 
        };
        self
    }
}

struct ConfigPaths(PathBuf);

impl Iterator for ConfigPaths {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let current_path = self.0.clone();

        if current_path.parent().is_none() {
            return None;
        };

        self.0 = current_path.parent().unwrap().to_path_buf();

        if current_path.join(".ccver").exists() {
            return Some(current_path.join(".ccver"));
        };

        return self.next();
    }
}
