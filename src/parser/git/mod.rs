use std::str::FromStr;

use chrono::{DateTime, TimeZone};
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
                let inner = entry.into_inner();
                let mut tags = flat_tag_map(inner);
                dbg!(&tags);

                let date = if let Some(d) = tags.remove("date_string") {
                    chrono::DateTime::parse_from_str(&d, "%a %b %e %T %Y %z")
                        .expect("cannot parse date")
                        .to_utc()
                } else if let Some(d) = tags.remove("commiter_time").or(tags.remove("author_time")) {
                    chrono::DateTime::parse_from_str(&d, "%s %z")
                        .expect("cannot parse date")
                        .to_utc()
                } else {
                    panic!("no date found")
                };



        
                Some(LogEntry {
                    commit_sha: tags.remove("commit_sha").unwrap(),
                    tree_sha: tags.remove("tree_sha"),
                    parent_shas: tags.remove("parent_sha"),
                    author_name: tags.remove("author_name").unwrap(),
                    author_email: tags.remove("author_email").unwrap(),
                    date: date,
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
        let log = String::from_utf8(
            Command::new("git")
                .arg("log")
                .arg("--full-history")
                .arg("--pretty=raw")
                .arg("--topo-order")
                .arg("--decorate=full")
                .arg("--all")
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
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
            Rule::AUTHOR_INFO,
            "Author: RyanGuild <ryan.guild@us-ignite.org>",
        )
        .unwrap();
        Parser::parse(Rule::DATE_LOGLINE, "Date:   Fri Jul 9 14:00:00 2021 -0400").unwrap();
    }

    #[test]
    fn parse_author_logline() {
        let author = Parser::parse(
            Rule::AUTHOR_INFO,
            "author RyanGuild <ryan.guild@us-ignite.org> 1737909824 -0500",
        )
        .unwrap()
        .next()
        .unwrap();
        println!("{:#?}", author);

        let author2 = Parser::parse(
            Rule::AUTHOR_INFO,
            "Author: RyanGuild <ryan.guild@us-ignite.org>",
        )
        .unwrap()
        .next()
        .unwrap();
        println!("{:#?}", author2);
    }

    #[test]
    fn parse_log() {
        let entry = Parser::parse(Rule::LOG, include_str!("./log.txt"))
            .unwrap()
            .next()
            .unwrap();

        println!("{:#?}", entry);
    }

    #[test]
    fn parse_log_entry() {
        let entry = Parser::parse(Rule::LOG_ENTRY, include_str!("./entry.txt"))
            .unwrap()
            .next()
            .unwrap();
        println!("{:#?}", entry);
    }
}
