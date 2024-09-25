use log::warn;
use node_semver::Version;

use super::{
  orderable::{IsOrderable, Orderable},
  parser,
  regexes::{
    CARET, CARET_MINOR, CARET_TAG, GT, GTE, GTE_TAG, GT_TAG, LT, LTE, LTE_TAG, LT_TAG, RANGE_CHARS,
    TILDE, TILDE_MINOR, TILDE_TAG,
  },
  semver_range::SemverRange,
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
  /// eg. `^1.2`
  RangeMinor(String),
}

impl SimpleSemver {
  pub fn new(specifier: &String) -> Self {
    let str = parser::sanitise(specifier);
    let string = str.to_string();
    if parser::is_exact(str) {
      Self::Exact(string)
    } else if parser::is_latest(str) {
      Self::Latest(string)
    } else if parser::is_major(str) {
      Self::Major(string)
    } else if parser::is_minor(str) {
      Self::Minor(string)
    } else if parser::is_range(str) {
      Self::Range(string)
    } else if parser::is_range_minor(str) {
      Self::RangeMinor(string)
    } else {
      panic!("{specifier:?} is not SimpleSemver");
    }
  }

  /// Replace this version's semver range with another one
  pub fn with_range(&self, range: &SemverRange) -> SimpleSemver {
    match self {
      SimpleSemver::Latest(_) => {
        warn!("Cannot convert {self:?} to {range:?}, keeping as is");
        self.clone()
      }
      SimpleSemver::Exact(exact) => {
        let next_range = range.unwrap();
        SimpleSemver::new(&format!("{next_range}{exact}"))
      }
      SimpleSemver::Major(s)
      | SimpleSemver::Minor(s)
      | SimpleSemver::Range(s)
      | SimpleSemver::RangeMinor(s) => {
        let exact = RANGE_CHARS.replace(s, "");
        let next_range = range.unwrap();
        SimpleSemver::new(&format!("{next_range}{exact}"))
      }
    }
  }

  pub fn has_same_range(&self, other: &SimpleSemver) -> bool {
    self.get_range() == other.get_range()
  }

  pub fn has_same_version(&self, other: &SimpleSemver) -> bool {
    self.get_orderable().version == other.get_orderable().version
  }

  /// Get the semver range of this version, a simple semver specifier always has
  /// a semver range, even if it's `Exact`
  pub fn get_range(&self) -> SemverRange {
    match self {
      SimpleSemver::Exact(s) => SemverRange::Exact,
      SimpleSemver::Latest(s) => SemverRange::Any,
      SimpleSemver::Major(s) => SemverRange::Exact,
      SimpleSemver::Minor(s) => SemverRange::Exact,
      SimpleSemver::Range(s) | SimpleSemver::RangeMinor(s) => {
        if CARET.is_match(s) || CARET_MINOR.is_match(s) || CARET_TAG.is_match(s) {
          return SemverRange::Minor;
        }
        if TILDE.is_match(s) || TILDE_MINOR.is_match(s) || TILDE_TAG.is_match(s) {
          return SemverRange::Patch;
        }
        if GT.is_match(s) || GT_TAG.is_match(s) {
          return SemverRange::Gt;
        }
        if GTE.is_match(s) || GTE_TAG.is_match(s) {
          return SemverRange::Gte;
        }
        if LT.is_match(s) || LT_TAG.is_match(s) {
          return SemverRange::Lt;
        }
        if LTE.is_match(s) || LTE_TAG.is_match(s) {
          return SemverRange::Lte;
        }
        panic!("'{s}' has unrecognised semver range specifier");
      }
    }
  }
}

