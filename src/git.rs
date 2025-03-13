use crate::logs::GIT_FORMAT_ARGS;
use eyre::*;
use std::{path::Path, process::Command};

pub fn is_dirty(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["diff", "--exit-code"])
        .current_dir(path)
        .output()?;

    let status = output.status;

    let code = status
        .code()
        .ok_or_eyre("could not get status code from git diff")?;

    if code != 0 {
        Err(eyre!(String::from_utf8(output.stdout)?))
    } else {
        Ok(())
    }
}

pub fn git_installed() -> Result<()> {
    Ok(Command::new("git")
        .arg("--version")
        .output()
        .map(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(eyre!("git is not installed; invalid status code"))
            }
        })?)
    .flatten()
}

pub fn formatted_logs(path: &Path) -> Result<&'static mut str> {
    Ok(String::from_utf8(
        Command::new("git")
            .args(GIT_FORMAT_ARGS)
            .current_dir(path)
            .output()?
            .stdout,
    )?
    .leak())
}

#[cfg(test)]
mod test_commands {
    #[test]
    fn test_git_installed() {
        // I guess this is an assumption that git is installed on the system that runs the tests but how else would you get the tests
        assert_eq!(super::git_installed().is_ok(), true);
    }

    #[test]
    fn test_is_dirty() -> eyre::Result<()> {
        let dirty = super::is_dirty(std::path::Path::new("."))?;
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
}
