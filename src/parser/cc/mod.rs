use chrono::TimeZone;
use pest_consume::{match_nodes, Node, Parser as PP};
use std::collections::HashMap;

#[derive(PP)]
#[grammar = "parser/cc/rules.pest"]
pub struct Parser;

struct GitUserActionInfo {
    name: String,
    email: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    timezone: i64,
}

#[derive(Debug)]
struct LogRawEntry {
    commit_hash: String,
    parent_hash: Vec<String>,
    commit: Commit,
}

#[derive(Debug)]
pub struct ConventionalCommit {
    commit_type: String,
    breaking: bool,
    scope: Option<String>,
    description: String,
    body: Option<String>,
    footer: HashMap<String, String>,
}

#[derive(Debug)]
pub enum Commit {
    Conventional(ConventionalCommit),
    Text(String),
}

#[derive(Debug)]
struct CommitTag {
    commit_type: String,
    scope: Option<String>,
    breaking: bool,
}

#[pest_consume::parser]
impl Parser {
    fn HEADLINE(input: Node<Rule, ()>) -> Result<(CommitTag, String), pest_consume::Error<Rule>> {
        match_nodes!(input.children();
            [TAG(t), DESCRIPTION(d)] => Ok((t,d)),
        )
    }

    fn TAG(input: Node<Rule, ()>) -> Result<CommitTag, pest_consume::Error<Rule>> {
        match_nodes!(input.into_children();
            [TYPE(t), BREAKING_BANG(b)] => Ok(CommitTag { commit_type: t, scope: None, breaking: b }),
            [TYPE(t), SCOPE_SECTION(s), BREAKING_BANG(b)] => Ok(CommitTag { commit_type: t, scope: Some(s), breaking: b }),
        )
    }

    fn SCOPE_SECTION(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        match_nodes!(input.into_children();
            [SCOPE(s)] => Ok(s)
        )
    }

