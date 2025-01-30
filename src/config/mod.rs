use std::{
    env::current_dir,
    path::{Path, PathBuf},
    str::FromStr,
    vec,
};

use directories::BaseDirs;
use eyre::{OptionExt, Result};
use figment::{
    providers::{Env, Format, Json, Toml, Yaml},
    Figment,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CCVerConfig {
    commit_type: Option<LevelIndicators>,
    scope: Option<LevelIndicators>,
    branch: Option<LevelIndicators>,
}

#[derive(Deserialize)]
struct LevelIndicators {
    major: Option<Vec<String>>,
    minor: Option<Vec<String>>,
    patch: Option<Vec<String>>,
}

impl CCVerConfig {
    pub fn default() -> Result<CCVerConfig> {
        let system = PathBuf::from_str("/etc/ccver")?;
        let home = BaseDirs::new()
            .ok_or_eyre("could not get directories")?
            .home_dir()
            .to_path_buf();
        let cwd = current_dir()?;

        let  fg = Figment::new()
            .merge(Toml::file(system.join("ccver.toml")))
            .merge(Toml::file(home.join("ccver.toml")))
            .merge(Toml::file(cwd.join("ccver.toml")))
            .merge(Json::file(system.join("ccver.json")))
            .merge(Json::file(home.join("ccver.json")))
            .merge(Json::file(cwd.join("ccver.json")))
            .merge(Yaml::file(system.join("ccver.yaml")))
            .merge(Yaml::file(home.join("ccver.yaml")))
            .merge(Yaml::file(cwd.join("ccver.yaml")))
            .merge(Yaml::file(system.join(".ccver")))
            .merge(Yaml::file(home.join(".ccver")))
            .merge(Yaml::file(home.join(".ccver")))
            .merge(Env::prefixed("CCVER_"));

        let res: CCVerConfig = fg.extract()?;
        return Ok(res);
    }
}
