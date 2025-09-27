use ccver::version_format::{PreTagFormat, VersionFormat, VersionNumberFormat};
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

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        error!("Usage: {} <commit-message-file>", args[0]);
        std::process::exit(1);
    }

    let commit_message_file = &args[1];
    let commit_message = std::fs::read_to_string(commit_message_file).unwrap();
    info!("Commit message: {}", commit_message);

    let cwd = std::env::current_dir().unwrap();
    let next_version = ccver::peek(
        &cwd,
        &commit_message,
        &VersionFormat {
            v_prefix: false,
            major: VersionNumberFormat::CCVer,
            minor: VersionNumberFormat::CCVer,
            patch: VersionNumberFormat::CCVer,
            prerelease: Some(PreTagFormat::Build(VersionNumberFormat::CCVer)),
        },
    )?;

    let next_version_string = next_version.to_string();
    info!("Next version: {}", next_version_string);

    let cargo_toml_path = cwd.join("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path).unwrap();
    let binding = cargo_toml_content.parse::<Document<_>>().unwrap();
    let mut document = binding.into_mut();
    document["package"]["version"] = toml_edit::value(next_version_string);

    std::fs::write(&cargo_toml_path, document.to_string()).unwrap();

    Ok(())
}