    fn TYPE(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn SCOPE(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn BREAKING_BANG(input: Node<Rule, ()>) -> Result<bool, pest_consume::Error<Rule>> {
        Ok(input.as_str() == "!")
    }

    fn DESCRIPTION(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn BODY(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn FOOTER(input: Node<Rule, ()>) -> Result<(String, String), pest_consume::Error<Rule>> {
        match_nodes!(input.children();
            [SCOPE(k),DESCRIPTION(v)] => Ok((k, v))
        )
    }

    fn CONVENTIONAL_COMMIT(
        input: Node<Rule, ()>,
    ) -> Result<ConventionalCommit, pest_consume::Error<Rule>> {
        match_nodes!(input.children();
            [HEADLINE((tag, description))] => {
                Ok(ConventionalCommit {
                    commit_type: tag.commit_type,
                    breaking: tag.breaking,
                    scope: tag.scope,
                    description: description,
                    body: None,
                    footer: HashMap::new(),
                })
            },
            [HEADLINE((tag, description)), BODY(b)] => {
                Ok(ConventionalCommit {
                    commit_type: tag.commit_type,
                    breaking: tag.breaking,
                    scope: tag.scope,
                    description: description,
                    body: Some(b),
                    footer: HashMap::new(),
                })
            },
            [HEADLINE((tag, description)), FOOTER(f)..] => {
                Ok(ConventionalCommit {
                    commit_type: tag.commit_type,
                    breaking: tag.breaking,
                    scope: tag.scope,
                    description: description,
                    body: None,
                    footer: f.collect(),
                })
            },
            [HEADLINE((tag, description)), BODY(b),FOOTER(f)..] => {
                Ok(ConventionalCommit {
                    commit_type: tag.commit_type,
                    breaking: tag.breaking,
                    scope: tag.scope,
                    description: description,
                    body: Some(b),
                    footer: f.collect(),
                })
            },
        )
    }

    fn COMMIT_HASHLINE(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        match_nodes!(input.children();
            [SHA(s)] => Ok(s)
        )
    }

    fn SHA(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn PARENT_HASHLINE(input: Node<Rule, ()>) -> Result<Vec<String>, pest_consume::Error<Rule>> {
        match_nodes!(input.children();
            [SHA(s)..] => Ok(s.collect())
        )
    }


    fn LOG_RAW_ENTRY(input: Node<Rule, ()>) -> Result<LogRawEntry, pest_consume::Error<Rule>> {
        match_nodes!(input.children();
        [COMMIT_HASHLINE(h), PARENT_HASHLINE(p), COMMIT(commit)] => Ok(LogRawEntry {
            commit_hash: h,
            parent_hash: p,
            commit: commit,
        }))
    }

    fn COMMIT(input: Node<Rule, ()>) -> Result<Commit, pest_consume::Error<Rule>> {
        match_nodes!(input.children();
            [CONVENTIONAL_COMMIT(c)] => Ok(Commit::Conventional(c)),
            [BODY(t)] => Ok(Commit::Text(t)),
        )
    }

    fn LOG_RAW(input: Node<Rule, ()>) -> Result<Vec<LogRawEntry>, pest_consume::Error<Rule>> {
        match_nodes!(input.children();
            [LOG_RAW_ENTRY(e)..] => Ok(e.collect())
        )
    }
}

#[cfg(test)]
mod parser_tests {
    use std::process::Command;

    use indoc::indoc;

    use super::*;

    #[test]
    fn test_type() -> Result<(), pest_consume::Error<Rule>> {
        let t = Parser::parse(Rule::TYPE, "feat").unwrap();
        assert_eq!(Parser::TYPE(t.single()?)?, "feat");
        Ok(())
    }

    #[test]
    fn test_scope() -> Result<(), pest_consume::Error<Rule>> {
        let scope = Parser::parse(Rule::SCOPE, "src/data.ts").unwrap();
        assert_eq!(Parser::SCOPE(scope.single()?)?, "src/data.ts");
        Ok(())
    }

    #[test]
    fn test_scope_section() -> Result<(), pest_consume::Error<Rule>> {
        let scope_section = Parser::parse(Rule::SCOPE_SECTION, "(src/data.ts)").unwrap();
        assert_eq!(
            Parser::SCOPE_SECTION(scope_section.single()?)?,
            "src/data.ts"
        );
        Ok(())
    }

    #[test]
    fn test_breaking_bang() -> Result<(), pest_consume::Error<Rule>> {
        let breaking_bang = Parser::parse(Rule::BREAKING_BANG, "!").unwrap();
        assert!(Parser::BREAKING_BANG(breaking_bang.single()?)?);
        Ok(())
    }

    #[test]
    fn test_tag_with_scope_and_breaking() -> Result<(), pest_consume::Error<Rule>> {
        let tag = Parser::parse(Rule::TAG, "feat(scope)!").unwrap();
        let parsed_tag = Parser::TAG(tag.single()?)?;
        assert_eq!(parsed_tag.commit_type, "feat");
        assert_eq!(parsed_tag.scope, Some("scope".to_string()));
        assert!(parsed_tag.breaking);
        Ok(())
    }

    #[test]
    fn test_commit() -> Result<(), pest_consume::Error<Rule>> {
        let headline = Parser::parse(Rule::CONVENTIONAL_COMMIT, "feat(scope)!: title").unwrap();
        let commit = Parser::CONVENTIONAL_COMMIT(headline.single()?)?;
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("scope".to_string()));
        assert!(commit.breaking);
        assert_eq!(commit.description, "title");
        Ok(())
    }

    #[test]
    fn test_headline() -> Result<(), pest_consume::Error<Rule>> {
        let headline = Parser::parse(Rule::HEADLINE, "feat(scope)!: title").unwrap();
        let (commit, description) = Parser::HEADLINE(headline.single()?)?;
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("scope".to_string()));
        assert!(commit.breaking);
        assert_eq!(description, "title");
        Ok(())
    }

    #[test]
    fn spaceing_test() -> Result<(), pest_consume::Error<Rule>> {
        let example = if let Commit::Conventional(example) = Parser::COMMIT(
            Parser::parse(
                Rule::COMMIT,
                indoc! {
                    "feat(src): add new feature

            Body Content
            :}

            Footer-Name: Footer Content
            Data: test2
            "},
            )?
            .single()?,
        )? {
            example
        } else {
            panic!("Expected ConventionalCommit");
        };

        let example2 = if let Commit::Conventional(example2) = Parser::COMMIT(
            Parser::parse(
                Rule::COMMIT,
                indoc! {
                    "feat(src): add new feature


            Body Content
            :}






            Footer-Name: Footer Content
            Data: test2
            "},
            )?
            .single()?,
        )? {
            example2
        } else {
            panic!("Expected ConventionalCommit")
        };

        assert_eq!(example.commit_type, example2.commit_type);
        assert_eq!(example.scope, example2.scope);
        assert_eq!(example.breaking, example2.breaking);
        assert_eq!(example.description, example2.description);
        assert_eq!(example.body, example2.body);
        assert_eq!(example.footer, example2.footer);
        assert_eq!(example.footer.len(), 2);
        assert_eq!(example2.footer.len(), 2);
        Ok(())
    }

    #[test]
    fn test_tag_raw_type() -> Result<(), pest_consume::Error<Rule>> {
        let t = Parser::parse(Rule::TYPE, "fix").unwrap();
        assert_eq!(Parser::TYPE(t.single()?)?, "fix");
        Ok(())
    }

    #[test]
    fn test_tag_type_with_bang() -> Result<(), pest_consume::Error<Rule>> {
        let tag = Parser::parse(Rule::TAG, "fix!").unwrap();
        let parsed_tag = Parser::TAG(tag.single()?)?;
        assert_eq!(parsed_tag.commit_type, "fix");
        assert!(parsed_tag.breaking);
        Ok(())
    }

    #[test]
    fn test_tag_with_scope() -> Result<(), pest_consume::Error<Rule>> {
        let tag = Parser::parse(Rule::TAG, "fix(README.md)").unwrap();
        let parsed_tag = Parser::TAG(tag.single()?)?;
        assert_eq!(parsed_tag.commit_type, "fix");
        assert_eq!(parsed_tag.scope, Some("README.md".to_string()));
        Ok(())
    }

    #[test]
    fn test_commit_hashline() -> Result<(), pest_consume::Error<Rule>> {
        let commit_hashline = Parser::parse(Rule::COMMIT_HASHLINE, "b008bebb2c3109e6720a9d7afcb1e654781668cb\n")?;
        assert_eq!(Parser::COMMIT_HASHLINE(commit_hashline.single()?)?, "b008bebb2c3109e6720a9d7afcb1e654781668cb");
        Ok(())
    }

    #[test]
    fn test_parent_hashline() -> Result<(), pest_consume::Error<Rule>> {
        let parent_hashline = Parser::parse(Rule::PARENT_HASHLINE, "38aa9cdf8228f03997d0e953d03cb00a2c1be536 38aa9cdf8228f03997d0e953d03cb00a2c1be536\n")?;
        assert_eq!(Parser::PARENT_HASHLINE(parent_hashline.single()?)?, vec!["38aa9cdf8228f03997d0e953d03cb00a2c1be536", "38aa9cdf8228f03997d0e953d03cb00a2c1be536"]);
        Ok(())
    }
    
    

    #[test]
    fn test_log_raw_entry() -> Result<(), pest_consume::Error<Rule>> {
        let log_raw_entry = Parser::parse(
            Rule::LOG_RAW_ENTRY,
            indoc! {
                "b008bebb2c3109e6720a9d7afcb1e654781668cb
                38aa9cdf8228f03997d0e953d03cb00a2c1be536
                fix(README.md): made the readme mean something
                "
            },
        )?;
        let parsed_log_raw_entry = Parser::LOG_RAW_ENTRY(log_raw_entry.single()?)?;

        assert_eq!(
            parsed_log_raw_entry.commit_hash,
            "b008bebb2c3109e6720a9d7afcb1e654781668cb"
        );
        assert_eq!(
            parsed_log_raw_entry.parent_hash,
            vec!["38aa9cdf8228f03997d0e953d03cb00a2c1be536"]
        );
        Ok(())
    }

    #[test]
    fn parse_real_logs() -> Result<(), pest_consume::Error<Rule>> {
        let logs = Command::new("git").args(&["log", "--format=format:%H%n%P%n%B"]).output().unwrap();
        let logs = String::from_utf8(logs.stdout).unwrap();
        let logs = Parser::parse(Rule::LOG_RAW, &logs)?;
        let parsed_logs = Parser::LOG_RAW(logs.single()?)?;
        println!("{:#?}", parsed_logs);
        Ok(())
    }
}