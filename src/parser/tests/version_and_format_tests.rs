use crate::{
    graph::commit_graph,
    parser::macros::{cc_parse_format, cc_parse_with_data},
    version::Version,
    version_format::VersionFormat,
};
use eyre::Result;
#[test]
fn test_parsing() -> Result<()> {
    let graph = commit_graph!();
    dbg!(cc_parse_format!(
        CCVER_VERSION_FORMAT,
        "vCC.CC.CC",
        graph.clone()
    )?);
    dbg!(cc_parse_format!(
        CCVER_VERSION_FORMAT,
        "vCC.CC.CC-rc.CC",
        graph.clone()
    )?);
    dbg!(cc_parse_format!(
        CCVER_VERSION_FORMAT,
        "YY.MM.DD-test.SS",
        graph.clone()
    )?);
    dbg!(cc_parse_format!(
        CCVER_VERSION_FORMAT,
        "YYYY.CC.CC-<sha>",
        graph.clone()
    )?);
    dbg!(cc_parse_format!(
        CCVER_VERSION_FORMAT,
        "vCC.CC.YYMMDD-beta.CC",
        graph.clone()
    )?);
    cc_parse_format!(CCVER_VERSION_FORMAT, "vCC.MM.DD-rc.CC", graph.clone())
        .expect_err("Day before month will not monotonically increase");

    cc_parse_format!(CCVER_VERSION_FORMAT, "vCC.MM.DD-rc.CC", graph.clone())
        .expect_err("CalVer format segments must be proceeded by a year segment to maintain semver monotonic incresing versions");
    Ok(())
}

#[test]
fn test_version_ordering() -> Result<()> {
    let default_config = VersionFormat::default();

    let build_1: Version = cc_parse_with_data!(CCVER_VERSION, "1.0.0", default_config.clone())?;
    let build_2: Version =
        cc_parse_with_data!(CCVER_VERSION, "1.0.0-rc.1", default_config.clone())?;
    let build_3: Version =
        cc_parse_with_data!(CCVER_VERSION, "1.0.0-beta.1", default_config.clone())?;
    let build_4: Version =
        cc_parse_with_data!(CCVER_VERSION, "1.0.0-alpha.1", default_config.clone())?;
    let build_5: Version =
        cc_parse_with_data!(CCVER_VERSION, "1.0.11-alpha.1", default_config.clone())?;
    let build_6: Version =
        cc_parse_with_data!(CCVER_VERSION, "1.2.1-build.2", default_config.clone())?;

    assert!(build_1 > build_2);
    assert!(build_2 > build_3);
    assert!(build_3 > build_4);
    assert!(build_5 > build_1);
    assert!(build_6 > build_5);

    Ok(())
}
