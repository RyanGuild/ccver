use std::cmp::Ordering;

#[derive(Debug, Clone, Default)]
pub struct VersionFormat {
    pub v_prefix: bool,
    pub major: VersionNumberFormat,
    pub minor: VersionNumberFormat,
    pub patch: VersionNumberFormat,
    pub prerelease: Option<PreTagFormat>,
}

impl Default for &VersionFormat {
    fn default() -> Self {
        &DEFAULT_VERSION_FORMAT
    }
}

const DEFAULT_VERSION_FORMAT: VersionFormat = VersionFormat {
    v_prefix: false,
    major: VersionNumberFormat::CCVer,
    minor: VersionNumberFormat::CCVer,
    patch: VersionNumberFormat::CCVer,
    prerelease: None,
};

#[derive(Debug, Clone)]
pub enum VersionNumberFormat {
    CCVer,
    CalVer(CalVerFormat),
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

pub type CalVerFormat = Rc<[CalVerFormatSegment]>;

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
use CalVerFormatSegment::*;

use crate::version::VersionNumber;

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

const DEFAULT_PRE_TAG_FORMAT: PreTagFormat = PreTagFormat::Build(VersionNumberFormat::CCVer);

impl Default for PreTagFormat {
    fn default() -> Self {
        DEFAULT_PRE_TAG_FORMAT
    }
}

impl Default for &PreTagFormat {
    fn default() -> Self {
        &DEFAULT_PRE_TAG_FORMAT
    }
}

impl PreTagFormat {
    pub fn version_format(&self) -> Option<&VersionNumberFormat> {
        match self {
            PreTagFormat::Rc(vf)
            | PreTagFormat::Beta(vf)
            | PreTagFormat::Alpha(vf)
            | PreTagFormat::Build(vf)
            | PreTagFormat::Named(_, vf) => Some(vf),
            PreTagFormat::Sha => None,
            PreTagFormat::ShortSha => None,
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
        }
    }
}
