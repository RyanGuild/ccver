use crate::git;
use crate::parser::parse_log;
use crate::pattern_macros::{major_subject, minor_subject, patch_subject};
use crate::version::Version;
use crate::version_format::VersionFormat;
use eyre::*;
use std::sync::Arc;
use std::{env::current_dir, path::Path};

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
    pub fn as_initial_version(&self, commit: &LogEntry, format: &VersionFormat) -> Version {
        match self {
            major_subject!() => format.as_default_version(commit).major(commit, format),
            minor_subject!() => format.as_default_version(commit).minor(commit, format),
            patch_subject!() => format.as_default_version(commit).patch(commit, format),
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
pub struct LogEntry<'a> {
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

impl LogEntry<'_> {
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

    pub fn as_initial_version(&self, commit: &LogEntry, version_format: &VersionFormat) -> Version {
        self.subject.as_initial_version(commit, version_format)
    }
}

#[derive(Debug)]
pub struct Logs<'a>(&'a [LogEntry<'a>]);

impl Logs<'_> {
    pub fn iter(&'_ self) -> impl Iterator<Item = &'_ LogEntry<'_>> {
        self.0.into_iter()
    }
}

impl<'a> FromIterator<LogEntry<'a>> for Logs<'a> {
    fn from_iter<T: IntoIterator<Item = LogEntry<'a>>>(iter: T) -> Self {
        let entries: Vec<LogEntry<'a>> = iter.into_iter().collect();
        Logs(entries.leak())
    }
}

pub const GIT_FORMAT_ARGS: [&str; 5] = [
    "log",
    "--full-history",
    "--source",
    "--branches",
    "--format=name=%n%f%nbranch=%n%S%ncommit=%n%H%ncommit-time=%n%cI%ndec=%n%d%nparent=%n%P%nsub=%n%s%ntrailers=%n%(trailers:only)%n",
];

impl Logs<'_> {
    pub fn from_str<'a>(raw: &'a str) -> Result<Logs<'a>> {
        Ok(parse_log(raw)?)
    }

    pub fn from_path(path: &Path) -> Result<Logs<'static>> {
        let raw = git::formatted_logs(path)?;
        Logs::from_str(raw)
    }
}

impl Default for Logs<'_> {
    fn default() -> Self {
        Logs::from_path(&current_dir().expect("could not get current dir")).unwrap()
    }
}

#[cfg(test)]
mod logs_tests {

    use super::*;

    #[test]
    fn test_logs_parsed() {
        let _logs = Logs::default();
    }
}
