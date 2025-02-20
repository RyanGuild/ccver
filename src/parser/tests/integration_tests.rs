use crate::logs::Logs;
use crate::parser::macros::cc_parse;
use eyre::Result;

#[test]
fn test_self_git_log() -> Result<()> {
    let log_str = Logs::default().get_raw();
    println!("{}", log_str);
    let logs = cc_parse!(CCVER_LOG, &log_str)?;
    assert!(logs.len() > 1);
    println!("{logs:#?}");
    Ok(())
}
