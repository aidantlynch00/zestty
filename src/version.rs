use std::num::ParseIntError;
use std::cmp::Ordering;
use std::fmt::{self, Display};

#[derive(Default)]
pub struct CompatibilityInfo {
    pub min_version: SemanticVersion,
    pub actual_version: Option<SemanticVersion>,
}

impl CompatibilityInfo {
    pub const fn new(version: Option<SemanticVersion>) -> Self {
        Self {
            min_version: SemanticVersion::new(0, 40, 0),
            actual_version: version,
        }
    }

    pub fn compatible(&self) -> bool {
        match &self.actual_version {
            Some(version) if *version >= self.min_version => true,
            _ => false
        }
    }
}

#[derive(Default, PartialEq, Eq)]
pub struct SemanticVersion {
    major: usize,
    minor: usize,
    patch: usize,
}

impl SemanticVersion {
    pub const fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self { major, minor, patch }
    }
}

pub struct SemanticVersionError;

impl From<ParseIntError> for SemanticVersionError {
    fn from(_value: ParseIntError) -> Self { Self }
}

impl<'a> TryFrom<&'a str> for SemanticVersion
{
    type Error = SemanticVersionError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut versions = value.split(".");
        match (versions.next(), versions.next(), versions.next()) {
            (Some(major), Some(minor), Some(patch)) => Ok(SemanticVersion {
                major: major.parse()?,
                minor: minor.parse()?,
                patch: patch.parse()?,
            }),
            _ => Err(SemanticVersionError)
        }
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // compare major versions
        if self.major > other.major { return Ordering::Greater; }
        if self.major < other.major { return Ordering::Less; }

        // compare minor versions
        if self.minor > other.minor { return Ordering::Greater; }
        if self.minor < other.minor { return Ordering::Less; }

        // compare patch versions
        if self.minor > other.minor { Ordering::Greater }
        else if self.minor < other.minor { Ordering::Less }
        else { Ordering::Equal }
    }
}
