use crate::graph::{CommitGraph, CommitGraphData};
use crate::parser::parse_log;
use crate::version::Version;
use crate::version_format::VERSION_FORMAT;
use crate::version_map::{VersionMap, VersionMapData};
use eyre::{eyre, Result, *};
use std::rc::Rc;
use std::sync::Arc;
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
    pub fn as_initial_version(&self, commit: LogEntry) -> Version {
        let format = VERSION_FORMAT.lock().unwrap().clone();
        match self {
            Subject::Conventional(sub) => match (sub.breaking, sub.commit_type) {
                (true, _) => format.as_default_version(commit.clone()).major(commit),
                (_, "feat") => format.as_default_version(commit.clone()).minor(commit),
                (_, "fix") => format.as_default_version(commit.clone()).patch(commit),
                _ => format.as_default_version(commit),
            },
            _ => format.as_default_version(commit),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tag<'input> {
    Text(&'input str),
    Version(Version),
}

#[derive(Debug)]
pub struct LogEntryData<'a> {
    pub name: &'a str,
    pub branch: &'a str,
    pub commit_hash: &'a str,
    pub commit_timezone: chrono::Utc,
    pub commit_datetime: chrono::DateTime<chrono::Utc>,
    pub parent_hashes: Arc<[&'a str]>,
    pub decorations: Arc<[Decoration<'a>]>,
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

    pub fn as_initial_version(&self, commit: LogEntry) -> Version {
        self.subject.as_initial_version(commit)
    }
}

pub type LogEntry<'a> = Arc<LogEntryData<'a>>;

pub type Log<'a> = Arc<[LogEntry<'a>]>;

pub const GIT_FORMAT_ARGS: [&str;5] = ["log", "--full-history", "--source", "--branches","--format=name=%n%f%nbranch=%n%S%ncommit=%n%H%ncommit-time=%n%cI%ndec=%n%d%nparent=%n%P%nsub=%n%s%ntrailers=%n%(trailers:only)%n"];

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

    pub fn get_parsed(&mut self) -> Result<Log<'a>> {
        if let Some(parsed) = self.parsed.clone() {
            Ok(parsed)
        } else {
            let parsed = parse_log(self.raw)?;
            self.parsed = Some(parsed.clone());
            Ok(parsed)
        }
    }

    pub fn get_graph(&mut self) -> Result<CommitGraph<'a>> {
        if let Some(graph) = self.graph.clone() {
            Ok(graph.clone())
        } else {
            let log = self.get_parsed()?;
            let graph = CommitGraphData::new(log)?;
            self.graph = Some(graph.clone());
            Ok(graph)
        }
    }

    pub fn get_version_map(&mut self) -> Result<VersionMap> {
        if let Some(version_map) = self.version_map.clone() {
            Ok(version_map.clone())
        } else {
            let graph = self.get_graph()?;
            let version_map: Rc<VersionMapData> = VersionMapData::new(graph)?;
            self.version_map = Some(version_map.clone());
            Ok(version_map.clone())
        }
    }

    pub fn get_commit_version(&mut self, commit_hash: &str) -> Result<Version> {
        let graph = self.get_graph()?;
        let version_map = self.get_version_map()?;
        let commit_idx = graph.commitidx(commit_hash)?;
        version_map
            .get(commit_idx)
            .cloned()
            .ok_or(eyre!("commit not found in version map"))
    }

    pub fn get_latest_version(&mut self) -> Result<Version> {
        let graph = self.get_graph()?;
        let version_map = self.get_version_map()?;
        let head = graph.headidx();
        version_map
            .get(head)
            .cloned()
            .ok_or(eyre!("head not found in version map"))
    }

    pub fn get_uncommited_version(&mut self) -> Result<Version> {
        let graph = self.get_graph()?;
        let head = graph.head();
        self.get_latest_version().map(|version| {
            let branch = self.current_branch_name().unwrap();
            match branch.as_str() {
                "main" | "master" => version.build(head),
                "staging" => version.rc(head),
                "development" => version.beta(head),
                "next" => version.alpha(head),
                _ => version.named(head),
            }
        })
    }

    pub fn current_branch_name(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.dir)
            .output()
            .expect("error getting command output");
        let output_str = String::from_utf8(output.stdout).expect("error parsing utf8");
        Ok(output_str.trim().to_string())
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
        let _ = logs.get_parsed();
    }

    #[test]
    fn test_logs_graph() {
        let mut logs = Logs::default();
        let _ = logs.get_graph();
    }
}
