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
#[command(about = "Install ccver git hooks in a git repository")]
pub struct InstallArgs {}

#[derive(Args, Debug)]
#[command(about = "Tag the current commit with a version")]
pub struct TagArgs {}

#[derive(Subcommand, Debug)]
pub enum CCVerSubCommand {
    Init(InitArgs),
    Install(InstallArgs),
    Tag(TagArgs),
    ChangeLog,
}
