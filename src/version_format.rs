use std::cell::{Cell, RefCell};
use std::cmp::Ordering;

pub static VERSION_FORMAT: Mutex<VersionFormat> = Mutex::new(VersionFormat {
    v_prefix: true,
    major: VersionNumberFormat::CCVer,
    minor: VersionNumberFormat::CCVer,
    patch: VersionNumberFormat::CCVer,
    prerelease: None,
});


#[derive(Debug, Clone, Default)]
pub struct VersionFormat<'ctx> {
    pub v_prefix: bool,
    pub major: VersionNumberFormat,
    pub minor: VersionNumberFormat,
    pub patch: VersionNumberFormat,
    pub prerelease: Option<PreTagFormat<'ctx>>,
}

impl<'ctx> VersionFormat<'ctx> {
    pub fn as_default_version(&self, commit: LogEntry) -> Version {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.as_default_version_number(commit.clone()),
            minor: self.minor.as_default_version_number(commit.clone()),
            patch: self.patch.as_default_version_number(commit.clone()),
            prerelease: self
                .prerelease
                .as_ref()
                .map(|ptf| ptf.as_default_pre_tag(commit)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VersionNumberFormat {
    CCVer,
    CalVer(CalVerFormat),
    Sha,
    ShortSha,
}

impl VersionNumberFormat {
    pub fn as_default_version_number(&self, commit: LogEntry) -> VersionNumber {
        match self {
            VersionNumberFormat::CCVer => VersionNumber::CCVer(0),
            VersionNumberFormat::CalVer(calendar_parts) => {
                VersionNumber::CalVer(calendar_parts.clone(), commit.commit_datetime)
            },
            VersionNumberFormat::Sha => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumberFormat::ShortSha => VersionNumber::ShortSha(commit.commit_hash[0..7].to_string()),
        }
    }
}

impl Default for VersionNumberFormat {
    fn default() -> Self {
        VersionNumberFormat::CCVer
    }
}

impl Default for &VersionNumberFormat {
    fn default() -> Self {
        &VersionNumberFormat::CCVer
    }
}

pub type CalVerFormat = Arc<[CalVerFormatSegment]>;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum CalVerFormatSegment {
    Year4,
    Year2,
    Epoch,
    Month,
    Day,
    DayOfYear,
    Hour,
    Minute,
    Second,
}

impl PartialOrd for CalVerFormatSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use std::cmp::Ordering::*;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use CalVerFormatSegment::*;

use crate::{
    graph::CommitGraph, logs::LogEntry, version::{PreTag, Version, VersionNumber}
};

impl Ord for CalVerFormatSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Year4 | Year2 | Epoch => match other {
                Year4 | Year2 | Epoch => Equal,
                _ => Greater,
            },

            DayOfYear => match other {
                Year4 | Year2 | Epoch => Greater,
                DayOfYear => Equal,
                _ => Less,
            },

            Month => match other {
                Year4 | Year2 | Epoch | DayOfYear => Less,
                Month => Equal,
                _ => Greater,
            },

            Day => match other {
                Year4 | Year2 | Epoch | DayOfYear | Month => Less,
                Day => Equal,
                _ => Greater,
            },

            Hour => match other {
                Minute | Second => Greater,
                Hour => Equal,
                _ => Less,
            },

            Minute => match other {
                Second => Greater,
                Minute => Equal,
                _ => Less,
            },

            Second => match other {
                Second => Equal,
                _ => Less,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum PreTagFormat<'ctx> {
    Rc(VersionNumberFormat),
    Beta(VersionNumberFormat),
    Alpha(VersionNumberFormat),
    Build(VersionNumberFormat),
    Named(String, VersionNumberFormat),
    Sha(CommitGraph<'ctx>, VersionNumberFormat),
    ShortSha(CommitGraph<'ctx>, VersionNumberFormat),
}

pub static PRE_TAG_FORMAT: Mutex<PreTagFormat> =
    Mutex::new(PreTagFormat::Build(VersionNumberFormat::CCVer));

impl Default for PreTagFormat<'_> {
    fn default() -> Self {
        PRE_TAG_FORMAT.lock().unwrap().clone()
    }
}

impl PreTagFormat<'_> {
    pub fn version_format(&self) -> VersionNumberFormat {
        match self {
            PreTagFormat::Rc(vf)
            | PreTagFormat::Beta(vf)
            | PreTagFormat::Alpha(vf)
            | PreTagFormat::Build(vf)
            | PreTagFormat::Sha(_, vf)
            | PreTagFormat::ShortSha(_, vf)
            | PreTagFormat::Named(_, vf) => vf.clone(),
        }
    }

    pub fn as_default_pre_tag(&self, commit: LogEntry) -> PreTag {
        match self {
            PreTagFormat::Rc(vf) => PreTag::Rc(vf.as_default_version_number(commit)),
            PreTagFormat::Beta(vf) => PreTag::Beta(vf.as_default_version_number(commit)),
            PreTagFormat::Alpha(vf) => PreTag::Alpha(vf.as_default_version_number(commit)),
            PreTagFormat::Build(vf) => PreTag::Build(vf.as_default_version_number(commit)),
            PreTagFormat::Named(name, vf) => {
                PreTag::Named(name.clone(), vf.as_default_version_number(commit))
            }
            PreTagFormat::Sha(graph, _) => PreTag::Sha(VersionNumber::ShortSha(graph.tail().commit_hash.to_string())),
            PreTagFormat::ShortSha(graph, _) => {
                PreTag::ShortSha(VersionNumber::ShortSha(graph.tail().commit_hash[0..7].to_string()))
            }
        }
    }

    pub fn parse(&self, data: &str) -> PreTag {
        match self {
            PreTagFormat::Rc(vf) => PreTag::Rc(vf.parse(data)),
            PreTagFormat::Beta(vf) => PreTag::Beta(vf.parse(data)),
            PreTagFormat::Alpha(vf) => PreTag::Alpha(vf.parse(data)),
            PreTagFormat::Build(vf) => PreTag::Build(vf.parse(data)),
            PreTagFormat::Named(name, vf) => PreTag::Named(name.clone(), vf.parse(data)),
            PreTagFormat::Sha(_, vf) => PreTag::Sha(vf.parse(data)),
            PreTagFormat::ShortSha(_,vf) => PreTag::ShortSha(vf.parse(data)),
        }
    }
}

impl VersionNumberFormat {
    pub fn parse(&self, data: &str) -> VersionNumber {
        match self {
            VersionNumberFormat::CCVer => VersionNumber::CCVer(usize::from_str(data).unwrap()),
            VersionNumberFormat::CalVer(calendar_parts) => {
                        let format_str: String = calendar_parts
                            .iter()
                            .map(|part| match part {
                                Year4 => "%Y",
                                Year2 => "%y",
                                Epoch => "%s",
                                Month => "%m",
                                Day => "%d",
                                DayOfYear => "%j",
                                Hour => "%H",
                                Minute => "%M",
                                Second => "%S",
                            })
                            .collect::<Vec<&str>>()
                            .join("");

                        let date = chrono::DateTime::parse_from_str(data, &format_str).unwrap();
                        VersionNumber::CalVer(calendar_parts.clone(), date.to_utc())
                    }
            VersionNumberFormat::Sha => VersionNumber::Sha(data.to_string()),
            VersionNumberFormat::ShortSha => VersionNumber::ShortSha(data.to_string()),

        }
    }
}
