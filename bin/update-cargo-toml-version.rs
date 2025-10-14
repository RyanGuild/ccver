use std::env::current_dir;

use ccver::{
    git,
    graph::{MemoizedCommitGraph, version::ExistingVersionExt as _},
    logs::{Logs, PEEK_COMMIT_HASH, PeekLogEntry as _},
    parser,
    version::{PreTag, Version, VersionNumber},
    version_format::{PreTagFormat, VersionFormat, VersionNumberFormat},
};
use eyre::{OptionExt as _, Result};
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
    let logs = Logs::from_path(&cwd)?;

    let version_format = VersionFormat {
        v_prefix: false,
        major: VersionNumberFormat::CCVer,
        minor: VersionNumberFormat::CCVer,
        patch: VersionNumberFormat::CCVer,
        prerelease: Some(PreTagFormat::Build(VersionNumberFormat::CCVer)),
    };

    let graph = MemoizedCommitGraph::new(logs, &version_format);

    let (last_version, next_version) = if git::is_dirty(&current_dir().unwrap())? {
        // If the repo is dirty perform a graph peek into the next commit

        let parent_commit = graph.head().unwrap().lock().unwrap().log_entry.clone();
        let next_entry = commit_message
            .leak()
            .into_peek_log_entry(parent_commit.commit_hash, parent_commit.branch);
        let last_version = graph
            .head()
            .unwrap()
            .as_existing_version()
            .unwrap_or_else(|| version_format.as_default_version(&parent_commit).clone());
        let next_version = last_version.next_version(&next_entry, &version_format);

        if let Some(PreTag::ShortSha(VersionNumber::ShortSha(ref s))) = next_version.prerelease
            && s.as_ref() == &PEEK_COMMIT_HASH[0..7]
        {
            return Err(eyre::eyre!(
                "A short sha cannot be calculated before the commit is created; please make changes from a feature branch or use a conventional commit"
            ));
        };

        Ok::<(Version, Version), eyre::Error>((last_version, next_version))
    } else {
        // If the repo is clean; return the head version
        let head = graph.head().unwrap().lock().unwrap();
        let next_version = head.as_existing_version().unwrap();
        let cargo_toml_path = std::env::current_dir()?.join("Cargo.toml");
        let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)?;
        let cargo_toml_doc = cargo_toml_content.parse::<Document<_>>()?;
        let existing_version = parser::parse_version(
            cargo_toml_doc["package"]["version"]
                .as_str()
                .ok_or_eyre("Cargo.toml version not found")?,
            version_format.clone(),
        )?;
        Ok::<(Version, Version), eyre::Error>((existing_version, next_version))
    }?;

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
