use std::cmp::Ordering;

#[derive(PartialEq, Eq)]
pub struct Version {
    pub major: VersionNumber,
    pub minor: VersionNumber,
    pub patch: VersionNumber,
    pub prerelease: Option<PreTag>,
}

impl Version {
    pub fn major(&mut self) -> Self {
        Version {
            major: self.major.bump(),
            minor: self.minor.zero(),
            patch: self.patch.zero(),
            prerelease: None
        }
    }

    pub fn minor(&mut self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.bump(),
            patch: self.minor.zero(),
            prerelease: None
        }
    }

    pub fn patch(&mut self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.bump(),
            prerelease: None
        }
    }

    pub fn build(&mut self) -> Self {
        Version {
            major: self.major.clone(),
            minor: self.minor.clone(),
            patch: self.patch.clone(),
            prerelease: match &self.prerelease {
                None => Some(PreTag::Build(VersionNumber::CCVer(0))),
                Some(pre)  => match pre {
                    PreTag::Build(v) => Some(PreTag::Build(v.bump())),
                    _ => Some(PreTag::Build(VersionNumber::CCVer(0)))

                }
            }
        }
    }

    pub fn rc() {

    }

    pub fn beta() {

    }

    pub fn alpha() {

    }

    pub fn named() {

    }

    pub fn sha() {

    }


}

impl Default for Version {
    fn default() -> Self {
        Version {
            major: VersionNumber::CCVer(0),
            minor: VersionNumber::CCVer(0),
            patch: VersionNumber::CCVer(0),
            prerelease: None,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.major.partial_cmp(&other.major) {
            Some(Ordering::Equal) => match self.minor.partial_cmp(&other.minor) {
                Some(Ordering::Equal) => match self.patch.partial_cmp(&other.patch) {
                    Some(Ordering::Equal) => self.prerelease.partial_cmp(&other.prerelease),
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum PreTag {
    Rc(VersionNumber),
    Beta(VersionNumber),
    Alpha(VersionNumber),
    Build(VersionNumber),
    Named(String, VersionNumber),
    Sha(String),
}

impl PartialOrd for PreTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            PreTag::Rc(v) => match other {
                PreTag::Rc(v2) => v.partial_cmp(v2),
                PreTag::Alpha(_) | PreTag::Beta(_) | PreTag::Build(_) => Some(Ordering::Greater),
                _ => None
            },
            PreTag::Beta(v) => match other {
                PreTag::Rc(_) => Some(Ordering::Less),
                PreTag::Beta(v2) => v.partial_cmp(v2),
                PreTag::Alpha(_) | PreTag::Build(_) => Some(Ordering::Greater),
                _ => None
            },
            PreTag::Alpha(v) => match other {
                PreTag::Rc(_) | PreTag::Beta(_) => Some(Ordering::Less),
                PreTag::Alpha(v2) => v.partial_cmp(v2),
                PreTag::Build(_) => Some(Ordering::Greater),
                _ => None
            },
            PreTag::Named(tag, v) => match other {
                PreTag::Named(tag2, v2) => {
                    if tag == tag2 {
                        v.partial_cmp(v2)
                    } else {
                        None
                    }
                },
                _ => None
            }
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum VersionNumber {
    CCVer(usize),
    CalVer(String, chrono::DateTime<chrono::Utc>),
}


impl VersionNumber {
    pub fn bump(& self) -> Self {
        match self {
            VersionNumber::CCVer(v) => VersionNumber::CCVer(*v + 1),
            VersionNumber::CalVer(format,_) => VersionNumber::CalVer(format.to_string(), chrono::Utc::now()),
        }
    }

    pub fn zero(& self) -> Self {
        match self {
            VersionNumber::CCVer(_) => VersionNumber::CCVer(0),
            VersionNumber::CalVer(_, _) => self.bump()
        }
    }
}

impl PartialOrd for VersionNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            VersionNumber::CCVer(ver) => match other {
                VersionNumber::CCVer(ver2) => ver.partial_cmp(ver2),
                _ => None,
            },
            VersionNumber::CalVer(format, date) => match other {
                VersionNumber::CalVer(format, date2) => date.partial_cmp(date2),
                _ => None,
            },
        }
    }
}
