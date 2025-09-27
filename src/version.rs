use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use crate::{
    logs::{LogEntry, Subject},
    pattern_macros::*,
    version_format::{
        CalVerFormat, CalVerFormatSegment, PreTagFormat, VersionFormat, VersionNumberFormat,
    },
};

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
            Some(pre) => write!(f, "{}", pre),
        }
    }
}

impl Version {
    pub fn no_pre(&self) -> Version {
        Version {
            v_prefix: self.v_prefix,
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.clone(),
            prerelease: None,
        }
    }
    pub fn next_version<'a>(
        &self,
        log_entry: &LogEntry<'a>,
        version_format: &VersionFormat,
    ) -> Version {
        match (
            &log_entry.subject,
            log_entry.branch,
            log_entry.parent_hashes.len() == 2,
        ) {
            (major_subject!(), release_branches!(), _) => self.major(log_entry, version_format),
            (minor_subject!(), release_branches!(), _) => self.minor(log_entry, version_format),
            (patch_subject!(), release_branches!(), _) => self.patch(log_entry, version_format),
            (Subject::Conventional(_), release_branches!(), _) => {
                self.short_sha(log_entry, version_format)
            }
            (major_subject!(), rc_branches!(), _) => self
                .major(log_entry, version_format)
                .rc(log_entry, version_format),
            (minor_subject!(), rc_branches!(), _) => self
                .minor(log_entry, version_format)
                .rc(log_entry, version_format),
            (patch_subject!(), rc_branches!(), _) => self
                .patch(log_entry, version_format)
                .rc(log_entry, version_format),
            (Subject::Conventional(_), rc_branches!(), _) => self.rc(log_entry, version_format),
            (major_subject!(), beta_branches!(), _) => self
                .major(log_entry, version_format)
                .beta(log_entry, version_format),
            (minor_subject!(), beta_branches!(), _) => self
                .minor(log_entry, version_format)
                .beta(log_entry, version_format),
            (patch_subject!(), beta_branches!(), _) => self
                .patch(log_entry, version_format)
                .beta(log_entry, version_format),
            (Subject::Conventional(_), beta_branches!(), _) => self.beta(log_entry, version_format),
            (major_subject!(), alpha_branches!(), _) => self
                .major(log_entry, version_format)
                .alpha(log_entry, version_format),
            (minor_subject!(), alpha_branches!(), _) => self
                .minor(log_entry, version_format)
                .alpha(log_entry, version_format),
            (patch_subject!(), alpha_branches!(), _) => self
                .patch(log_entry, version_format)
                .alpha(log_entry, version_format),
            (Subject::Conventional(_), alpha_branches!(), _) => {
                self.alpha(log_entry, version_format)
            }
            (major_subject!(), _, _) => self
                .major(log_entry, version_format)
                .named(log_entry, version_format),
            (minor_subject!(), _, _) => self
                .minor(log_entry, version_format)
                .named(log_entry, version_format),
            (patch_subject!(), _, _) => self
                .patch(log_entry, version_format)
                .named(log_entry, version_format),
            (Subject::Conventional(_), _, _) => self.named(log_entry, version_format),
            (Subject::Text(_), release_branches!(), true) => {
                self.release(log_entry, version_format)
            }
            (Subject::Text(_), release_branches!(), _) => self.short_sha(log_entry, version_format),
            (Subject::Text(_), rc_branches!(), _) => self.rc(log_entry, version_format),
            (Subject::Text(_), beta_branches!(), _) => self.beta(log_entry, version_format),
            (Subject::Text(_), alpha_branches!(), _) => self.alpha(log_entry, version_format),
            (Subject::Text(_), _, _) => self.named(log_entry, version_format),
        }
    }

    pub fn major(&self, commit: &LogEntry, version_format: &VersionFormat) -> Self {
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.bump(commit),
            minor: self.minor.zero(commit),
            patch: self.patch.zero(commit),
            prerelease: None,
        }
    }

    pub fn minor(&self, commit: &LogEntry, version_format: &VersionFormat) -> Self {
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.bump(commit),
            patch: self.minor.zero(commit),
            prerelease: None,
        }
    }

    pub fn patch(&self, commit: &LogEntry, version_format: &VersionFormat) -> Self {
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.bump(commit),
            prerelease: None,
        }
    }

    pub fn build(&self, commit: &LogEntry, version_format: &VersionFormat) -> Self {
        let pre_format = version_format.prerelease.as_ref().unwrap_or_default();

        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Build(
                    pre_format
                        .version_format()
                        .as_default_version_number(commit),
                )),
                Some(PreTag::Build(v)) => Some(PreTag::Build(v.bump(commit))),
                Some(_) => Some(PreTag::Build(
                    pre_format
                        .version_format()
                        .as_default_version_number(commit),
                )),
            },
        }
    }

    pub fn rc(&self, commit: &LogEntry, version_format: &VersionFormat) -> Self {
        let pre_format = version_format.prerelease.as_ref().unwrap_or_default();
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: match &self.prerelease {
                Some(PreTag::Rc(v)) => Some(PreTag::Rc(v.bump(commit))),
                _ => Some(PreTag::Rc(
                    pre_format
                        .version_format()
                        .as_default_version_number(commit),
                )),
            },
        }
    }

    pub fn beta(&self, commit: &LogEntry, version_format: &VersionFormat) -> Self {
        let pre_format = version_format.prerelease.as_ref().unwrap_or_default();
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: match &self.prerelease {
                Some(PreTag::Beta(v)) => Some(PreTag::Beta(v.bump(commit))),
                _ => Some(PreTag::Beta(
                    pre_format
                        .version_format()
                        .as_default_version_number(commit),
                )),
            },
        }
    }

    pub fn alpha(&self, commit: &LogEntry, version_format: &VersionFormat) -> Version {
        let pre_format = version_format.prerelease.as_ref().unwrap_or_default();
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: match &self.prerelease {
                Some(PreTag::Alpha(v)) => Some(PreTag::Alpha(v.bump(commit))),
                _ => Some(PreTag::Alpha(
                    pre_format
                        .version_format()
                        .as_default_version_number(commit),
                )),
            },
        }
    }

    pub fn named(&self, commit: &LogEntry, version_format: &VersionFormat) -> Version {
        let pre_format = version_format.prerelease.as_ref().unwrap_or_default();
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: match &self.prerelease {
                Some(PreTag::Named(tag, v)) if tag.eq(commit.branch) => {
                    Some(PreTag::Named(tag.to_string(), v.bump(commit)))
                }
                _ => Some(PreTag::Named(
                    commit.branch.to_string(),
                    pre_format
                        .version_format()
                        .as_default_version_number(commit),
                )),
            },
        }
    }

    pub fn release(&self, commit: &LogEntry, version_format: &VersionFormat) -> Version {
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: None,
        }
    }

    pub fn sha(&self, commit: &LogEntry, version_format: &VersionFormat) -> Version {
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: Some(PreTag::Sha(VersionNumber::Sha(
                commit.commit_hash.to_string(),
            ))),
        }
    }

    pub fn short_sha(&self, commit: &LogEntry, version_format: &VersionFormat) -> Version {
        Version {
            v_prefix: version_format.v_prefix,
            major: self.major.peek(commit),
            minor: self.minor.peek(commit),
            patch: self.patch.peek(commit),
            prerelease: Some(PreTag::ShortSha(VersionNumber::ShortSha(
                commit.commit_hash[0..7].to_string(),
            ))),
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => match (&self.prerelease, &other.prerelease) {
                        (None, None) => Ordering::Equal,
                        (Some(PreTag::Build(_)), None) => Ordering::Greater,
                        (None, Some(PreTag::Build(_))) => Ordering::Less,
                        (Some(PreTag::Build(a)), Some(PreTag::Build(b))) => a.cmp(b),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (Some(a), Some(b)) => match a.partial_cmp(b) {
                            Some(ord) => ord,
                            None => Ordering::Equal,
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
    Sha(VersionNumber),
    ShortSha(VersionNumber),
}

impl Display for PreTag {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PreTag::Rc(v) => write!(f, "-rc.{}", v),
            PreTag::Beta(v) => write!(f, "-beta.{}", v),
            PreTag::Alpha(v) => write!(f, "-alpha.{}", v),
            PreTag::Build(v) => write!(f, "-build.{}", v),
            PreTag::Named(tag, v) => write!(f, "-{}.{}", tag, v),
            PreTag::Sha(s) => write!(f, "+{}", s),
            PreTag::ShortSha(s) => write!(f, "+{}", s),
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

impl From<PreTag> for PreTagFormat {
    fn from(val: PreTag) -> Self {
        match val {
            PreTag::Rc(v) => PreTagFormat::Rc(v.into()),
            PreTag::Beta(v) => PreTagFormat::Beta(v.into()),
            PreTag::Alpha(v) => PreTagFormat::Alpha(v.into()),
            PreTag::Build(v) => PreTagFormat::Build(v.into()),
            PreTag::Named(tag, v) => PreTagFormat::Named(tag, v.into()),
            PreTag::Sha(_) => PreTagFormat::Sha,
            PreTag::ShortSha(_) => PreTagFormat::ShortSha,
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

impl From<Version> for VersionFormat {
    fn from(val: Version) -> Self {
        VersionFormat {
            v_prefix: val.v_prefix,
            major: val.major.into(),
            minor: val.minor.into(),
            patch: val.patch.into(),
            prerelease: val.prerelease.map(|p| p.into()),
        }
    }
}

impl From<VersionNumber> for VersionNumberFormat {
    fn from(val: VersionNumber) -> Self {
        match val {
            VersionNumber::CCVer(_) => VersionNumberFormat::CCVer,
            VersionNumber::CalVer(format, _) => VersionNumberFormat::CalVer(format),
            VersionNumber::Sha(_) => VersionNumberFormat::Sha,
            VersionNumber::ShortSha(_) => VersionNumberFormat::ShortSha,
        }
    }
}

impl VersionNumber {
    pub fn bump(&self, commit: &LogEntry) -> Self {
        match self {
            VersionNumber::CCVer(v) => VersionNumber::CCVer(*v + 1),
            VersionNumber::CalVer(format, _) => {
                VersionNumber::CalVer(format.clone(), commit.commit_datetime)
            }
            VersionNumber::Sha(_) => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumber::ShortSha(_) => {
                VersionNumber::ShortSha(commit.commit_hash[0..7].to_string())
            }
        }
    }

    pub fn peek(&self, commit: &LogEntry) -> Self {
        match self {
            VersionNumber::CCVer(v) => VersionNumber::CCVer(*v),
            VersionNumber::CalVer(format, _) => {
                VersionNumber::CalVer(format.clone(), commit.commit_datetime)
            }
            VersionNumber::Sha(_) => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumber::ShortSha(_) => {
                VersionNumber::ShortSha(commit.commit_hash[0..7].to_string())
            }
        }
    }

    pub fn zero(&self, commit: &LogEntry) -> Self {
        match self {
            VersionNumber::CCVer(_) => VersionNumber::CCVer(0),
            VersionNumber::CalVer(_, _) => self.bump(commit),
            VersionNumber::Sha(_) => VersionNumber::Sha(commit.commit_hash.to_string()),
            VersionNumber::ShortSha(_) => {
                VersionNumber::ShortSha(commit.commit_hash[0..7].to_string())
            }
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
                _ => panic!(
                    "Cannot compare ShortSha Version Number with non ShortSha Version Number"
                ),
            },
        }
    }
}

impl PartialOrd for VersionNumber {
    #[allow(clippy::non_canonical_partial_ord_impl)]
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
