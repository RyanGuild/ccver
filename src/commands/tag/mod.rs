use crate::{config::TagArgs, config::CCVerConfig};
use std::{env::Command};
use eyre::Result

pub fn run(args: TagArgs, config: CCVerConfig) -> Result<String> {
    Command::new("git").args(["tag", "-l"])
}