impl IsOrderable for SimpleSemver {
  /// Parse this version specifier into a struct w can compare and order
  fn get_orderable(&self) -> Orderable {
    let range = self.get_range();
    let version = match self {
      SimpleSemver::Exact(s) => Version::parse(s).unwrap(),
      SimpleSemver::Latest(_) => {
        let huge_version = "9999.9999.9999";
        warn!(
          "Cannot parse {self:?} for ordering, working around by treating it as {huge_version}"
        );
        Version::parse(huge_version).unwrap()
      }
      SimpleSemver::Major(s) => Version::parse(format!("{}.0.0", s)).unwrap(),
      SimpleSemver::Minor(s) => Version::parse(format!("{}.0", s)).unwrap(),
      SimpleSemver::Range(s) => {
        let exact = RANGE_CHARS.replace(s, "");
        Version::parse(exact).unwrap()
      }
      SimpleSemver::RangeMinor(s) => {
        let exact = RANGE_CHARS.replace(s, "");
        Version::parse(format!("{}.0", exact)).unwrap()
      }
    };
    Orderable { range, version }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use node_semver::{Identifier, Version};

  fn to_strings(specifiers: Vec<&str>) -> Vec<String> {
    specifiers.iter().map(|s| s.to_string()).collect()
  }

  #[test]
  fn get_orderable() {
    let cases: Vec<(&str, Orderable)> = vec![
      (
        "0.0.0",
        Orderable {
          range: SemverRange::Exact,
          version: Version {
            major: 0,
            minor: 0,
            patch: 0,
            build: vec![],
            pre_release: vec![],
          },
        },
      ),
      (
        "1.2.3-alpha",
        Orderable {
          range: SemverRange::Exact,
          version: Version {
            major: 1,
            minor: 2,
            patch: 3,
            build: vec![],
            pre_release: vec![Identifier::AlphaNumeric("alpha".to_string())],
          },
        },
      ),
      (
        "1.2.3-rc.18",
        Orderable {
          range: SemverRange::Exact,
          version: Version {
            major: 1,
            minor: 2,
            patch: 3,
            build: vec![],
            pre_release: vec![
              Identifier::AlphaNumeric("rc".to_string()),
              Identifier::Numeric(18),
            ],
          },
        },
      ),
    ];
    for (str, expected) in cases {
      let raw = str.to_string();
      let semver = SimpleSemver::new(&raw);
      let orderable = semver.get_orderable();
      assert_eq!(orderable.range, expected.range, "range");
      assert_eq!(
        orderable.version.major, expected.version.major,
        "version.major"
      );
      assert_eq!(
        orderable.version.minor, expected.version.minor,
        "version.minor"
      );
      assert_eq!(
        orderable.version.patch, expected.version.patch,
        "version.patch"
      );
      assert_eq!(
        orderable.version.build, expected.version.build,
        "version.build"
      );
      assert_eq!(
        orderable.version.pre_release, expected.version.pre_release,
        "version.pre_release"
      );
    }
  }

  #[test]
  fn has_same_range() {
    let cases: Vec<(&str, &str, bool)> = vec![
      ("0.0.0", "0.0.1", true),
      ("0.0.0", "^0.0.0", false),
      ("^0.0.0", "^0.0.0", true),
      ("^0.0.0", "~0.0.0", false),
      ("*", "*", true),
      ("*", "latest", true),
      ("^0.0.0", "^0.0", true),
      ("0.0.0", "^0.0", false),
      ("~0.0.0", "^0.0", false),
      (">=0.0.0", ">=0.0.0", true),
      (">=0.0.0", ">0.0.0", false),
      (">0.0.0", ">=0.0.0", false),
      ("<=0.0.0", "<=0.0.0", true),
      ("<=0.0.0", "<0.0.0", false),
      ("<0.0.0", "<=0.0.0", false),
    ];
    for (str_a, str_b, expected) in cases {
      let a = SimpleSemver::new(&str_a.to_string());
      let b = SimpleSemver::new(&str_b.to_string());
      assert_eq!(
        a.has_same_range(&b),
        expected,
        "{str_a} has same range as {str_b} should be {expected}"
      );
    }
  }
}
