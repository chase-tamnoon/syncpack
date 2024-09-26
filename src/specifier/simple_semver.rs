use log::warn;
use node_semver::Version;

use crate::specifier::regexes::matches_any;

use super::{
  orderable::{IsOrderable, Orderable},
  parser,
  regexes::{
    CARET, CARET_MAJOR, CARET_MINOR, CARET_TAG, GT, GTE, GTE_MAJOR, GTE_MINOR, GTE_TAG, GT_MAJOR, GT_MINOR, GT_TAG, LT,
    LTE, LTE_MAJOR, LTE_MINOR, LTE_TAG, LT_MAJOR, LT_MINOR, LT_TAG, RANGE_CHARS, TILDE, TILDE_MAJOR, TILDE_MINOR,
    TILDE_TAG,
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
  /// eg. `>1`
  RangeMajor(String),
  /// eg. `^1.2`
  RangeMinor(String),
}

impl SimpleSemver {
  pub fn new(specifier: &String) -> Result<Self, String> {
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
    } else if parser::is_range_major(str) {
      Ok(Self::RangeMajor(string))
    } else if parser::is_range_minor(str) {
      Ok(Self::RangeMinor(string))
    } else {
      Err(format!(
        "'{specifier}' was expected to be a simple semver specifier but was not recognised"
      ))
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
        let next_range = range.unwrap();
        let next_specifier = format!("{next_range}{exact}");
        SimpleSemver::new(&next_specifier).unwrap()
      }
      SimpleSemver::Minor(string)
      | SimpleSemver::Range(string)
      | SimpleSemver::RangeMajor(string)
      | SimpleSemver::RangeMinor(string) => {
        let exact = RANGE_CHARS.replace(string, "");
        let next_range = range.unwrap();
        let next_specifier = format!("{next_range}{exact}");
        SimpleSemver::new(&next_specifier).unwrap()
      }
    }
  }

  /// Does this specifier and the other both have eg "^" as their range?
  pub fn has_same_range(&self, other: &SimpleSemver) -> bool {
    self.get_range() == other.get_range()
  }

  /// Regardless of the range, does this specifier and the other both have eg.
  /// "1.4.1" as their version?
  pub fn has_same_version(&self, other: &SimpleSemver) -> bool {
    self.get_orderable().version == other.get_orderable().version
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
        if matches_any(vec![&CARET, &CARET_MAJOR, &CARET_MINOR, &CARET_TAG], string) {
          return SemverRange::Minor;
        }
        if matches_any(vec![&TILDE, &TILDE_MAJOR, &TILDE_MINOR, &TILDE_TAG], string) {
          return SemverRange::Patch;
        }
        if matches_any(vec![&GT, &GT_MAJOR, &GT_MINOR, &GT_TAG], string) {
          return SemverRange::Gt;
        }
        if matches_any(vec![&GTE, &GTE_MAJOR, &GTE_MINOR, &GTE_TAG], string) {
          return SemverRange::Gte;
        }
        if matches_any(vec![&LT, &LT_MAJOR, &LT_MINOR, &LT_TAG], string) {
          return SemverRange::Lt;
        }
        if matches_any(vec![&LTE, &LTE_MAJOR, &LTE_MINOR, &LTE_TAG], string) {
          return SemverRange::Lte;
        }
        panic!("failed to find a recognised semver range in specifier '{string}'");
      }
    }
  }
}

