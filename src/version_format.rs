use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone)]
pub struct VersionFormat {
    pub v_prefix: bool,
    pub major: VersionNumberFormat,
    pub minor: VersionNumberFormat,
    pub patch: VersionNumberFormat,
    pub prerelease: Option<PreTagFormat>,
}

impl Default for VersionFormat {
    fn default() -> Self {
        VersionFormat {
            v_prefix: true,
            major: VersionNumberFormat::default(),
            minor: VersionNumberFormat::default(),
            patch: VersionNumberFormat::default(),
            prerelease: Some(PreTagFormat::default()),
        }
    }
}

impl VersionFormat {
    pub fn as_default_version(&self, commit: &LogEntry) -> Version {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.as_default_version_number(commit),
            minor: self.minor.as_default_version_number(commit),
            patch: self.patch.as_default_version_number(commit),
            prerelease: self
                .prerelease
                .as_ref()
                .map(|ptf| ptf.as_default_pre_tag(commit)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum VersionNumberFormat {
    #[default]
    CCVer,
    CalVer(CalVerFormat),
    Sha,
    ShortSha,
}

impl VersionNumberFormat {
    pub fn as_default_version_number(&self, commit: &LogEntry) -> VersionNumber {
        match self {
            VersionNumberFormat::CCVer => VersionNumber::CCVer(0),
            VersionNumberFormat::CalVer(calendar_parts) => {
                VersionNumber::CalVer(calendar_parts.clone(), commit.commit_datetime)
            }
            VersionNumberFormat::Sha => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumberFormat::ShortSha => {
                VersionNumber::ShortSha(commit.commit_hash[0..7].to_string())
            }
        }
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

use CalVerFormatSegment::*;
use std::cmp::Ordering::*;
use std::str::FromStr;
use std::sync::Arc;

use crate::{
    logs::LogEntry,
    version::{PreTag, Version, VersionNumber},
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
pub enum PreTagFormat {
    Rc(VersionNumberFormat),
    Beta(VersionNumberFormat),
    Alpha(VersionNumberFormat),
    Build(VersionNumberFormat),
    Named(String, VersionNumberFormat),
    Sha,
    ShortSha,
}

impl Display for PreTagFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PreTagFormat::Rc(vf) => write!(f, "rc.{}", vf),
            PreTagFormat::Beta(vf) => write!(f, "beta.{}", vf),
            PreTagFormat::Alpha(version_number_format) => {
                write!(f, "alpha.{}", version_number_format)
            }
            PreTagFormat::Build(version_number_format) => {
                write!(f, "build.{}", version_number_format)
            }
            PreTagFormat::Named(name, version_number_format) => {
                write!(f, "{}.{}", name, version_number_format)
            }
            sha => write!(f, "+{}", sha),
        }
    }
}

impl Default for PreTagFormat {
    fn default() -> Self {
        PreTagFormat::Build(VersionNumberFormat::CCVer)
    }
}

impl Default for &PreTagFormat {
    fn default() -> Self {
        &PreTagFormat::Build(VersionNumberFormat::CCVer)
    }
}

impl PreTagFormat {
    pub fn as_default_pre_tag(&self, commit: &LogEntry) -> PreTag {
        match self {
            PreTagFormat::Rc(vf) => PreTag::Rc(vf.as_default_version_number(commit)),
            PreTagFormat::Beta(vf) => PreTag::Beta(vf.as_default_version_number(commit)),
            PreTagFormat::Alpha(vf) => PreTag::Alpha(vf.as_default_version_number(commit)),
            PreTagFormat::Build(vf) => PreTag::Build(vf.as_default_version_number(commit)),
            PreTagFormat::Named(name, vf) => {
                PreTag::Named(name.clone(), vf.as_default_version_number(commit))
            }
            PreTagFormat::Sha => {
                PreTag::Sha(VersionNumber::ShortSha(commit.commit_hash.to_string()))
            }
            PreTagFormat::ShortSha => PreTag::ShortSha(VersionNumber::ShortSha(
                commit.commit_hash[0..7].to_string(),
            )),
        }
    }

    pub fn version_format(&self) -> VersionNumberFormat {
        match self {
            PreTagFormat::Rc(vf) => vf.clone(),
            PreTagFormat::Beta(vf) => vf.clone(),
            PreTagFormat::Alpha(vf) => vf.clone(),
            PreTagFormat::Build(vf) => vf.clone(),
            PreTagFormat::Named(_, vf) => vf.clone(),
            PreTagFormat::Sha => VersionNumberFormat::Sha,
            PreTagFormat::ShortSha => VersionNumberFormat::ShortSha,
        }
    }

    pub fn parse(&self, data: &str) -> PreTag {
        match self {
            PreTagFormat::Rc(vf) => PreTag::Rc(vf.parse(data)),
            PreTagFormat::Beta(vf) => PreTag::Beta(vf.parse(data)),
            PreTagFormat::Alpha(vf) => PreTag::Alpha(vf.parse(data)),
            PreTagFormat::Build(vf) => PreTag::Build(vf.parse(data)),
            PreTagFormat::Named(name, vf) => PreTag::Named(name.clone(), vf.parse(data)),
            PreTagFormat::Sha => PreTag::Sha(VersionNumber::Sha(data.to_string())),
            PreTagFormat::ShortSha => {
                PreTag::ShortSha(VersionNumber::ShortSha(data[0..7].to_string()))
            }
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
            VersionNumberFormat::ShortSha => VersionNumber::ShortSha(data[0..7].to_string()),
        }
    }
}

impl Display for VersionFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pre = match &self.prerelease {
            Some(pre) => pre,
            None => &PreTagFormat::default(),
        };

        write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
    }
}

impl Display for VersionNumberFormat {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            VersionNumberFormat::CCVer => write!(f, "CC"),
            VersionNumberFormat::CalVer(format) => {
                for seg in format.iter() {
                    write!(f, "{}", seg)?;
                }
                std::fmt::Result::Ok(())
            }
            VersionNumberFormat::Sha => write!(f, "<sha>"),
            VersionNumberFormat::ShortSha => write!(f, "<short-sha>"),
        }
    }
}

impl Display for CalVerFormatSegment {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Year4 => write!(f, "YYYY"),
            Year2 => write!(f, "YY"),
            Epoch => write!(f, "E"),
            Month => write!(f, "MM"),
            Day => write!(f, "DD"),
            DayOfYear => write!(f, "DDD"),
            Hour => write!(f, "hh"),
            Minute => write!(f, "mm"),
            Second => write!(f, "ss"),
        }
    }
}
