use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use crate::{logs::LogEntry, version_format::{CalVerFormat, CalVerFormatSegment, PRE_TAG_FORMAT}};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Version {
    pub v_prefix: bool,
    pub major: VersionNumber,
    pub minor: VersionNumber,
    pub patch: VersionNumber,
    pub prerelease: Option<PreTag>,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.v_prefix {
            write!(f, "v")?;
        }
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        match &self.prerelease {
            None => Ok(()),
            Some(pre) => write!(f, "-{}", pre),
        }
    }
}

impl Version {


    pub fn major(&self, commit: LogEntry) -> Self {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.bump(commit.clone()),
            minor: self.minor.zero(commit.clone()),
            patch: self.patch.zero(commit.clone()),
            prerelease: None,
        }
    }

    pub fn minor(&self, commit: LogEntry) -> Self {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.bump(commit.clone()),
            patch: self.minor.zero(commit),
            prerelease: None,
        }
    }

    pub fn patch(&self, commit: LogEntry) -> Self {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.bump(commit),
            prerelease: None,
        }
    }

    pub fn build(&self, commit: LogEntry) -> Self {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Build(PRE_TAG_FORMAT.lock().unwrap().version_format().as_default_version_number(commit))),
                Some(pre) => match pre {
                    PreTag::Build(v) => Some(PreTag::Build(v.bump(commit))),
                    _ => Some(PreTag::Build(PRE_TAG_FORMAT.lock().unwrap().version_format().as_default_version_number(commit))),
                },
            },
        }
    }

    pub fn rc(&self, commit: LogEntry) -> Self {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Rc(PRE_TAG_FORMAT.lock().unwrap().version_format().as_default_version_number(commit))),
                Some(pre) => match pre {
                    PreTag::Rc(v) => Some(PreTag::Rc(v.bump(commit))),
                    _ => Some(PreTag::Rc(PRE_TAG_FORMAT.lock().unwrap().version_format().as_default_version_number(commit))),
                },
            },
        }
    }

    pub fn beta(&self, commit: LogEntry) -> Self {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Beta(PRE_TAG_FORMAT.lock().unwrap().version_format().as_default_version_number(commit))),
                Some(pre) => match pre {
                    PreTag::Beta(v) => Some(PreTag::Beta(v.bump(commit))),
                    _ => Some(PreTag::Beta(PRE_TAG_FORMAT.lock().unwrap().version_format().as_default_version_number(commit))),
                },
            },
        }
    }

    pub fn alpha(&self, commit: LogEntry) -> Version {
        let pre_version_format = PRE_TAG_FORMAT.lock().unwrap().version_format();
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Alpha(pre_version_format.as_default_version_number(commit))),
                Some(pre) => match pre {
                    PreTag::Alpha(v) => Some(PreTag::Alpha(v.bump(commit))),
                    _ => Some(PreTag::Alpha(pre_version_format.as_default_version_number(commit))),
                },
            },
        }
    }

    pub fn named(&self, commit: LogEntry) -> Version {
        let pre_version_format = PRE_TAG_FORMAT.lock().unwrap().version_format();
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Named(commit.branch.to_string(), pre_version_format.as_default_version_number(commit))),
                Some(pre) => match pre {
                    PreTag::Named(tag, v) if tag.eq(commit.branch) => Some(PreTag::Named(tag.to_string(), v.bump(commit))),
                    PreTag::Named(_, _) => Some(PreTag::Named(commit.branch.to_string(), pre_version_format.as_default_version_number(commit))),
                    _ => Some(PreTag::Named(commit.branch.to_string(), pre_version_format.as_default_version_number(commit))),
                },
            },
        }
    }

    pub fn release(&self, commit: LogEntry) -> Version {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: None,
        }
    }

    pub fn sha(&self, commit: LogEntry) -> Version {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: Some(PreTag::Sha(VersionNumber::Sha(commit.commit_hash.to_string()))),
        }
    }

    pub fn short_sha(&self, commit: LogEntry) -> Version {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.peek(commit.clone()),
            minor: self.minor.peek(commit.clone()),
            patch: self.patch.peek(commit.clone()),
            prerelease: Some(PreTag::ShortSha(VersionNumber::ShortSha(commit.commit_hash[0..7].to_string()))),
        }
    }
}



impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => match (&self.prerelease,&other.prerelease) {
                        (None, None) => Ordering::Equal,
                        (Some(PreTag::Build(_)), None) => Ordering::Greater,
                        (None, Some(PreTag::Build(_))) => Ordering::Less,
                        (Some(PreTag::Build(a)), Some(PreTag::Build(b))) => a.cmp(b),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (Some(a), Some(b)) => match a.partial_cmp(b) {
                            Some(ord) => ord,
                            None => Ordering::Equal,
                        }

                    },
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PreTag {
    Rc(VersionNumber),
    Beta(VersionNumber),
    Alpha(VersionNumber),
    Build(VersionNumber),
    Named(String, VersionNumber),
    Sha(VersionNumber),
    ShortSha(VersionNumber),
}

impl Display for PreTag {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PreTag::Rc(v) => write!(f, "rc.{}", v),
            PreTag::Beta(v) => write!(f, "beta.{}", v),
            PreTag::Alpha(v) => write!(f, "alpha.{}", v),
            PreTag::Build(v) => write!(f, "build.{}", v),
            PreTag::Named(tag, v) => write!(f, "{}.{}", tag, v),
            PreTag::Sha(s) => write!(f, "{}", s),
            PreTag::ShortSha(s) => write!(f, "{}", s),
        }
    }
}


