use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    version = "0.0.1",
    about = "A tool for managing versioning in git repositories"
)]
pub struct CCVerArgs {
    #[command(subcommand)]
    pub command: Option<CCVerSubCommand>,

    #[arg(long = "path", short = 'p')]
    pub path: Option<String>,

    #[arg(long = "force-major")]
    pub force_major: bool,

    #[arg(long = "force-minor")]
    pub force_minor: bool,

    #[arg(long = "force-patch")]
    pub force_patch: bool,

    #[arg(long = "format", short = 'f')]
    pub format: Option<String>,

    #[arg(long = "no-pre")]
    pub no_pre: bool,

    #[arg(
        long = "raw",
        short = 'r',
        help = "Collect logs from stdin (must use --format=$(ccver git-format))"
    )]
    pub raw: bool,

    #[arg(long = "ci", help = "Throw an error if the repository is dirty")]
    pub ci: bool,
}

#[derive(Args, Debug)]
#[command(about = "Tag the current commit with a version")]
pub struct PeekArgs {
    #[arg(long = "message", short = 'm')]
    pub message: String,
}

#[derive(Args, Debug)]
#[command(about = "Tag the current commit with a version")]
pub struct TagArgs {
    #[arg(long = "all", short = 'a')]
    pub all: bool,
}

#[derive(Subcommand, Debug)]
pub enum CCVerSubCommand {
    #[command(about = "Tag git with calculated version")]
    Tag(TagArgs),
    #[command(about = "Print the changelog")]
    ChangeLog,
    #[command(about = "Print the git format string")]
    GitFormat,
    Peek(PeekArgs),
}
