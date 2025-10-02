use crate::parser::parse_log;
use crate::pattern_macros::{major_subject, minor_subject, patch_subject};
use crate::version::Version;
use crate::version_format::VersionFormat;
use crate::{git, parser};
use eyre::*;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::{env::current_dir, path::Path};
use tracing::{debug, info, instrument};

pub const PEEK_COMMIT_HASH: &str = "0000000000000000000000000000000000000000";

#[derive(Debug)]
pub enum Decoration<'a> {
    HeadIndicator(&'a str),
    Tag(Tag<'a>),
    RemoteBranch((&'a str, &'a str)),
    Branch(&'a str),
}

#[derive(Debug, Clone)]
pub struct ConventionalSubject<'a> {
    pub commit_type: &'a str,
    pub breaking: bool,
    pub scope: Option<&'a str>,
    pub description: &'a str,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    pub fn as_initial_version(&self, commit: &LogEntry, version_format: &VersionFormat) -> Version {
        self.subject.as_initial_version(commit, version_format)
    }
}

pub trait PeekLogEntry {
    fn into_peek_log_entry(
        self,
        parent_commit: &'static str,
        branch: &'static str,
    ) -> LogEntry<'static>;
}

impl PeekLogEntry for &'static str {
    fn into_peek_log_entry(
        self,
        parent_commit: &'static str,
        branch: &'static str,
    ) -> LogEntry<'static> {
        let parsed_subject = parser::parse_subject(self).unwrap();

        // For peek entries, we need to convert the subject to use the parent's lifetime
        // Since this is a preview operation, we'll create a simplified subject
        let subject = match parsed_subject {
            Subject::Conventional(conv) => {
                // Leak the strings to get 'static lifetime, then coerce to 'a
                let commit_type: &str = Box::leak(conv.commit_type.to_string().into_boxed_str());
                let description: &str = Box::leak(conv.description.to_string().into_boxed_str());
                let scope: Option<&str> = conv.scope.map(|s| {
                    let leaked: &'static str = Box::leak(s.to_string().into_boxed_str());
                    leaked
                });

                Subject::Conventional(ConventionalSubject {
                    commit_type,
                    breaking: conv.breaking,
                    scope,
                    description,
                })
            }
            Subject::Text(text) => {
                let leaked_text: &str = Box::leak(text.to_string().into_boxed_str());
                Subject::Text(leaked_text)
            }
        };

        LogEntry {
            name: "peek-next-commit",
            branch,
            commit_hash: PEEK_COMMIT_HASH,
            commit_timezone: chrono::Utc,
            commit_datetime: chrono::Utc::now(),
            parent_hashes: vec![parent_commit].into(),
            decorations: Arc::new([Decoration::HeadIndicator(branch)]),
            subject,
            footers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Logs<'a>(Vec<LogEntry<'a>>);

impl Logs<'_> {
    pub fn iter(&'_ self) -> impl Iterator<Item = &'_ LogEntry<'_>> {
        self.0.iter()
    }
}

impl<'a> Deref for Logs<'a> {
    type Target = Vec<LogEntry<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Logs<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for Logs<'a> {
    type Item = LogEntry<'a>;
    type IntoIter = std::vec::IntoIter<LogEntry<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> FromIterator<LogEntry<'a>> for Logs<'a> {
    fn from_iter<T: IntoIterator<Item = LogEntry<'a>>>(iter: T) -> Self {
        let entries: Vec<LogEntry<'a>> = iter.into_iter().collect();
        Logs(entries)
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
    #[instrument(skip(raw))]
    pub fn from_log_str<'a>(raw: &'a str) -> Result<Logs<'a>> {
        debug!("Parsing logs from string ({} chars)", raw.len());
        let logs = parse_log(raw)?;
        info!("Successfully parsed logs");
        Ok(logs)
    }

    #[instrument]
    pub fn from_path(path: &Path) -> Result<Logs<'static>> {
        info!("Loading logs from path: {:?}", path);
        let raw = git::formatted_logs(path)?;
        Logs::from_log_str(raw)
    }
}

impl<'a> Logs<'a> {
    pub fn with_additional_log_entry<'b: 'a>(&self, log: LogEntry<'b>) -> Logs<'a> {
        let mut next_logs = self.0.clone();

        next_logs.push(log);

        Logs(next_logs)
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

pub trait InfersVersionFormat {
    fn infer_version_format(&self) -> VersionFormat;
}

impl<'a> InfersVersionFormat for Logs<'a> {
    fn infer_version_format(&self) -> VersionFormat {
        let mut log_entries: Vec<_> = self.iter().collect();
        log_entries.sort_by(|a, b| a.commit_datetime.cmp(&b.commit_datetime));

        log_entries
            .into_iter()
            .flat_map(|log| {
                log.decorations.iter().find_map(|d| match d {
                    Decoration::Tag(Tag::Version(version)) => Some(version.clone()),
                    _ => None,
                })
            })
            .map(Into::<VersionFormat>::into)
            .next()
            .unwrap_or_default()
    }
}