impl PartialOrd for PreTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            PreTag::Rc(v) => match other {
                PreTag::Rc(v2) => v.partial_cmp(v2),
                PreTag::Alpha(_) | PreTag::Beta(_) | PreTag::Build(_) => Some(Ordering::Greater),
                _ => None,
            },
            PreTag::Beta(v) => match other {
                PreTag::Rc(_) => Some(Ordering::Less),
                PreTag::Beta(v2) => v.partial_cmp(v2),
                PreTag::Alpha(_) | PreTag::Build(_) => Some(Ordering::Greater),
                _ => None,
            },
            PreTag::Alpha(v) => match other {
                PreTag::Rc(_) | PreTag::Beta(_) => Some(Ordering::Less),
                PreTag::Alpha(v2) => v.partial_cmp(v2),
                PreTag::Build(_) => Some(Ordering::Greater),
                _ => None,
            },
            PreTag::Build(v) => match other {
                PreTag::Rc(_) | PreTag::Beta(_) | PreTag::Alpha(_) => Some(Ordering::Less),
                PreTag::Build(v2) => v.partial_cmp(v2),
                _ => None,
            },
            PreTag::Named(tag, v) => match other {
                PreTag::Named(tag2, v2) => {
                    if tag == tag2 {
                        v.partial_cmp(v2)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum VersionNumber {
    CCVer(usize),
    CalVer(CalVerFormat, chrono::DateTime<chrono::Utc>),
    Sha(String),
    ShortSha(String),
}

impl VersionNumber {
    pub fn bump(&self, commit: LogEntry) -> Self {
        match self {
            VersionNumber::CCVer(v) => VersionNumber::CCVer(*v + 1),
            VersionNumber::CalVer(format, _) => {
                        VersionNumber::CalVer(format.clone(), commit.commit_datetime)
                    }
            VersionNumber::Sha(_) => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumber::ShortSha(_) => VersionNumber::ShortSha(commit.commit_hash[0..7].to_string()),
        }
    }

    pub fn peek(&self, commit: LogEntry) -> Self {
        match self {
            VersionNumber::CCVer(v) => VersionNumber::CCVer(*v),
            VersionNumber::CalVer(format, _) => {
                        VersionNumber::CalVer(format.clone(), commit.commit_datetime)
                    }
            VersionNumber::Sha(_) => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumber::ShortSha(_) => VersionNumber::ShortSha(commit.commit_hash[0..7].to_string()),
        }
    }

    pub fn zero(&self, commit: LogEntry) -> Self {
        match self {
            VersionNumber::CCVer(_) => VersionNumber::CCVer(0),
            VersionNumber::CalVer(_, _) => self.bump(commit),
            VersionNumber::Sha(_) => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumber::ShortSha(_) => VersionNumber::ShortSha(commit.commit_hash[0..7].to_string()),
        }
    }
}

impl Ord for VersionNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            VersionNumber::CCVer(ver) => match other {
                VersionNumber::CCVer(ver2) => ver.cmp(ver2),
                _ => panic!("Cannot compare CCVer Version Number with CalVer Version Number"),
            },
            VersionNumber::CalVer(format, date) => match other {
                VersionNumber::CalVer(format2, date2) => {
                    if format.iter().eq(format2.iter()) {
                        date.cmp(date2)
                    } else {
                        // TODO: Implement a link between all the calver in a version so that date segments can be compared without regard to format
                        panic!("Cannot compare CalVer Version Number with different formats")
                    }
                }
                _ => panic!("Cannot compare CCVer Version Number with CalVer Version Number"),
            },
            VersionNumber::Sha(s) => match other {
                VersionNumber::Sha(s2) => s.cmp(s2),
                _ => panic!("Cannot compare Sha Version Number with non Sha Version Number"),
            },
            VersionNumber::ShortSha(s) => match other {
                VersionNumber::ShortSha(s2) => s.cmp(s2),
                _ => panic!("Cannot compare ShortSha Version Number with non ShortSha Version Number"),
            },
        }
    }
}

impl PartialOrd for VersionNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            VersionNumber::CCVer(ver) => match other {
                        VersionNumber::CCVer(ver2) => Some(ver.cmp(ver2)),
                        _ => None,
                    },
            VersionNumber::CalVer(format, date) => match other {
                        VersionNumber::CalVer(format2, date2) => {
                            if format.iter().eq(format2.iter()) {
                                date.partial_cmp(date2)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
            VersionNumber::Sha(_) | VersionNumber::ShortSha(_) => None,
        }
    }
}

impl Display for VersionNumber {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            VersionNumber::CCVer(v) => write!(f, "{}", v),
            VersionNumber::CalVer(format, date) => {
                        let err = format
                            .iter()
                            .map(|seg| match seg {
                                CalVerFormatSegment::Year4 => write!(f, "{}", date.format("%Y")),
                                CalVerFormatSegment::Year2 => write!(f, "{}", date.format("%y")),
                                CalVerFormatSegment::Epoch => write!(f, "{}", date.format("%s")),
                                CalVerFormatSegment::Month => write!(f, "{}", date.format("%m")),
                                CalVerFormatSegment::Day => write!(f, "{}", date.format("%d")),
                                CalVerFormatSegment::DayOfYear => write!(f, "{}", date.format("%j")),
                                CalVerFormatSegment::Hour => write!(f, "{}", date.format("%H")),
                                CalVerFormatSegment::Minute => write!(f, "{}", date.format("%M")),
                                CalVerFormatSegment::Second => write!(f, "{}", date.format("%S")),
                            })
                            .find(|res| res.is_err())
                            .map(|r| r.unwrap_err());

                        match err {
                            Some(e) => Err(e),
                            None => Ok(()),
                        }
                    }
            VersionNumber::Sha(s) => write!(f, "{}", s),
            VersionNumber::ShortSha(s) => write!(f, "{}", s),
        }
    }
}
