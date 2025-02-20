use crate::graph::{CommitGraph, CommitGraphData};
use crate::parser::parse_log;
use crate::version::Version;
use std::rc::Rc;
use std::{env::current_dir, path::PathBuf, process::Command};

#[derive(Debug)]
pub enum Decoration<'a> {
    HeadIndicator(&'a str),
    Tag(Tag<'a>),
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

#[derive(Debug, PartialEq, Eq)]
pub enum Tag<'a> {
    Text(&'a str),
    Version(Version),
}

#[derive(Debug)]
pub struct LogEntryData<'a> {
    pub name: &'a str,
    pub branch: &'a str,
    pub commit_hash: &'a str,
    pub commit_timezone: chrono::Utc,
    pub commit_datetime: chrono::DateTime<chrono::Utc>,
    pub parent_hashes: Rc<[&'a str]>,
    pub decorations: Rc<[Decoration<'a>]>,
    pub subject: Subject<'a>,
    pub footers: std::collections::HashMap<&'a str, &'a str>,
}

pub type LogEntry<'a> = Rc<LogEntryData<'a>>;

pub type Log<'a> = Rc<[LogEntry<'a>]>;

pub const GIT_FORMAT_ARGS: [&str;4] = ["log", "--source", "--branches","--format=name=%n%f%nbranch=%n%S%ncommit=%n%H%ncommit-time=%n%cI%ndec=%n%d%nparent=%n%P%nsub=%n%s%ntrailers=%n%(trailers:only)%n"];

#[derive(Debug)]
pub struct Logs<'a> {
    raw: &'a str,
    parsed: Option<Log<'a>>,
    graph: Option<CommitGraph<'a>>,
}

impl<'a> Logs<'a> {
    pub fn new(dir: PathBuf) -> Logs<'a> {
        let output = Command::new("git")
            .args(&GIT_FORMAT_ARGS)
            .current_dir(dir)
            .output()
            .expect("error getting command output");
        let output_str = String::from_utf8(output.stdout).expect("error parsing utf8");
        Logs {
            raw: Box::leak(output_str.into_boxed_str()),
            parsed: None,
            graph: None,
        }
    }

    pub fn get_raw(&self) -> &'a str {
        self.raw
    }

    pub fn get_parsed(&mut self) -> Log<'a> {
        if let Some(parsed) = self.parsed.clone() {
            parsed
        } else {
            let parsed = parse_log(self.raw).expect("could not parse raw logs");
            self.parsed = Some(parsed.clone());
            parsed
        }
    }

    pub fn get_graph(&mut self) -> CommitGraph<'a> {
        if let Some(graph) = self.graph.clone() {
            graph.clone()
        } else {
            let log = self.get_parsed();
            let graph = CommitGraphData::new(log).expect("could not parse commits to graph");
            self.graph = Some(graph.clone());
            graph
        }
    }
}

impl Default for Logs<'_> {
    fn default() -> Self {
        let dir = current_dir().unwrap();
        Logs::new(dir)
    }
}

#[cfg(test)]
mod logs_tests {

    use super::*;

    #[test]
    fn test_default() {
        Logs::default();
    }

    #[test]
    fn test_logs_parsed() {
        let mut logs = Logs::default();
        logs.get_parsed();
    }

    #[test]
    fn test_logs_graph() {
        let mut logs = Logs::default();
        logs.get_graph();
    }
}
