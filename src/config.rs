use std::{
    env::current_dir,
    path::PathBuf,
    str::FromStr,
};

use eyre::Result;
use figment::{
    providers::{Format, Json},
    Figment,
};
use serde::Deserialize;


#[derive(Deserialize)]
pub struct CCVerConfig {
    pub path: Option<PathBuf>,
    pub major: Option<Vec<Targets>>,
    pub minor: Option<Vec<Targets>>,
    pub patch: Option<Vec<Targets>>,
}

#[derive(Deserialize)]
pub enum Targets {
    CC(CCTargets),
    Git(GitTargets)
}

#[derive(Deserialize)]
pub struct CCTargets {
    pub type_name: Option<String>,
    pub scope: Option<String>
}

#[derive(Deserialize)]
pub struct GitTargets {
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub remote: Option<String>
}

impl CCVerConfig {
    pub fn default() -> Result<CCVerConfig> {
        let system = PathBuf::from_str("/etc/ccver/.ccver.json")?;
        let home = PathBuf::from_str("~/.ccver.json")?;
        let cwd = current_dir()?.join("ccver.json");

        let  fg = Figment::new()
            .merge(Json::file(system))
            .merge(Json::file(home))
            .merge(Json::file(cwd));

        let res: CCVerConfig = fg.extract()?;
        return Ok(res);
    }
}
