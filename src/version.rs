use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use crate::version_format::{CalVerFormat, CalVerFormatSegment};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Version {
    pub major: VersionNumber,
    pub minor: VersionNumber,
    pub patch: VersionNumber,
    pub prerelease: Option<PreTag>,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        match &self.prerelease {
            None => Ok(()),
            Some(pre) => write!(f, "-{}", pre),
        }
    }
}

impl Version {
    pub fn major(&mut self) -> Self {
        Version {
            major: self.major.bump(),
            minor: self.minor.zero(),
            patch: self.patch.zero(),
            prerelease: None,
        }
    }

    pub fn minor(&mut self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.bump(),
            patch: self.minor.zero(),
            prerelease: None,
        }
    }

    pub fn patch(&mut self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.bump(),
            prerelease: None,
        }
    }

    pub fn build(&mut self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.clone(),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Build(VersionNumber::CCVer(0))),
                Some(pre) => match pre {
                    PreTag::Build(v) => Some(PreTag::Build(v.bump())),
                    _ => Some(PreTag::Build(VersionNumber::CCVer(0))),
                },
            },
        }
    }

    pub fn rc(&self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.clone(),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Rc(VersionNumber::CCVer(0))),
                Some(pre) => match pre {
                    PreTag::Rc(v) => Some(PreTag::Rc(v.bump())),
                    _ => Some(PreTag::Rc(VersionNumber::CCVer(0))),
                },
            },
        }
    }

    pub fn beta(&mut self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.clone(),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Beta(VersionNumber::CCVer(0))),
                Some(pre) => match pre {
                    PreTag::Beta(v) => Some(PreTag::Beta(v.bump())),
                    _ => Some(PreTag::Beta(VersionNumber::CCVer(0))),
                },
            },
        }
    }

    pub fn alpha(&self) -> Version {
        Version {
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.clone(),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Alpha(VersionNumber::CCVer(0))),
                Some(pre) => match pre {
                    PreTag::Alpha(v) => Some(PreTag::Alpha(v.bump())),
                    _ => Some(PreTag::Alpha(VersionNumber::CCVer(0))),
                },
            },
        }
    }

    pub fn named() {}

    pub fn sha() {}
}

impl Default for Version {
    fn default() -> Self {
        Version {
            major: VersionNumber::default(),
            minor: VersionNumber::default(),
            patch: VersionNumber::default(),
            prerelease: Some(PreTag::default()),
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => match &self.prerelease {
                        None => match &other.prerelease {
                            None => Ordering::Equal,
                            Some(_) => Ordering::Greater,
                        },
                        Some(pre) => match &other.prerelease {
                            None => Ordering::Less,
                            Some(other_pre) => match pre.partial_cmp(other_pre) {
                                Some(ord) => ord,
                                None => Ordering::Equal,
                            },
                        },
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
    Sha(String),
    ShortSha(String),
}

impl Display for PreTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

impl Default for PreTag {
    fn default() -> Self {
        PreTag::Build(VersionNumber::CCVer(0))
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
}

impl Default for VersionNumber {
    fn default() -> Self {
        VersionNumber::CCVer(0)
    }
}

impl VersionNumber {
    pub fn bump(&self) -> Self {
        match self {
            VersionNumber::CCVer(v) => VersionNumber::CCVer(*v + 1),
            VersionNumber::CalVer(format, _) => {
                VersionNumber::CalVer(format.clone(), chrono::Utc::now())
            }
        }
    }

    pub fn zero(&self) -> Self {
        match self {
            VersionNumber::CCVer(_) => VersionNumber::CCVer(0),
            VersionNumber::CalVer(_, _) => self.bump(),
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
        }
    }
}

impl Display for VersionNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
        }
    }
}
