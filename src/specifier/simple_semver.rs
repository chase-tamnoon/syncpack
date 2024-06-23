use super::Specifier;
use super::REGEX_CARET;
use super::REGEX_GT;
use super::REGEX_GTE;
use super::REGEX_LT;
use super::REGEX_LTE;
use super::REGEX_RANGE_CHAR;
use super::REGEX_TILDE;
use crate::semver_range::SemverRange;
use std::cmp::Ordering;

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
    Ordering::Equal
  }
}

impl PartialOrd for SimpleSemver {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for SimpleSemver {
  fn eq(&self, other: &Self) -> bool {
    true
  }
}

impl Eq for SimpleSemver {}
