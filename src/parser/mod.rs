pub mod cc;
pub use cc::{Parser, LogRawEntry, Rule};

use eyre::Result;
use std::process::Command;
use std::path::Path;
use pest_consume::Parser as _;

pub fn get_commit_logs(git_repo_path: &Path) -> Result<Vec<LogRawEntry>> {
    let logs = Command::new("git").args(&["log", "--format=format:%H%n%P%n%B"]).current_dir(git_repo_path).output().unwrap();
    let logs = String::from_utf8(logs.stdout).unwrap();
    let logs = Parser::parse(
        Rule::LOG_RAW, &logs
    )?
    .single()?;
    let parsed_logs = Parser::LOG_RAW(logs)?;
    Ok(parsed_logs)
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_commit_logs() {
        assert_ne!(get_commit_logs(Path::new(".")).unwrap().len(), 0);
    }
}
