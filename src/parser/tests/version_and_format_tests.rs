use crate::parser::{
    macros::{cc_parse, cc_parse_with_data},
    LexerInput,
};
use eyre::Result;
#[test]
fn test_parsing() -> Result<()> {
    dbg!(cc_parse!(CCVER_VERSION_FORMAT, "vCC.CC.CC")?);
    dbg!(cc_parse!(CCVER_VERSION_FORMAT, "vCC.CC.CC-rc.CC")?);
    dbg!(cc_parse!(CCVER_VERSION_FORMAT, "YY.MM.DD-test.SS")?);
    dbg!(cc_parse!(CCVER_VERSION_FORMAT, "YYYY.CC.CC-<sha>")?);
    dbg!(cc_parse!(CCVER_VERSION_FORMAT, "vCC.CC.YYMMDD-beta.CC")?);
    cc_parse!(CCVER_VERSION_FORMAT, "vCC.CC.YYDDMM-beta.CC")
        .expect_err("Day before month will not monotonically increase");

    cc_parse!(CCVER_VERSION_FORMAT, "vCC.MM.DD-rc.CC")
        .expect_err("CalVer format segments must be proceeded by a year segment to maintain semver monotonic incresing versions");
    Ok(())
}

#[test]
fn test_version_ordering() -> Result<()> {
    let default_config = LexerInput::default();

    let build_1 = cc_parse_with_data!(CCVER_VERSION, "1.0.0", default_config.clone())?;
    let build_2 = cc_parse_with_data!(CCVER_VERSION, "1.0.0-rc.1", default_config.clone())?;
    let build_3 = cc_parse_with_data!(CCVER_VERSION, "1.0.0-beta.1", default_config.clone())?;
    let build_4 = cc_parse_with_data!(CCVER_VERSION, "1.0.0-alpha.1", default_config.clone())?;
    let build_5 = cc_parse_with_data!(CCVER_VERSION, "1.0.11-alpha.1", default_config.clone())?;
    let build_6 = cc_parse_with_data!(CCVER_VERSION, "1.2.1-build.2", default_config.clone())?;

    assert!(build_1 > build_2);
    assert!(build_2 > build_3);
    assert!(build_3 > build_4);
    assert!(build_5 > build_1);
    assert!(build_6 > build_5);

    Ok(())
}
