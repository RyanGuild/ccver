use crate::logs::Logs;
use eyre::Result;

#[test]
fn test_self_git_log() -> Result<()> {
    let logs = Logs::default();
    println!("{logs:#?}");
    Ok(())
}
