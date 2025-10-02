use crate::{
    logs::GIT_FORMAT_ARGS,
    version::Version,
    version_format::{self, VersionFormat},
};
use core::hash;
use eyre::*;
use std::{path::Path, process::Command};
use tracing::{debug, info, instrument, warn};

#[instrument]
pub fn is_dirty(path: &Path) -> Result<bool> {
    debug!("Checking if repository is dirty at path: {:?}", path);
    let output = Command::new("git")
        .args(["diff", "--exit-code"])
        .current_dir(path)
        .output()?;

    let status = output.status;

    let code = status
        .code()
        .ok_or_eyre("could not get status code from git diff")?;

    let is_dirty = code != 0;
    debug!("Repository dirty status: {}", is_dirty);
    Ok(is_dirty)
}

#[instrument]
pub fn tag_commit_with_version(hash: &str, version: &Version, path: &Path) -> Result<()> {
    debug!("Tagging commit with version: {}", version);
    let tag = format!("{}", version);
    Command::new("git")
        .args(["tag", tag.as_str(), hash])
        .current_dir(path)
        .output()?;
    Ok(())
}

#[instrument]
pub fn commit_hash(path: &Path, message: &str) -> Result<String> {
    debug!("Creating commit hash for message: {}", message);
    let tree_hash = tree_hash(path)?;
    let hash = String::from_utf8(
        Command::new("git")
            .args([
                "commit-tree",
                tree_hash.as_str(),
                "-p",
                "HEAD",
                "-m",
                message,
            ])
            .current_dir(path)
            .output()?
            .stdout,
    )?
    .trim()
    .to_string();
    debug!("Generated commit hash: {}", hash);
    Ok(hash)
}

#[instrument]
pub fn current_branch(path: &Path) -> Result<String> {
    debug!("Getting current branch name");
    let branch = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()?
            .stdout,
    )?
    .trim()
    .to_string();
    debug!("Current branch: {}", branch);
    Ok(branch)
}

#[instrument]
pub fn author_name(path: &Path) -> Result<String> {
    debug!("Getting git author name");
    let name = String::from_utf8(
        Command::new("git")
            .args(["config", "user.name"])
            .current_dir(path)
            .output()?
            .stdout,
    )?;
    debug!(name = %name.trim(), "Retrieved author name");
    Ok(name)
}

#[instrument]
pub fn author_email(path: &Path) -> Result<String> {
    debug!("Getting git author email");
    let email = String::from_utf8(
        Command::new("git")
            .args(["config", "user.email"])
            .current_dir(path)
            .output()?
            .stdout,
    )?;
    debug!(email = %email.trim(), "Retrieved author email");
    Ok(email)
}

#[instrument]
pub fn head_hash(path: &Path) -> Result<String> {
    debug!("Getting HEAD hash");
    let hash = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(path)
            .output()?
            .stdout,
    )?;
    debug!(hash = %hash.trim(), "Retrieved HEAD hash");
    Ok(hash)
}

#[instrument]
pub fn tree_hash(path: &Path) -> Result<String> {
    debug!("Getting tree hash");
    let hash = String::from_utf8(
        Command::new("git")
            .args(["write-tree"])
            .current_dir(path)
            .output()?
            .stdout,
    )?
    .trim()
    .to_string();
    debug!(tree_hash = %hash, "Retrieved tree hash");
    Ok(hash)
}

#[instrument]
pub fn git_installed() -> Result<()> {
    debug!("Checking if git is installed");
    let output = Command::new("git").arg("--version").output()?;

    if output.status.success() {
        info!("Git is installed and available");
        Ok(())
    } else {
        warn!("Git command failed with non-zero exit code");
        Err(eyre!("git is not installed; invalid status code"))
    }
}

#[instrument]
pub fn formatted_logs(path: &Path) -> Result<&'static mut str> {
    let start = std::time::Instant::now();
    info!("Fetching formatted git logs");
    debug!("Git format args: {:?}", GIT_FORMAT_ARGS);

    let logs = String::from_utf8(
        Command::new("git")
            .args(GIT_FORMAT_ARGS)
            .current_dir(path)
            .output()?
            .stdout,
    )?
    .leak();

    let duration = start.elapsed();
    info!(
        duration_ms = duration.as_millis(),
        log_size = logs.len(),
        "Retrieved git log data"
    );
    debug!(
        "Retrieved {} characters of log data in {:?}",
        logs.len(),
        duration
    );
    Ok(logs)
}

#[cfg(test)]
mod test_commands {
    use std::env::current_dir;

    #[test]
    fn test_git_installed() {
        // I guess this is an assumption that git is installed on the system that runs the tests but how else would you get the tests
        assert_eq!(super::git_installed().is_ok(), true);
    }

    #[test]
    fn test_is_dirty() -> eyre::Result<()> {
        let dirty = super::is_dirty(&current_dir().unwrap())?;
        dbg!(dirty);
        Ok(())
    }

    #[test]
    fn test_formatted_logs() -> eyre::Result<()> {
        let logs = super::formatted_logs(std::path::Path::new("."))?;
        // this is a random number but enough to catch significant changes
        assert!(logs.split('\n').count() > 50);
        Ok(())
    }

    #[test]
    fn tree_hash_exists() -> eyre::Result<()> {
        let tree_hash = super::tree_hash(&current_dir().unwrap())?;
        assert!(!tree_hash.is_empty());
        println!("tree_hash: {:?}", tree_hash);
        Ok(())
    }

    #[test]
    fn commit_hash_exists() -> eyre::Result<()> {
        let commit_hash = super::commit_hash(&current_dir().unwrap(), "test")?;
        assert!(!commit_hash.is_empty());
        println!("commit_hash: {:?}", commit_hash);
        Ok(())
    }

    #[test]
    fn current_branch_exists() -> eyre::Result<()> {
        let current_branch = super::current_branch(std::path::Path::new("."))?;
        assert!(!current_branch.is_empty());
        Ok(())
    }
}
