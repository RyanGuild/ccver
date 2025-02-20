use indoc::indoc;

use crate::logs::Tag;
use crate::parser::macros::cc_parse;
use crate::parser::InterpreterResult;
use crate::version::{Version, VersionNumber};

#[test]
fn test_footers() -> InterpreterResult<()> {
    let footer = indoc! {
        "Footer-Key: Footer Value"
    };

    let (k, v) = cc_parse!(FOOTER, footer)?;
    assert_eq!(k, "Footer-Key");
    assert_eq!(v, "Footer Value");

    let footers = indoc! {"
        Footer1: val1
        footer-2: bar
        "};

    let footers_map = cc_parse!(FOOTER_SECTION, footers)?;
    println!("{footers_map:#?}");
    assert_eq!(footers_map.len(), 2);

    Ok(())
}

#[test]
fn parse_macro() -> InterpreterResult<()> {
    let _commit = cc_parse!(CONVENTIONAL_SUBJECT, "test(anotha)!: build")?;

    Ok(())
}

#[test]
fn test_type() -> InterpreterResult<()> {
    let t = cc_parse!(TYPE, "feat")?;
    assert_eq!(t, "feat");
    Ok(())
}

#[test]
fn test_scope() -> InterpreterResult<()> {
    let scope = cc_parse!(SCOPE, "src/data.ts")?;
    assert_eq!(scope, "src/data.ts");
    Ok(())
}

#[test]
fn test_scope_section() -> InterpreterResult<()> {
    let scope_section = cc_parse!(SCOPE_SECTION, "(src/data.ts)")?;
    assert_eq!(scope_section, "src/data.ts");
    Ok(())
}

#[test]
fn test_breaking_bang() -> InterpreterResult<()> {
    let breaking_bang = cc_parse!(BREAKING_BANG, "!")?;
    assert!(breaking_bang);
    Ok(())
}

#[test]
fn test_tag_with_scope_and_breaking() -> InterpreterResult<()> {
    let (commit_type, scope, breaking) = cc_parse!(TAG, "feat(scope)!")?;
    assert_eq!(commit_type, "feat");
    assert_eq!(scope, Some("scope"));
    assert!(breaking);
    Ok(())
}

#[test]
fn test_commit() -> InterpreterResult<()> {
    let commit = cc_parse!(CONVENTIONAL_SUBJECT, "feat(scope)!: title")?;
    assert_eq!(commit.commit_type, "feat");
    assert_eq!(commit.scope, Some("scope"));
    assert!(commit.breaking);
    assert_eq!(commit.description, "title");
    Ok(())
}

#[test]
fn test_headline() -> InterpreterResult<()> {
    let commit = cc_parse!(CONVENTIONAL_SUBJECT, "feat(scope)!: title")?;
    assert_eq!(commit.commit_type, "feat");
    assert_eq!(commit.scope, Some("scope"));
    assert!(commit.breaking);
    assert_eq!(commit.description, "title");
    Ok(())
}

#[test]
fn test_tag_raw_type() -> InterpreterResult<()> {
    let parsed_tag = cc_parse!(TYPE, "fix")?;
    assert_eq!(parsed_tag, "fix");
    Ok(())
}

#[test]
fn test_tag_type_with_bang() -> InterpreterResult<()> {
    let (commit_type, _, breaking) = cc_parse!(TAG, "fix!")?;
    assert_eq!(commit_type, "fix");
    assert!(breaking);
    Ok(())
}

#[test]
fn test_tag_with_scope() -> InterpreterResult<()> {
    let (commit_type, scope, _) = cc_parse!(TAG, "fix(README.md)")?;
    assert_eq!(commit_type, "fix");
    assert_eq!(scope, Some("README.md"));
    Ok(())
}

#[test]
fn test_commit_hashline() -> InterpreterResult<()> {
    let commit_hashline = cc_parse!(
        COMMIT_HASHLINE,
        "b008bebb2c3109e6720a9d7afcb1e654781668cb\n"
    )?;
    assert_eq!(commit_hashline, "b008bebb2c3109e6720a9d7afcb1e654781668cb");
    Ok(())
}

#[test]
fn test_parent_hashline() -> InterpreterResult<()> {
    let parent_hashline = cc_parse!(
        PARENT_HASHLINE,
        "38aa9cdf8228f03997d0e953d03cb00a2c1be536 38aa9cdf8228f03997d0e953d03cb00a2c1be536\n"
    )?;
    assert_eq!(
        *parent_hashline,
        [
            "38aa9cdf8228f03997d0e953d03cb00a2c1be536",
            "38aa9cdf8228f03997d0e953d03cb00a2c1be536"
        ]
    );
    Ok(())
}

#[test]
fn test_head_dec() -> InterpreterResult<()> {
    let head_dec = cc_parse!(HEAD_DEC, "HEAD -> master")?;
    println!("head_dec: {}", head_dec);
    Ok(())
}

#[test]
fn test_tag_dec() -> InterpreterResult<()> {
    let head_dec = cc_parse!(TAG_DEC, "tag: v0.1.1")?;
    assert_eq!(
        head_dec,
        Tag::Version(Version {
            major: VersionNumber::CCVer(0),
            minor: VersionNumber::CCVer(1),
            patch: VersionNumber::CCVer(1),
            prerelease: None
        })
    );
    Ok(())
}
