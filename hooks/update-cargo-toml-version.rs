use ccver::version_format::VersionFormat;
use eyre::Result;
use toml_edit::Document;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};
fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(std::fs::File::create("update-cargo-toml-version.log").unwrap()),
        )
        .init();
    let cwd = std::env::current_dir().unwrap();
    let git_path = cwd.join(".git");
    if !git_path.exists() {
        error!("Not a git repository");
        std::process::exit(1);
    }

    let commit_message = std::fs::read_to_string(git_path.join("COMMIT_EDITMSG")).unwrap();
    info!("Commit message: {}", commit_message);

    let version_format = VersionFormat::default();
    let next_version = ccver::peek(&cwd, &commit_message, &version_format)?;

    let next_version_string = next_version.to_string();
    let next_version_string = next_version_string.strip_prefix("v").unwrap();
    info!("Next version: {}", next_version_string);

    let cargo_toml_path = cwd.join("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path).unwrap();
    let binding = cargo_toml_content.parse::<Document<_>>().unwrap();
    let mut document = binding.into_mut();
    document["package"]["version"] = toml_edit::value(next_version_string);

    std::fs::write(&cargo_toml_path, document.to_string()).unwrap();

    Ok(())
}
