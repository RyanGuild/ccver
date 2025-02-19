#[allow(unused_macros)]
macro_rules! cc_parse {
    ($rule:ident, $str_ref:expr) => {
        match Parser::parse(Rule::$rule, $str_ref) {
            Err(e) => Err(e),
            Ok(parsed) => match parsed.single() {
                Err(e) => Err(e),
                Ok(single) => match Parser::$rule(single) {
                    Err(e) => Err(e),
                    Ok(hydrated) => Ok(hydrated)
                },
            },
        }
    };
}

use std::{collections::HashMap, rc::Rc};

use eyre::Result;
use pest_consume::{match_nodes, Node, Parser as PP};

use crate::version::{PreTag, Version, VersionNumber};

#[derive(PP)]
#[grammar = "parser/rules.pest"]
pub struct Parser;

#[derive(Debug)]
pub enum Decoration<'a> {
    HeadIndicator(&'a str),
    Tag(&'a str),
    RemoteBranch((&'a str, &'a str)),
    Branch(&'a str),
}


#[derive(Debug)]
pub struct ConventionalSubject<'a> {
    pub commit_type: &'a str,
    pub breaking: bool,
    pub scope: Option<&'a str>,
    pub description: &'a str,
}

#[derive(Debug)]
pub enum Subject<'a> {
    Conventional(ConventionalSubject<'a>),
    Text(&'a str),
}

#[derive(Debug)]
pub struct CCVerLogEntryData<'a> {
    pub name: &'a str,
    pub commit_hash: &'a str,
    pub commit_timezone: chrono::Utc,
    pub commit_datetime: chrono::DateTime<chrono::Utc>,
    pub parent_hashes: Rc<[&'a str]>,
    pub decorations: Rc<[Decoration<'a>]>,
    pub subject: Subject<'a>,
    pub footers: std::collections::HashMap<&'a str, &'a str>,
}

pub type CCVerLogEntry<'a> = Rc<CCVerLogEntryData<'a>>;

pub type CCVerLog<'a> = Rc<[CCVerLogEntry<'a>]>;

pub type ParserResult<T> = eyre::Result<T, pest_consume::Error<Rule>>;

#[pest_consume::parser]
impl Parser {
    pub fn CCVER(input: Node<Rule, ()>) -> ParserResult<Version> {
        match_nodes!(input.children();
            [VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch)] => Ok(Version {
                major,
                minor,
                patch,
                prerelease: None
            }),
            [VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), SHA(sha)] => Ok(Version {
                major,
                minor,
                patch,
                prerelease: Some(PreTag::Sha(sha.to_string()))
            }),
            [VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), RC_PRE_TAG(prerelease)] => Ok(Version {
                major,
                minor,
                patch,
                prerelease: Some(prerelease)
            }),
            [VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), BETA_PRE_TAG(prerelease)] => Ok(Version {
                major,
                minor,
                patch,
                prerelease: Some(prerelease)
            }),
            [VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), ALPHA_PRE_TAG(prerelease)] => Ok(Version {
                major,
                minor,
                patch,
                prerelease: Some(prerelease)
            }),
            [VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), BUILD_PRE_TAG(prerelease)] => Ok(Version {
                major,
                minor,
                patch,
                prerelease: Some(prerelease)
            }),
            [VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), NAMED_PRE_TAG(prerelease)] => Ok(Version {
                major,
                minor,
                patch,
                prerelease: Some(prerelease)
            })
        )
    }

    fn VERSION_NUMBER(input: Node<Rule, ()>) -> ParserResult<VersionNumber> {
        todo!()
    }

    fn RC_PRE_TAG(input: Node<Rule, ()>) -> ParserResult<PreTag> {
        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Rc(v))
        )
    }

    fn BETA_PRE_TAG(input: Node<Rule, ()>) -> ParserResult<PreTag> {
        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Beta(v))
        )
    }

    fn ALPHA_PRE_TAG(input: Node<Rule, ()>) -> ParserResult<PreTag> {
        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Alpha(v))
        )
    }

    fn BUILD_PRE_TAG(input: Node<Rule, ()>) -> ParserResult<PreTag> {
        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Build(v))
        )
    }

    fn NAMED_PRE_TAG(input: Node<Rule, ()>) -> ParserResult<PreTag> {
        match_nodes!(input.children();
            [NAME(n), VERSION_NUMBER(v)] => Ok(PreTag::Named(n.to_string(),v))
        )
    }

    fn NAME(input: Node<Rule, ()>) -> ParserResult<&str> {
        Ok(input.as_str())
    }


    fn CCVER_LOG_ENTRY(input: Node<Rule, ()>) -> ParserResult<CCVerLogEntry> {
        match_nodes!(input.children();
            [
                SCOPE(name), 
                COMMIT_HASHLINE(commit_hash), 
                ISO8601_DATE(commit_datetime), 
                DECORATIONS_LINE(decorations), 
                PARENT_HASHLINE(parents), 
                SUBJECT(subject),
                FOOTER_SECTION(footers), 

            ] => {
                Ok(
                   Rc::new( CCVerLogEntryData {
                        name,
                        commit_hash,
                        commit_datetime,
                        commit_timezone: commit_datetime.timezone(),
                        parent_hashes: parents,
                        footers,
                        decorations,
                        subject
                    })
                )
            },
            [
                SCOPE(name), 
                COMMIT_HASHLINE(commit_hash), 
                ISO8601_DATE(commit_datetime), 
                PARENT_HASHLINE(parents), 
                SUBJECT(subject),
                FOOTER_SECTION(footers), 

            ] => {
                Ok(
                    Rc::new(CCVerLogEntryData {
                        name,
                        commit_hash,
                        commit_datetime,
                        commit_timezone: commit_datetime.timezone(),
                        parent_hashes: parents,
                        footers,
                        decorations: Rc::new([]),
                        subject
                    })
                )
            }
        )
    }

    fn ISO8601_DATE(
        input: Node<Rule, ()>,
    ) -> ParserResult<chrono::DateTime<chrono::Utc>> {
        Ok(chrono::DateTime::parse_from_rfc3339(input.as_str())
            .unwrap()
            .to_utc())
    }

    fn EOI(input: Node<Rule, ()>) -> ParserResult<()> {
        Ok(())
    }

    fn TAG(input: Node<Rule, ()>) -> ParserResult<(&str, Option<&str>, bool)> {
        match_nodes!(input.into_children();
            [TYPE(t), BREAKING_BANG(b)] => Ok((t, None, b)),
            [TYPE(t), SCOPE_SECTION(s), BREAKING_BANG(b)] => Ok((t, Some(s), b)),
        )
    }

    fn SCOPE_SECTION(input: Node<Rule, ()>) -> ParserResult<&str> {
        match_nodes!(input.into_children();
            [SCOPE(s)] => Ok(s)
        )
    }

    fn TYPE(input: Node<Rule, ()>) -> ParserResult<&str> {
        Ok(input.as_str())
    }

    fn SCOPE(input: Node<Rule, ()>) -> ParserResult<&str> {
        Ok(input.as_str())
    }

    fn BREAKING_BANG(input: Node<Rule, ()>) -> ParserResult<bool> {
        Ok(input.as_str() == "!")
    }

    fn DESCRIPTION(input: Node<Rule, ()>) -> ParserResult<&str> {
        Ok(input.as_str())
    }

    fn BODY(input: Node<Rule, ()>) -> ParserResult<&str> {
        Ok(input.as_str().trim())
    }

    fn BODY_SECTION(input: Node<Rule, ()>) -> ParserResult<&str> {
        match_nodes!(input.children();
            [BODY(b)] => Ok(b)
        )
    }

    fn FOOTER(input: Node<Rule, ()>) -> ParserResult<(&str, &str)> {
        match_nodes!(input.children();
            [SCOPE(k),DESCRIPTION(v)] => Ok((k, v))
        )
    }

    fn FOOTER_SECTION(
        input: Node<Rule, ()>,
    ) -> ParserResult<HashMap<&str, &str>> {
        if input.children().count() == 0 {
            return Ok(HashMap::new());
        } else {
            Ok(input.children().map(Parser::FOOTER).map(|f| f.unwrap()).collect())
        }
    }

    fn COMMIT_HASHLINE(input: Node<Rule, ()>) -> ParserResult<&str> {
        match_nodes!(input.children();
            [SHA(s)] => Ok(s)
        )
    }

    fn SHA(input: Node<Rule, ()>) -> ParserResult<&str> {
        Ok(input.as_str())
    }

    fn PARENT_HASHLINE(input: Node<Rule, ()>) -> ParserResult<Rc<[&str]>> {
        match_nodes!(input.children();
            [SHA(s)..] => Ok(s.collect())
        )
    }

    pub fn CONVENTIONAL_SUBJECT(
        input: Node<Rule, ()>,
    ) -> ParserResult<ConventionalSubject> {
        match_nodes!(input.children();
            [TAG((commit_type, scope, breaking)), DESCRIPTION(description)] => Ok(
                ConventionalSubject {
                    breaking,
                    commit_type,
                    description,
                    scope,
                }
            )
        )
    }

    pub fn SUBJECT(input: Node<Rule, ()>) -> ParserResult<Subject> {
        match_nodes!(input.children();
            [CONVENTIONAL_SUBJECT(s)] => Ok(Subject::Conventional(s)),
            [DESCRIPTION(t)] => Ok(Subject::Text(t)),
        )
    }
    pub fn HEAD_DEC(input: Node<Rule, ()>) -> ParserResult<&str> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn TAG_DEC(input: Node<Rule, ()>) -> ParserResult<&str> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn BRANCH_DEC(input: Node<Rule, ()>) -> ParserResult<&str> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn FNAME(input: Node<Rule, ()>) -> ParserResult<&str> {
        Ok(input.as_str())
    }

    pub fn REMOTE_DEC(input: Node<Rule, ()>) -> ParserResult<(&str, &str)> {
        match_nodes!(input.children();
            [FNAME(o), SCOPE(s)] => Ok((o,s))
        )
    }

    pub fn DECORATION(
        input: Node<Rule, ()>,
    ) -> ParserResult<Decoration> {
        match_nodes!(input.children();
            [HEAD_DEC(h)] =>Ok(Decoration::HeadIndicator(h)),
            [TAG_DEC(t)] => Ok(Decoration::Tag(t)),
            [REMOTE_DEC((o,s))] => Ok(Decoration::RemoteBranch((o,s))),
            [BRANCH_DEC(b)] => Ok(Decoration::Branch(b))
        )
    }

    pub fn DECORATIONS_LINE(
        input: Node<Rule, ()>,
    ) -> ParserResult<Rc<[Decoration]>> {
        if input.children().count() == 0 {
            return Ok(Rc::new([]));
        }
        else {
            input.children().map(Parser::DECORATION).collect()
        }
    }

    pub fn CCVER_LOG(input: Node<Rule, ()>) -> ParserResult<CCVerLog> {
        match_nodes!(input.children();
            [CCVER_LOG_ENTRY(e).., EOI(_)] => Ok(e.collect())
        )
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use eyre::Result;
    use std::process::Command;
    #[test]
    fn test_self_git_log() -> Result<()> {
        let log_str = String::from_utf8(
            Command::new("git")
                .args(["log", "--format=name=%n%f%ncommit=%n%H%ncommit-time=%n%cI%ndec=%n%d%nparent=%n%P%nsub=%n%s%ntrailers=%n%(trailers:only)%n"])
                .output()?
                .stdout,
        )?;
        println!("{}",log_str);
        let logs = cc_parse!(CCVER_LOG, &log_str)?;
        assert!(logs.len() > 1);
        println!("{logs:#?}");
        Ok(())
    }
    #[test]
    fn test_self_git_log_single() -> Result<()> {
        let log_str = String::from_utf8(
            Command::new("git")
                .args(["log", "-1", "--format=name=%n%f%ncommit=%n%H%ncommit-time=%n%cI%ndec=%n%d%nparent=%n%P%nsub=%n%s%ntrailers=%n%(trailers:only)%n"])
                .output()?
                .stdout,
        )?;

        let nodes = Parser::parse(Rule::CCVER_LOG_ENTRY, &log_str)?.single()?.children();
        println!("{nodes:#?}");
        let logs = cc_parse!(CCVER_LOG_ENTRY, &log_str)?;
        println!("{logs:#?}");
        Ok(())
    }
}

