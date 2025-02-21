use crate::graph::{CommitGraph, CommitGraphData};
use crate::parser::parse_log;
use crate::version::Version;
use crate::version_map::{VersionMap, VersionMapData};
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

impl Subject<'_> {
    pub fn as_initial_version(&self) -> Version {
        match self {
            Subject::Conventional(sub) => match (sub.breaking, sub.commit_type) {
                (true, _) => Version::default().major(),
                (_, "feat") => Version::default().minor(),
                (_, "fix") => Version::default().patch(),
                _ => Version::default(),
            },
            _ => Version::default(),
        }
    }
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

impl LogEntryData<'_> {
    pub fn tagged_version(&self) -> Option<Version> {
        for decoration in self.decorations.iter() {
            if let Decoration::Tag(tag) = decoration {
                if let Tag::Version(version) = tag {
                    return Some(version.clone());
                }
            }
        }
        None
    }

    pub fn as_initial_version(&self) -> Version {
        self.subject.as_initial_version()
    }
}

pub type LogEntry<'a> = Rc<LogEntryData<'a>>;

pub type Log<'a> = Rc<[LogEntry<'a>]>;

pub const GIT_FORMAT_ARGS: [&str;4] = ["log", "--source", "--branches","--format=name=%n%f%nbranch=%n%S%ncommit=%n%H%ncommit-time=%n%cI%ndec=%n%d%nparent=%n%P%nsub=%n%s%ntrailers=%n%(trailers:only)%n"];

#[derive(Debug)]
pub struct Logs<'a> {
    raw: &'a str,
    dir: PathBuf,
    parsed: Option<Log<'a>>,
    graph: Option<CommitGraph<'a>>,
    version_map: Option<VersionMap>,
}

impl<'a> Logs<'a> {
    pub fn new(dir: PathBuf) -> Logs<'a> {
        let output = Command::new("git")
            .args(&GIT_FORMAT_ARGS)
            .current_dir(&dir)
            .output()
            .expect("error getting command output");
        let output_str = String::from_utf8(output.stdout).expect("error parsing utf8");
        Logs {
            raw: Box::leak(output_str.into_boxed_str()),
            dir,
            parsed: None,
            graph: None,
            version_map: None,
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


    pub fn get_version_map(&mut self) -> VersionMap {
        if let Some(version_map) = self.version_map.clone() {
            version_map.clone()
        } else {
            let graph = self.get_graph();
            let version_map = VersionMapData::new(&graph).expect("could not create version map");
            self.version_map = Some(version_map.clone());
            version_map
        }
    }


    pub fn get_commit_version(&mut self, commit_hash: &str) -> Version {
        let graph = self.get_graph();
        let version_map = self.get_version_map();
        let commit_idx = graph.commitidx(commit_hash).expect("commit not found in graph");
        version_map.get(commit_idx).expect("commit not found in version map").clone()
    }

    pub fn get_latest_version(&mut self) -> Version {
        let graph = self.get_graph();
        let version_map = self.get_version_map();
        let head = graph.headidx();
        version_map.get(head).expect("tail not found in version map").clone()
    }

    pub fn get_uncommited_version(&mut self) -> Version {
        self.get_latest_version().build()
    }

    pub fn is_dirty(&self) -> bool {
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&self.dir)
            .output()
            .expect("error getting command output");
        let output_str = String::from_utf8(output.stdout).expect("error parsing utf8");
        !output_str.is_empty()
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
