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
#[command(about = "Initialize ccver in a git repository")]
pub struct InitArgs {
    #[arg(long = "rewrite", short = 'r')]
    pub rewrite: bool,

    #[arg(long = "install", short = 'i')]
    pub install_hooks: bool,
}

#[derive(Args, Debug)]
#[command(about = "Tag the current commit with a version")]
pub struct PeekArgs {
    #[arg(long = "message", short = 'm')]
    pub message: String,
}

#[derive(Subcommand, Debug)]
pub enum CCVerSubCommand {
    Init(InitArgs),
    Install,
    Tag,
    #[command(about = "Print the changelog")]
    ChangeLog,
    #[command(about = "Print the git format string")]
    GitFormat,
    Peek(PeekArgs),
}

impl From<&str> for CCVerSubCommand {
    fn from(command: &str) -> Self {
        match command {
            "install" => CCVerSubCommand::Install,
            "tag" => CCVerSubCommand::Tag,
            "changelog" => CCVerSubCommand::ChangeLog,
            "git-format" => CCVerSubCommand::GitFormat,
            _ => panic!("Invalid command: {}", command),
        }
    }
}