#[cfg(test)]
mod syntax_tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_footers() -> ParserResult<()> {
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
    fn parse_macro() -> ParserResult<()> {
        let commit = cc_parse!(CONVENTIONAL_SUBJECT, "test(anotha)!: build")?;

        Ok(())
    }

    #[test]
    fn test_type() -> ParserResult<()> {
        let t = cc_parse!(TYPE, "feat")?;
        assert_eq!(t, "feat");
        Ok(())
    }

    #[test]
    fn test_scope() -> ParserResult<()> {
        let scope = cc_parse!(SCOPE, "src/data.ts")?;
        assert_eq!(scope, "src/data.ts");
        Ok(())
    }

    #[test]
    fn test_scope_section() -> ParserResult<()> {
        let scope_section = cc_parse!(SCOPE_SECTION, "(src/data.ts)")?;
        assert_eq!(scope_section, "src/data.ts");
        Ok(())
    }

    #[test]
    fn test_breaking_bang() -> ParserResult<()> {
        let breaking_bang = cc_parse!(BREAKING_BANG, "!")?;
        assert!(breaking_bang);
        Ok(())
    }

    #[test]
    fn test_tag_with_scope_and_breaking() -> ParserResult<()> {
        let (commit_type, scope, breaking) = cc_parse!(TAG, "feat(scope)!")?;
        assert_eq!(commit_type, "feat");
        assert_eq!(scope, Some("scope"));
        assert!(breaking);
        Ok(())
    }

