#[cfg(test)]
#[path = "simple_semver_test.rs"]
mod simple_semver_test;

use {
  super::{
    orderable::{IsOrderable, Orderable},
    parser,
    regexes::{
      CARET, CARET_MAJOR, CARET_MINOR, CARET_TAG, GT, GTE, GTE_MAJOR, GTE_MINOR, GTE_TAG, GT_MAJOR, GT_MINOR, GT_TAG, LT, LTE, LTE_MAJOR, LTE_MINOR, LTE_TAG, LT_MAJOR, LT_MINOR, LT_TAG, RANGE_CHARS, TILDE, TILDE_MAJOR, TILDE_MINOR,
      TILDE_TAG,
    },
    semver_range::SemverRange,
  },
  crate::specifier::regexes,
  log::warn,
  node_semver::Version,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SimpleSemver {
  /// eg. `1.2.3`
  Exact(String),
  /// eg. `*`
  Latest(String),
  /// eg. `1`
  Major(String),
  /// eg. `1.2`
  Minor(String),
  /// eg. `>1.2.3`
  Range(String),
  /// eg. `^`
  /// Occurs when `workspace:^` is found and workspace is stripped
  RangeOnly(String),
  /// eg. `>1`
  RangeMajor(String),
  /// eg. `^1.2`
  RangeMinor(String),
}

impl SimpleSemver {
  pub fn new(specifier: &str) -> Result<Self, String> {
    let str = parser::sanitise(specifier);
    let string = str.to_string();
    if parser::is_exact(str) {
      Ok(Self::Exact(string))
    } else if parser::is_latest(str) {
      Ok(Self::Latest(string))
    } else if parser::is_major(str) {
      Ok(Self::Major(string))
    } else if parser::is_minor(str) {
      Ok(Self::Minor(string))
    } else if parser::is_range(str) {
      Ok(Self::Range(string))
    } else if parser::is_range_only(str) {
      Ok(Self::RangeOnly(string))
    } else if parser::is_range_major(str) {
      Ok(Self::RangeMajor(string))
    } else if parser::is_range_minor(str) {
      Ok(Self::RangeMinor(string))
    } else {
      Err(format!("'{specifier}' was expected to be a simple semver specifier but was not recognised"))
    }
  }

  /// Replace this version's semver range with another one
  pub fn with_range(&self, range: &SemverRange) -> SimpleSemver {
    if matches!(range, SemverRange::Any) {
      return SimpleSemver::Latest("*".to_string());
    }
    match self {
      SimpleSemver::Major(_) | SimpleSemver::Latest(_) => {
        warn!("Cannot convert {self:?} to {range:?}, keeping as is");
        self.clone()
      }
      SimpleSemver::Exact(exact) => {
        let next_range = range.get_raw();
        let next_specifier = format!("{next_range}{exact}");
        SimpleSemver::new(&next_specifier).unwrap()
      }
      SimpleSemver::Minor(string) | SimpleSemver::Range(string) | SimpleSemver::RangeMajor(string) | SimpleSemver::RangeMinor(string) => {
        let exact = RANGE_CHARS.replace(string, "");
        let next_range = range.get_raw();
        let next_specifier = format!("{next_range}{exact}");
        SimpleSemver::new(&next_specifier).unwrap()
      }
      SimpleSemver::RangeOnly(_) => {
        let next_range = range.get_raw();
        SimpleSemver::new(&next_range).unwrap()
      }
    }
  }

  /// Does this specifier have the given semver range?
  pub fn has_semver_range_of(&self, range: &SemverRange) -> bool {
    self.get_range() == *range
  }

  /// Regardless of the range, does this specifier and the other both have eg.
  /// "1.4.1" as their version?
  pub fn has_same_version_number_as(&self, other: &SimpleSemver) -> bool {
    self.get_orderable(None).version == other.get_orderable(None).version
  }

  /// Get the semver range of this version, a simple semver specifier always has
  /// a semver range, even if it's `Exact`
  pub fn get_range(&self) -> SemverRange {
    match self {
      SimpleSemver::Exact(_) => SemverRange::Exact,
      SimpleSemver::Latest(_) => SemverRange::Any,
      SimpleSemver::Major(_) => SemverRange::Exact,
      SimpleSemver::Minor(_) => SemverRange::Exact,
      SimpleSemver::Range(string) | SimpleSemver::RangeMajor(string) | SimpleSemver::RangeMinor(string) => {
        if regexes::matches_any(vec![&CARET, &CARET_MAJOR, &CARET_MINOR, &CARET_TAG], string) {
          return SemverRange::Minor;
        }
        if regexes::matches_any(vec![&TILDE, &TILDE_MAJOR, &TILDE_MINOR, &TILDE_TAG], string) {
          return SemverRange::Patch;
        }
        if regexes::matches_any(vec![&GT, &GT_MAJOR, &GT_MINOR, &GT_TAG], string) {
          return SemverRange::Gt;
        }
        if regexes::matches_any(vec![&GTE, &GTE_MAJOR, &GTE_MINOR, &GTE_TAG], string) {
          return SemverRange::Gte;
        }
        if regexes::matches_any(vec![&LT, &LT_MAJOR, &LT_MINOR, &LT_TAG], string) {
          return SemverRange::Lt;
        }
        if regexes::matches_any(vec![&LTE, &LTE_MAJOR, &LTE_MINOR, &LTE_TAG], string) {
          return SemverRange::Lte;
        }
        panic!("failed to find a recognised semver range in specifier '{string}'");
      }
      SimpleSemver::RangeOnly(string) => SemverRange::new(string).unwrap(),
    }
  }

  pub fn get_raw(&self) -> String {
    match self {
      Self::Exact(raw) | Self::Latest(raw) | Self::Major(raw) | Self::Minor(raw) | Self::Range(raw) | Self::RangeOnly(raw) | Self::RangeMajor(raw) | Self::RangeMinor(raw) => raw.clone(),
    }
  }
}

impl IsOrderable for SimpleSemver {
  /// Parse this version specifier into a struct we can compare and order
  fn get_orderable(&self, canonical_specifier: Option<&SimpleSemver>) -> Orderable {
    let get_canonical_version = || {
      canonical_specifier.map(|s| s.get_orderable(None).version).unwrap_or_else(|| Version {
        major: 999999,
        minor: 999999,
        patch: 999999,
        build: vec![],
        pre_release: vec![],
      })
    };
    Orderable {
      range: self.get_range(),
      version: match self {
        Self::Exact(_) | Self::Range(_) => {
          let exact = self.with_range(&SemverRange::Exact).get_raw();
          Version::parse(exact).unwrap()
        }
        Self::Latest(_) | Self::RangeOnly(_) => get_canonical_version(),
        Self::Major(_) | Self::RangeMajor(_) => {
          let exact = self.with_range(&SemverRange::Exact).get_raw();
          let major = exact.split('.').next().unwrap();
          let major_u64 = major.parse::<u64>().unwrap();
          Version { major: major_u64, ..get_canonical_version() }
        }
        Self::Minor(_) | Self::RangeMinor(_) => {
          let exact = self.with_range(&SemverRange::Exact).get_raw();
          let mut split = exact.split('.');
          let major = split.next().unwrap();
          let minor = split.next().unwrap();
          let major_u64 = major.parse::<u64>().unwrap();
          let minor_u64 = minor.parse::<u64>().unwrap();
          Version {
            major: major_u64,
            minor: minor_u64,
            ..get_canonical_version()
          }
        }
      },
    }
  }
}
