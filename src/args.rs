use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    version = "0.0.1",
    about = "A tool for managing versioning in git repositories"
)]
pub struct CCVerArgs {
    #[command(subcommand)]
    pub command: Option<CCVerSubCommand>,

    #[arg(long = "path", short = 'p', env = "CCVER_PATH", default_value = ".")]
    pub path: PathBuf,

    #[arg(long = "message", short = 'm', env = "CCVER_PEEK_MESSAGE")]
    pub message: Option<String>,

    #[arg(
        long = "force-major",
        env = "CCVER_FORCE_MAJOR",
        default_value = "false"
    )]
    pub force_major: bool,

    #[arg(
        long = "force-minor",
        env = "CCVER_FORCE_MINOR",
        default_value = "false"
    )]
    pub force_minor: bool,

    #[arg(
        long = "force-patch",
        env = "CCVER_FORCE_PATCH",
        default_value = "false"
    )]
    pub force_patch: bool,

    #[arg(
        long = "format",
        short = 'f',
        env = "CCVER_FORMAT",
        default_value = "vCC.CC.CC-build.CC"
    )]
    pub format: String,

    #[arg(long = "no-pre", env = "CCVER_NO_PRE", default_value = "false")]
    pub no_pre: bool,

    #[arg(
        long = "raw",
        short = 'r',
        env = "CCVER_RAW",
        help = "Collect logs from stdin (must use --format=$(ccver git-format))"
    )]
    pub raw: bool,

    #[arg(
        long = "ci",
        env = "CCVER_CI",
        help = "Throw an error if the repository is dirty",
        default_value = "false"
    )]
    pub ci: bool,

    #[arg(long = "tag-all", env = "CCVER_TAG_ALL", default_value = "false")]
    pub tag_all: bool,

    #[arg(long = "tag", short = 't', env = "CCVER_TAG", default_value = "false")]
    pub tag: bool,

    #[arg(long = "changelog", short = 'l', env = "CCVER_EMIT_CHANGELOG")]
    pub changelog_path: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum CCVerSubCommand {
    #[command(about = "Print the git format string")]
    GitFormat,
}