impl IsOrderable for SimpleSemver {
  /// Parse this version specifier into a struct we can compare and order
  fn get_orderable(&self) -> Orderable {
    let range = self.get_range();
    let huge = "999999";
    Orderable {
      range,
      version: Version::parse(match self {
        Self::Exact(s) => s.clone(),
        Self::Latest(_) => format!("{huge}.{huge}.{huge}"),
        Self::Major(s) => format!("{}.{huge}.{huge}", s),
        Self::Minor(s) => format!("{}.{huge}", s),
        Self::Range(s) => RANGE_CHARS.replace(s, "").to_string(),
        Self::RangeMajor(s) => format!("{}.{huge}.{huge}", RANGE_CHARS.replace(s, "")),
        Self::RangeMinor(s) => format!("{}.{huge}", RANGE_CHARS.replace(s, "")),
      })
      .unwrap(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use node_semver::{Identifier, Version};

  #[test]
  fn returns_err_when_specifier_is_not_simple_semver() {
    assert_eq!(
      SimpleSemver::new(&"<2 || >3".to_string()),
      Err("'<2 || >3' was expected to be a simple semver specifier but was not recognised".to_string())
    );
  }

  #[test]
  fn returns_struct_for_comparison_and_sorting() {
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
            pre_release: vec![Identifier::AlphaNumeric("rc".to_string()), Identifier::Numeric(18)],
          },
        },
      ),
    ];
    for (str, expected) in cases {
      let raw = str.to_string();
      let semver = SimpleSemver::new(&raw).unwrap();
      let orderable = semver.get_orderable();
      assert_eq!(orderable.range, expected.range, "range");
      assert_eq!(orderable.version.major, expected.version.major, "version.major");
      assert_eq!(orderable.version.minor, expected.version.minor, "version.minor");
      assert_eq!(orderable.version.patch, expected.version.patch, "version.patch");
      assert_eq!(orderable.version.build, expected.version.build, "version.build");
      assert_eq!(
        orderable.version.pre_release, expected.version.pre_release,
        "version.pre_release"
      );
    }
  }

  #[test]
  fn replaces_the_semver_range_of_a_specifier() {
    let cases: Vec<(&str, SemverRange, SimpleSemver)> = vec![
      // from exact
      ("0.0.0", SemverRange::Any, SimpleSemver::Latest("*".to_string())),
      ("0.0.0", SemverRange::Minor, SimpleSemver::Range("^0.0.0".to_string())),
      ("0.0.0", SemverRange::Exact, SimpleSemver::Exact("0.0.0".to_string())),
      ("0.0.0", SemverRange::Gt, SimpleSemver::Range(">0.0.0".to_string())),
      ("0.0.0", SemverRange::Gte, SimpleSemver::Range(">=0.0.0".to_string())),
      ("0.0.0", SemverRange::Lt, SimpleSemver::Range("<0.0.0".to_string())),
      ("0.0.0", SemverRange::Lte, SimpleSemver::Range("<=0.0.0".to_string())),
      ("0.0.0", SemverRange::Patch, SimpleSemver::Range("~0.0.0".to_string())),
      // from another range
      ("~0.0.0", SemverRange::Any, SimpleSemver::Latest("*".to_string())),
      ("~0.0.0", SemverRange::Minor, SimpleSemver::Range("^0.0.0".to_string())),
      ("~0.0.0", SemverRange::Exact, SimpleSemver::Exact("0.0.0".to_string())),
      ("~0.0.0", SemverRange::Gt, SimpleSemver::Range(">0.0.0".to_string())),
      ("~0.0.0", SemverRange::Gte, SimpleSemver::Range(">=0.0.0".to_string())),
      ("~0.0.0", SemverRange::Lt, SimpleSemver::Range("<0.0.0".to_string())),
      ("~0.0.0", SemverRange::Lte, SimpleSemver::Range("<=0.0.0".to_string())),
      ("~0.0.0", SemverRange::Patch, SimpleSemver::Range("~0.0.0".to_string())),
      // from major
      ("0", SemverRange::Any, SimpleSemver::Latest("*".to_string())),
      ("0", SemverRange::Minor, SimpleSemver::Major("0".to_string())),
      ("0", SemverRange::Exact, SimpleSemver::Major("0".to_string())),
      ("0", SemverRange::Gt, SimpleSemver::Major("0".to_string())),
      ("0", SemverRange::Gte, SimpleSemver::Major("0".to_string())),
      ("0", SemverRange::Lt, SimpleSemver::Major("0".to_string())),
      ("0", SemverRange::Lte, SimpleSemver::Major("0".to_string())),
      ("0", SemverRange::Patch, SimpleSemver::Major("0".to_string())),
      // from minor
      ("0.0", SemverRange::Any, SimpleSemver::Latest("*".to_string())),
      ("0.0", SemverRange::Minor, SimpleSemver::RangeMinor("^0.0".to_string())),
      ("0.0", SemverRange::Exact, SimpleSemver::Minor("0.0".to_string())),
      ("0.0", SemverRange::Gt, SimpleSemver::RangeMinor(">0.0".to_string())),
      ("0.0", SemverRange::Gte, SimpleSemver::RangeMinor(">=0.0".to_string())),
      ("0.0", SemverRange::Lt, SimpleSemver::RangeMinor("<0.0".to_string())),
      ("0.0", SemverRange::Lte, SimpleSemver::RangeMinor("<=0.0".to_string())),
      ("0.0", SemverRange::Patch, SimpleSemver::RangeMinor("~0.0".to_string())),
      // from another range minor
      ("^0.0", SemverRange::Any, SimpleSemver::Latest("*".to_string())),
      ("^0.0", SemverRange::Minor, SimpleSemver::RangeMinor("^0.0".to_string())),
      ("^0.0", SemverRange::Exact, SimpleSemver::Minor("0.0".to_string())),
      ("^0.0", SemverRange::Gt, SimpleSemver::RangeMinor(">0.0".to_string())),
      ("^0.0", SemverRange::Gte, SimpleSemver::RangeMinor(">=0.0".to_string())),
      ("^0.0", SemverRange::Lt, SimpleSemver::RangeMinor("<0.0".to_string())),
      ("^0.0", SemverRange::Lte, SimpleSemver::RangeMinor("<=0.0".to_string())),
      ("^0.0", SemverRange::Patch, SimpleSemver::RangeMinor("~0.0".to_string())),
    ];
    for (before, range, expected) in cases {
      let semver = SimpleSemver::new(&before.to_string()).unwrap();
      let after = semver.with_range(&range);
      assert_eq!(after, expected);
    }
  }

  #[test]
  fn cannot_replace_the_semver_range_of_latest_since_the_version_is_not_known() {
    let before = SimpleSemver::new(&"*".to_string()).unwrap();
    let after = before.with_range(&SemverRange::Exact);
    assert_eq!(after, SimpleSemver::Latest("*".to_string()));
  }

  #[test]
  fn asserts_whether_two_specifiers_have_same_range() {
    let cases: Vec<(&str, &str, bool)> = vec![
      ("0.0.0", "0.0.1", true),
      ("0.0.0", "^0.0.0", false),
      ("^0.0.0", "^0.0.0", true),
      ("^0.0.0", "~0.0.0", false),
      ("0", "0", true),
      ("0.0", "0.0", true),
      ("0.0", "^0.0", false),
      ("^0.0", "^0.0", true),
      ("^0.0", "~0.0", false),
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
      let a = SimpleSemver::new(&str_a.to_string()).unwrap();
      let b = SimpleSemver::new(&str_b.to_string()).unwrap();
      assert_eq!(
        a.has_same_range(&b),
        expected,
        "{str_a} has same range as {str_b} should be {expected}"
      );
    }
  }
}