    #[test]
    fn test_commit() -> ParserResult<()> {
        let commit = cc_parse!(CONVENTIONAL_SUBJECT, "feat(scope)!: title")?;
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("scope"));
        assert!(commit.breaking);
        assert_eq!(commit.description, "title");
        Ok(())
    }

    #[test]
    fn test_headline() -> ParserResult<()> {
        let commit = cc_parse!(CONVENTIONAL_SUBJECT, "feat(scope)!: title")?;
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("scope"));
        assert!(commit.breaking);
        assert_eq!(commit.description, "title");
        Ok(())
    }

    #[test]
    fn test_tag_raw_type() -> ParserResult<()> {
        let parsed_tag = cc_parse!(TYPE, "fix")?;
        assert_eq!(parsed_tag, "fix");
        Ok(())
    }

    #[test]
    fn test_tag_type_with_bang() -> ParserResult<()> {
        let (commit_type, _, breaking) = cc_parse!(TAG, "fix!")?;
        assert_eq!(commit_type, "fix");
        assert!(breaking);
        Ok(())
    }

    #[test]
    fn test_tag_with_scope() -> ParserResult<()> {
        let (commit_type, scope, _) = cc_parse!(TAG, "fix(README.md)")?;
        assert_eq!(commit_type, "fix");
        assert_eq!(scope, Some("README.md"));
        Ok(())
    }

    #[test]
    fn test_commit_hashline() -> ParserResult<()> {
        let commit_hashline = cc_parse!(
            COMMIT_HASHLINE,
            "b008bebb2c3109e6720a9d7afcb1e654781668cb\n"
        )?;
        assert_eq!(commit_hashline, "b008bebb2c3109e6720a9d7afcb1e654781668cb");
        Ok(())
    }

    #[test]
    fn test_parent_hashline() -> ParserResult<()> {
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
    fn test_head_dec() -> ParserResult<()> {
        let head_dec = cc_parse!(HEAD_DEC, "HEAD -> master")?;
        println!("head_dec: {}", head_dec);
        Ok(())
    }

    #[test]
    fn test_tag_dec() -> ParserResult<()> {
        let head_dec = cc_parse!(TAG_DEC, "tag: v0.1.1")?;
        assert_eq!(head_dec, "v0.1.1");
        Ok(())
    }
}



pub fn parse(log:  &str) -> ParserResult<CCVerLog> {
    cc_parse!(CCVER_LOG, log)
}