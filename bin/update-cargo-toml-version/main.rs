use std::env::current_dir;

use ccver::{
    logs::PEEK_COMMIT_HASH,
    version::{PreTag, VersionNumber},
    version_format::{PreTagFormat, VersionFormat, VersionNumberFormat},
};
use eyre::Result;
use toml_edit::Document;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(std::fs::File::create("update-cargo-toml-version.log").unwrap()),
        )
        .init();

    let commit_message_file = current_dir().unwrap().join(".git/COMMIT_EDITMSG");
    debug!("commit_message_file: {}", commit_message_file.display());
    let commit_message = std::fs::read_to_string(commit_message_file).unwrap();
    info!("Commit message: {}", commit_message);

    let cwd = std::env::current_dir().unwrap();
    let (last_version, next_version) = ccver::peek(
        &cwd,
        commit_message,
        &VersionFormat {
            v_prefix: false,
            major: VersionNumberFormat::CCVer,
            minor: VersionNumberFormat::CCVer,
            patch: VersionNumberFormat::CCVer,
            prerelease: Some(PreTagFormat::Build(VersionNumberFormat::CCVer)),
        },
    )?;

    if let Some(PreTag::ShortSha(VersionNumber::ShortSha(ref s))) = next_version.prerelease {
        if s.eq(&PEEK_COMMIT_HASH[0..7]) {
            return Err(eyre::eyre!(
                "A short sha cannot be calculated before the commit is pushed; please make changes from a feature branch"
            ));
        };
    };

    let next_version_string = next_version.to_string();
    info!("Next version: {}", next_version_string);

    let cargo_toml_path = cwd.join("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path).unwrap();
    let binding = cargo_toml_content.parse::<Document<_>>().unwrap();
    let mut document = binding.into_mut();
    document["package"]["version"] = toml_edit::value(next_version_string.clone());

    std::fs::write(&cargo_toml_path, document.to_string()).unwrap();

    println!(
        "Updated Cargo.toml version to {}->{}",
        last_version, next_version_string
    );

    Ok(())
}
