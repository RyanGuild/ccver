use chrono::DateTime;
use itertools::Itertools as _;
use pest::Parser as _;
use pest_derive::Parser as PP;

#[path = "../flat_tag_map.rs"]
mod ftm;
use ftm::flat_tag_map;

#[derive(PP)]
#[grammar = "parser/git/rules.pest"]
pub struct Parser;

#[derive(Debug)]
#[allow(dead_code)]
pub struct LogEntry {
    commit_sha: String,
    tree_sha: Option<String>,
    parent_shas: Option<String>,
    author_name: String,
    author_email: String,
    date: DateTime<chrono::Utc>,
    message: String,
}

pub fn parse_git_log(log: &str) -> Vec<LogEntry> {
    Parser::parse(Rule::LOG, &log)
        .expect("parsing git log failed")
        .next()
        .expect("pulling parser failed")
        .into_inner()
        .map(|entry| match entry.as_rule() {
            Rule::LOG_ENTRY => {
                let mut tags = flat_tag_map(entry.into_inner());
                Some(LogEntry {
                    commit_sha: tags.remove("commit_sha").unwrap(),
                    tree_sha: tags.remove("tree_sha"),
                    parent_shas: tags.remove("parent_shas"),
                    author_name: tags.remove("author_name").unwrap(),
                    author_email: tags.remove("author_email").unwrap(),
                    date: chrono::DateTime::parse_from_str(
                        &tags.remove("date_string").unwrap(),
                        "%a %b %e %T %Y %z",
                    )
                    .expect("cannot parse date")
                    .to_utc(),
                    message: tags.remove("message").unwrap(),
                })
            }
            _ => None,
        })
        .while_some()
        .collect()
}

#[cfg(test)]
mod parser_tests {
    use pest::Parser as _;

    use super::*;

    use std::process::Command;

    #[test]
    fn parse_real_logs() {
        let log =
            String::from_utf8(Command::new("git").arg("log").output().unwrap().stdout).unwrap();
        let parsed = parse_git_log(&log);
        println!("{:#?}", parsed);
    }

    #[test]
    fn parse_tests() {
        Parser::parse(Rule::SHA, "1234567890abcdef1234567890abcdef12345678").unwrap();
        Parser::parse(
            Rule::COMMIT_LOGLINE,
            "commit 1234567890abcdef1234567890abcdef12345678",
        )
        .unwrap();
        Parser::parse(
            Rule::COMMIT_LOGLINE,
            "commit 1234567890abcdef1234567890abcdef12345678 (HEAD -> master, origin/master, origin/HEAD)"
        ).unwrap();
        Parser::parse(
            Rule::AUTHOR_LOGLINE,
            "Author: RyanGuild <ryan.guild@us-ignite.org>",
        )
        .unwrap();
        Parser::parse(Rule::DATE_LOGLINE, "Date:   Fri Jul 9 14:00:00 2021 -0400").unwrap();
    }
}
