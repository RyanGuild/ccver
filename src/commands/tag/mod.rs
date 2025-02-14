use std::process::Command;

use crate::{args::CCVerArgs, config::CCVerConfig};
use eyre::Result;

pub fn run(args: CCVerArgs, config: CCVerConfig) -> Result<String> {
    Command::new("git").args(["tag", "-l"]).output();
    panic!("Not implemented");
}