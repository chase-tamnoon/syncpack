use log::warn;
use node_semver::Version;
use std::cmp::Ordering;

use super::{semver_range::SemverRange, Specifier, REGEX_CARET, REGEX_GT, REGEX_GTE, REGEX_LT, REGEX_LTE, REGEX_RANGE_CHAR, REGEX_TILDE};

#[derive(Clone, Debug)]
pub struct OrderableSimpleSemver {
  pub range: SemverRange,
  pub version: Version,
}

impl Ord for OrderableSimpleSemver {
  fn cmp(&self, other: &Self) -> Ordering {
    match self.version.major.cmp(&other.version.major) {
      Ordering::Greater => return Ordering::Greater,
      Ordering::Less => return Ordering::Less,
      Ordering::Equal => match self.version.minor.cmp(&other.version.minor) {
        Ordering::Greater => return Ordering::Greater,
        Ordering::Less => return Ordering::Less,
        Ordering::Equal => match self.version.patch.cmp(&other.version.patch) {
          Ordering::Greater => return Ordering::Greater,
          Ordering::Less => return Ordering::Less,
          Ordering::Equal => self.range.cmp(&other.range),
        },
      },
    }
  }
}

impl PartialOrd for OrderableSimpleSemver {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for OrderableSimpleSemver {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(&other) == Ordering::Equal
  }
}

impl Eq for OrderableSimpleSemver {}

#[derive(Clone, Debug)]
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
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Exact(s) => SimpleSemver::Exact(s.clone()),
      Specifier::Latest(s) => SimpleSemver::Latest(s.clone()),
      Specifier::Major(s) => SimpleSemver::Major(s.clone()),
      Specifier::Minor(s) => SimpleSemver::Minor(s.clone()),
      Specifier::Range(s) => SimpleSemver::Range(s.clone()),
      Specifier::RangeMinor(s) => SimpleSemver::RangeMinor(s.clone()),
      _ => panic!("{specifier:?} is not SimpleSemver"),
    }
  }

  pub fn parse(&self) -> OrderableSimpleSemver {
    let range = self.get_range();
    let version = match self {
      SimpleSemver::Exact(s) => Version::parse(s).unwrap(),
      SimpleSemver::Latest(_) => {
        let huge_version = "9999.9999.9999";
        warn!("Cannot parse {self:?} for ordering, working around by treating it as {huge_version}");
        Version::parse(huge_version).unwrap()
      }
      SimpleSemver::Major(s) => Version::parse(&format!("{}.0.0", s)).unwrap(),
      SimpleSemver::Minor(s) => Version::parse(&format!("{}.0", s)).unwrap(),
      SimpleSemver::Range(s) => {
        let exact = REGEX_RANGE_CHAR.replace(s, "");
        Version::parse(&exact).unwrap()
      }
      SimpleSemver::RangeMinor(s) => {
        let exact = REGEX_RANGE_CHAR.replace(s, "");
        Version::parse(&format!("{}.0", exact)).unwrap()
      }
    };
    OrderableSimpleSemver { range, version }
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
        SimpleSemver::new(&Specifier::new(&format!("{next_range}{exact}")))
      }
      SimpleSemver::Major(s) | SimpleSemver::Minor(s) | SimpleSemver::Range(s) | SimpleSemver::RangeMinor(s) => {
        let exact = REGEX_RANGE_CHAR.replace(s, "");
        let next_range = range.unwrap();
        SimpleSemver::new(&Specifier::new(&format!("{next_range}{exact}")))
      }
    }
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
        if REGEX_CARET.is_match(s) {
          return SemverRange::Minor;
        }
        if REGEX_TILDE.is_match(s) {
          return SemverRange::Patch;
        }
        if REGEX_GT.is_match(s) {
          return SemverRange::Gt;
        }
        if REGEX_GTE.is_match(s) {
          return SemverRange::Gte;
        }
        if REGEX_LT.is_match(s) {
          return SemverRange::Lt;
        }
        if REGEX_LTE.is_match(s) {
          return SemverRange::Lte;
        }
        panic!("'{s}' has unrecognised semver range specifier");
      }
    }
  }
}

impl Ord for SimpleSemver {
  fn cmp(&self, other: &Self) -> Ordering {
    self.parse().cmp(&other.parse())
  }
}

impl PartialOrd for SimpleSemver {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for SimpleSemver {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(&other) == Ordering::Equal
  }
}

impl Eq for SimpleSemver {}

#[cfg(test)]
mod tests {
  use super::*;
  use std::cmp::Ordering;

  fn to_strings(specifiers: Vec<&str>) -> Vec<String> {
    specifiers.iter().map(|s| s.to_string()).collect()
  }

  #[test]
  fn parse() {
    let raw = "0.0.0".to_string();
    let semver = SimpleSemver::new(&Specifier::new(&raw));
    let parsed = semver.parse();
    assert_eq!(
      parsed,
      OrderableSimpleSemver {
        range: SemverRange::Exact,
        version: Version {
          major: 0,
          minor: 0,
          patch: 0,
          build: vec![],
          pre_release: vec![],
        },
      }
    );
  }

  #[test]
  fn compare() {
    let cases: Vec<(&str, &str, Ordering)> = vec![
      /* "" */
      ("0.0.0", "0.0.1", Ordering::Less),
      ("0.0.0", "0.1.0", Ordering::Less),
      ("0.0.0", "1.0.0", Ordering::Less),
      ("0.0.0", "0.0.0", Ordering::Equal),
      ("0.0.1", "0.0.0", Ordering::Greater),
      ("0.1.0", "0.0.0", Ordering::Greater),
      ("1.0.0", "0.0.0", Ordering::Greater),
      /* ~ */
      ("0.0.0", "~0.0.1", Ordering::Less),
      ("0.0.0", "~0.1.0", Ordering::Less),
      ("0.0.0", "~1.0.0", Ordering::Less),
      ("0.0.0", "~0.0.0", Ordering::Less),
      ("0.0.1", "~0.0.0", Ordering::Greater),
      ("0.1.0", "~0.0.0", Ordering::Greater),
      ("1.0.0", "~0.0.0", Ordering::Greater),
    ];
    for (str_a, str_b, expected) in cases {
      let a = Specifier::new(&str_a.to_string());
      let a = SimpleSemver::new(&a);
      let b = Specifier::new(&str_b.to_string());
      let b = SimpleSemver::new(&b);
      let ordering = a.cmp(&b);
      assert_eq!(ordering, expected, "{str_a} should {expected:?} {str_b}");
    }
  }
}
