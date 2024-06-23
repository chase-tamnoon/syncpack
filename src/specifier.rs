use std::cmp::Ordering;

use lazy_static::lazy_static;
use log::{debug, warn};
use node_semver::Version;
use regex::Regex;

use crate::semver_range::SemverRange;

lazy_static! {
  /// Any character used in a semver range
  static ref REGEX_RANGE_CHAR: Regex = Regex::new(r"[~><=*^]").unwrap();
  /// "1.2.3"
  static ref REGEX_EXACT: Regex = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
  /// "^1.2.3"
  static ref REGEX_CARET: Regex = Regex::new(r"^\^(\d+\.\d+\.\d+)$").unwrap();
  /// "~1.2.3"
  static ref REGEX_TILDE: Regex = Regex::new(r"^~(\d+\.\d+\.\d+)$").unwrap();
  /// ">1.2.3"
  static ref REGEX_GT: Regex = Regex::new(r"^>(\d+\.\d+\.\d+)$").unwrap();
  /// ">=1.2.3"
  static ref REGEX_GTE: Regex = Regex::new(r"^>=(\d+\.\d+\.\d+)$").unwrap();
  /// "<1.2.3"
  static ref REGEX_LT: Regex = Regex::new(r"^<(\d+\.\d+\.\d+)$").unwrap();
  /// "<=1.2.3"
  static ref REGEX_LTE: Regex = Regex::new(r"^<=(\d+\.\d+\.\d+)$").unwrap();
  /// "^1.2"
  static ref REGEX_CARET_MINOR: Regex = Regex::new(r"^\^(\d+\.\d+)$").unwrap();
  /// "~1.2"
  static ref REGEX_TILDE_MINOR: Regex = Regex::new(r"^~(\d+\.\d+)$").unwrap();
  /// ">1.2"
  static ref REGEX_GT_MINOR: Regex = Regex::new(r"^>(\d+\.\d+)$").unwrap();
  /// ">=1.2"
  static ref REGEX_GTE_MINOR: Regex = Regex::new(r"^>=(\d+\.\d+)$").unwrap();
  /// "<1.2"
  static ref REGEX_LT_MINOR: Regex = Regex::new(r"^<(\d+\.\d+)$").unwrap();
  /// "<=1.2"
  static ref REGEX_LTE_MINOR: Regex = Regex::new(r"^<=(\d+\.\d+)$").unwrap();
  /// "1"
  static ref REGEX_MAJOR: Regex = Regex::new(r"^(\d+)$").unwrap();
  /// "1.2"
  static ref REGEX_MINOR: Regex = Regex::new(r"^(\d+\.\d+)$").unwrap();
  /// "npm:"
  static ref REGEX_ALIAS: Regex = Regex::new(r"^npm:").unwrap();
  /// "file:"
  static ref REGEX_FILE: Regex = Regex::new(r"^file:").unwrap();
  /// "workspace:"
  static ref REGEX_WORKSPACE_PROTOCOL: Regex = Regex::new(r"^workspace:").unwrap();
  /// "https://"
  static ref REGEX_URL: Regex = Regex::new(r"^https?://").unwrap();
  /// "git://"
  static ref REGEX_GIT: Regex = Regex::new(r"^git(\+(ssh|https?))?://").unwrap();
  /// "alpha"
  static ref REGEX_TAG: Regex = Regex::new(r"^[a-zA-Z0-9-]+$").unwrap();
  /// a logical OR in a semver range
  static ref REGEX_OR_OPERATOR:Regex = Regex::new(r" ?\|\| ?").unwrap();
}

#[derive(Clone, Eq, Debug, Hash, PartialEq)]
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

pub enum Semver {
  Simple(SimpleSemver),
  Complex(String),
}

impl Semver {
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Exact(_) | Specifier::Latest(_) | Specifier::Major(_) | Specifier::Minor(_) | Specifier::Range(_) | Specifier::RangeMinor(_) => Semver::Simple(SimpleSemver::new(specifier)),
      Specifier::RangeComplex(s) => Semver::Complex(s.clone()),
      _ => panic!("{specifier:?} is not Semver"),
    }
  }
}

#[derive(Clone, Eq, Debug, Hash, PartialEq)]
pub enum NonSemver {
  /// eg. `npm:1.2.3`
  Alias(String),
  /// eg. `file:./path/to/package`
  File(String),
  /// eg. `git://github.com/user/repo.git`
  Git(String),
  /// eg. `alpha`
  Tag(String),
  /// eg. `{wh}at[the]fuu`
  Unsupported(String),
  /// eg. `https://example.com`
  Url(String),
  /// eg. `workspace:*`
  WorkspaceProtocol(String),
}

impl NonSemver {
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Alias(s) => NonSemver::Alias(s.clone()),
      Specifier::File(s) => NonSemver::File(s.clone()),
      Specifier::Git(s) => NonSemver::Git(s.clone()),
      Specifier::Tag(s) => NonSemver::Tag(s.clone()),
      Specifier::Unsupported(s) => NonSemver::Unsupported(s.clone()),
      Specifier::Url(s) => NonSemver::Url(s.clone()),
      Specifier::WorkspaceProtocol(s) => NonSemver::WorkspaceProtocol(s.clone()),
      _ => panic!("{specifier:?} is not NonSemver"),
    }
  }
}

pub enum SpecifierTree {
  Semver(Semver),
  NonSemver(NonSemver),
  None,
}

impl SpecifierTree {
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Exact(_) | Specifier::Latest(_) | Specifier::Major(_) | Specifier::Minor(_) | Specifier::Range(_) | Specifier::RangeComplex(_) | Specifier::RangeMinor(_) => SpecifierTree::Semver(Semver::new(specifier)),
      Specifier::Alias(_) | Specifier::File(_) | Specifier::Git(_) | Specifier::Tag(_) | Specifier::Unsupported(_) | Specifier::Url(_) | Specifier::WorkspaceProtocol(_) => SpecifierTree::NonSemver(NonSemver::new(specifier)),
      Specifier::None => SpecifierTree::None,
    }
  }
}

/// General purpose version specifier.
///
/// Use `SpecifierTree` for operations which are only applicable to specific
/// types of specifiers (only semver specifiers are sortable, for example)
#[derive(Clone, Eq, Debug, Hash, PartialEq)]
pub enum Specifier {
  // Semver
  Exact(String),
  Latest(String),
  Major(String),
  Minor(String),
  Range(String),
  RangeComplex(String),
  RangeMinor(String),
  // Non Semver
  Alias(String),
  File(String),
  Git(String),
  Tag(String),
  Unsupported(String),
  Url(String),
  WorkspaceProtocol(String),
  /// When reading, appears when a package is missing a .version property
  /// When writing, is used:
  ///
  /// 1. To represent a banned instance's specifier
  /// 2. To represent an unfixable instance's expected specifier
  None,
}

impl Specifier {
  pub fn new(specifier: &String) -> Self {
    Specifier::parse(specifier, false)
  }

  pub fn is_semver(&self) -> bool {
    matches!(SpecifierTree::new(self), SpecifierTree::Semver(_))
  }

  pub fn matches(&self, specifier: &Specifier) -> bool {
    *self == *specifier
  }

  /// Get the `specifier_type` name as used in config files.
  pub fn get_type_name(&self) -> String {
    match self {
      &Specifier::Exact(_) => "exact",
      &Specifier::Latest(_) => "latest",
      &Specifier::Major(_) => "major",
      &Specifier::Minor(_) => "minor",
      &Specifier::Range(_) => "range",
      &Specifier::RangeMinor(_) => "range-minor",
      &Specifier::RangeComplex(_) => "range-complex",
      &Specifier::Alias(_) => "alias",
      &Specifier::File(_) => "file",
      &Specifier::Git(_) => "git",
      &Specifier::Tag(_) => "tag",
      &Specifier::Unsupported(_) => "unsupported",
      &Specifier::Url(_) => "url",
      &Specifier::WorkspaceProtocol(_) => "workspace-protocol",
      &Specifier::None => "missing",
    }
    .to_string()
  }

  pub fn get_semver_range(&self) -> Option<SemverRange> {
    let specifier = self.unwrap();
    if specifier == "*" {
      return Some(SemverRange::Any);
    }
    if REGEX_EXACT.is_match(specifier) {
      return Some(SemverRange::Exact);
    }
    if REGEX_CARET.is_match(specifier) {
      return Some(SemverRange::Minor);
    }
    if REGEX_TILDE.is_match(specifier) {
      return Some(SemverRange::Patch);
    }
    if REGEX_GT.is_match(specifier) {
      return Some(SemverRange::Gt);
    }
    if REGEX_GTE.is_match(specifier) {
      return Some(SemverRange::Gte);
    }
    if REGEX_LT.is_match(specifier) {
      return Some(SemverRange::Lt);
    }
    if REGEX_LTE.is_match(specifier) {
      return Some(SemverRange::Lte);
    }
    return None;
  }

  pub fn has_range(&self, expected_range: &SemverRange) -> bool {
    self.get_semver_range().map_or(false, |range| range == *expected_range)
  }

  pub fn get_exact(&self) -> Self {
    self.with_semver_range(&SemverRange::Exact)
  }

  pub fn with_semver_range(&self, range: &SemverRange) -> Self {
    let replace = |current_range: &str| {
      let specifier = self.unwrap();
      let next_range = range.unwrap();
      Specifier::parse(&specifier.replace(current_range, &next_range), false)
    };
    match self.get_semver_range() {
      Some(SemverRange::Exact) => {
        let specifier = self.unwrap();
        let next_range = range.unwrap();
        return Specifier::parse(&format!("{}{}", &next_range, &specifier), false);
      }
      Some(SemverRange::Minor) => {
        return replace("^");
      }
      Some(SemverRange::Patch) => {
        return replace("~");
      }
      Some(SemverRange::Gt) => {
        return replace(">");
      }
      Some(SemverRange::Gte) => {
        return replace(">=");
      }
      Some(SemverRange::Lt) => {
        return replace("<");
      }
      Some(SemverRange::Lte) => {
        return replace("<=");
      }
      Some(SemverRange::Any) => {
        return Specifier::Latest("*".to_string());
      }
      None => {
        panic!("Cannot set a semver range on a non-semver specifier: {:?}", self);
      }
    }
  }

  /// Get the raw specifier value
  pub fn unwrap(&self) -> &String {
    match &self {
      &Specifier::Exact(specifier) => specifier,
      &Specifier::Latest(specifier) => specifier,
      &Specifier::Major(specifier) => specifier,
      &Specifier::Minor(specifier) => specifier,
      &Specifier::Range(specifier) => specifier,
      &Specifier::RangeMinor(specifier) => specifier,
      &Specifier::RangeComplex(specifier) => specifier,
      &Specifier::Alias(specifier) => specifier,
      &Specifier::File(specifier) => specifier,
      &Specifier::Git(specifier) => specifier,
      &Specifier::Tag(specifier) => specifier,
      &Specifier::Unsupported(specifier) => specifier,
      &Specifier::Url(specifier) => specifier,
      &Specifier::WorkspaceProtocol(specifier) => specifier,
      &Specifier::None => {
        panic!("Cannot unwrap a Specifier::None");
      }
    }
  }

  /// Convert non-semver specifiers to semver when behaviour is identical
  fn sanitise(specifier: &String) -> &str {
    let specifier = specifier.as_str();
    if specifier == "latest" || specifier == "x" {
      debug!("Sanitising specifier: {} → *", specifier);
      "*"
    } else {
      specifier
    }
  }

  /// Convert a raw string version specifier into a `Specifier` enum serving as a
  /// branded type
  fn parse(specifier: &String, is_recursive: bool) -> Specifier {
    let str = Specifier::sanitise(specifier);
    let string = str.to_string();
    if REGEX_EXACT.is_match(str) {
      Specifier::Exact(string)
    } else if Specifier::is_range(str) {
      Specifier::Range(string)
    } else if str == "*" || str == "latest" || str == "x" {
      Specifier::Latest(string)
    } else if REGEX_WORKSPACE_PROTOCOL.is_match(str) {
      Specifier::WorkspaceProtocol(string)
    } else if REGEX_ALIAS.is_match(str) {
      Specifier::Alias(string)
    } else if REGEX_MAJOR.is_match(str) {
      Specifier::Major(string)
    } else if REGEX_MINOR.is_match(str) {
      Specifier::Minor(string)
    } else if REGEX_TAG.is_match(str) {
      Specifier::Tag(string)
    } else if REGEX_GIT.is_match(str) {
      Specifier::Git(string)
    } else if REGEX_URL.is_match(str) {
      Specifier::Url(string)
    } else if Specifier::is_range_minor(str) {
      Specifier::RangeMinor(string)
    } else if REGEX_FILE.is_match(str) {
      Specifier::File(string)
    } else if !is_recursive && Specifier::is_complex_range(str) {
      Specifier::RangeComplex(string)
    } else {
      Specifier::Unsupported(string)
    }
  }

  /// Is this a semver range containing multiple parts?
  /// Such as OR (" || ") or AND (" ")
  fn is_complex_range(specifier: &str) -> bool {
    REGEX_OR_OPERATOR.split(specifier).map(|str| str.trim()).filter(|str| str.len() > 0).all(|or_condition| {
      or_condition
        .split(" ")
        .map(|str| str.trim())
        .filter(|str| str.len() > 0)
        .all(|and_condition| Specifier::parse(&and_condition.to_string(), true).is_semver())
    })
  }

  fn is_range(specifier: &str) -> bool {
    REGEX_CARET.is_match(specifier) || REGEX_TILDE.is_match(specifier) || REGEX_GT.is_match(specifier) || REGEX_GTE.is_match(specifier) || REGEX_LT.is_match(specifier) || REGEX_LTE.is_match(specifier)
  }

  fn is_range_minor(specifier: &str) -> bool {
    REGEX_CARET_MINOR.is_match(specifier) || REGEX_TILDE_MINOR.is_match(specifier) || REGEX_GT_MINOR.is_match(specifier) || REGEX_GTE_MINOR.is_match(specifier) || REGEX_LT_MINOR.is_match(specifier) || REGEX_LTE_MINOR.is_match(specifier)
  }

  pub fn compare_to(&self, other: &Specifier) -> Ordering {
    if self.get_exact().unwrap() == other.get_exact().unwrap() {
      let a = self.get_semver_range();
      let b = other.get_semver_range();
      return a.cmp(&b);
    } else {
      let a = self.unwrap().parse::<Version>().unwrap();
      let b = other.unwrap().parse::<Version>().unwrap();
      return a.cmp(&b);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::cmp::Ordering;

  fn to_strings(specifiers: Vec<&str>) -> Vec<String> {
    specifiers.iter().map(|s| s.to_string()).collect()
  }

  #[test]
  fn compare_specifiers() {
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
    for (a, b, expected) in cases {
      let parsed = Specifier::new(&a.to_string());
      let ordering = parsed.compare_to(&Specifier::new(&b.to_string()));
      assert_eq!(ordering, expected, "{a} should {expected:?} {b}");
    }
  }

  #[test]
  fn alias() {
    let cases: Vec<String> = to_strings(vec!["npm:@minh.nguyen/plugin-transform-destructuring@^7.5.2", "npm:@types/selenium-webdriver@4.1.18", "npm:foo@1.2.3"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Alias(case.to_string()), "{} should be alias", case);
    }
  }

  #[test]
  fn exact() {
    let cases: Vec<String> = to_strings(vec![
      "1.2.3",
      // @TODO: how to support postfix?
      // "1.2.3-alpha.1",
      // "1.2.3-alpha.1+build.123",
      // "1.2.3+build.123",
    ]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Exact(case.clone()), "{} should be exact", case);
    }
  }

  #[test]
  fn file() {
    let cases: Vec<String> = to_strings(vec![
      "file:../path/to/foo",
      "file:./path/to/foo",
      "file:/../path/to/foo",
      "file:/./path/to/foo",
      "file:/.path/to/foo",
      "file://../path/to/foo",
      "file://.",
      "file://./path/to/foo",
      "file:////path/to/foo",
      "file:///path/to/foo",
      "file://path/to/foo",
      "file:/~path/to/foo",
      "file:/path/to/foo",
      "file:path/to/directory",
      "file:path/to/foo.tar.gz",
      "file:path/to/foo.tgz",
      "file:path/to/foo",
    ]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::File(case.clone()), "{} should be file", case);
    }
  }

  #[test]
  fn git() {
    let cases: Vec<String> = to_strings(vec![
      "git://github.com/user/foo",
      "git://github.com/user/foo#1.2.3",
      "git://github.com/user/foo#semver:^1.2.3",
      "git://notgithub.com/user/foo",
      "git://notgithub.com/user/foo#1.2.3",
      "git://notgithub.com/user/foo#semver:^1.2.3",
      "git+ssh://github.com/user/foo",
      "git+ssh://github.com/user/foo#1.2.3",
      "git+ssh://github.com/user/foo#semver:^1.2.3",
      "git+ssh://notgithub.com/user/foo",
      "git+ssh://notgithub.com/user/foo#1.2.3",
      "git+ssh://notgithub.com/user/foo#semver:^1.2.3",
      "git+ssh://mydomain.com:1234/hey",
      "git://notgithub.com/user/foo",
      "git+ssh://git@github.com:user/foo#semver:^1.2.3",
      "git+ssh://git@github.com/user/foo#1.2.3",
      "git+ssh://git@github.com/user/foo#semver:^1.2.3",
      "git+ssh://git@notgithub.com:user/foo",
      "git+ssh://git@notgithub.com:user/foo#1.2.3",
      "git+ssh://git@notgithub.com:user/foo#semver:^1.2.3",
      "git+ssh://git@notgithub.com/user/foo",
      "git+ssh://git@notgithub.com/user/foo#1.2.3",
      "git+ssh://git@notgithub.com/user/foo#semver:^1.2.3",
      "git+ssh://mydomain.com:1234/hey",
      "git+ssh://mydomain.com:1234/hey#1.2.3",
      "git+ssh://mydomain.com:1234#1.2.3",
      "git+ssh://mydomain.com:foo",
      "git+ssh://mydomain.com:foo/bar#1.2.3",
      "git+ssh://mydomain.com:foo#1.2.3",
      "git+ssh://username:password@mydomain.com:1234/hey#1.2.3",
      "git+https://github.com/user/foo",
      "git+ssh://git@notgithub.com/user/foo#1.2.3",
    ]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Git(case.clone()), "{} should be git", case);
    }
  }

  #[test]
  fn latest() {
    let cases: Vec<String> = to_strings(vec!["latest", "*"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Latest("*".to_string()), "{} should be latest", case);
    }
  }

  #[test]
  fn major() {
    let cases: Vec<String> = to_strings(vec!["1"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Major(case.clone()), "{} should be major", case);
    }
  }

  #[test]
  fn minor() {
    let cases: Vec<String> = to_strings(vec!["1.2"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Minor(case.clone()), "{} should be minor", case);
    }
  }

  #[test]
  fn range() {
    let cases: Vec<String> = to_strings(vec![
      "^4.1.1", "~1.2.1", ">=5.0.0", "<=5.0.0", ">5.0.0", "<5.0.0",
      // ">=5.0.0 <6.0.0",
      // ">5.0.0 <6.0.0",
      // ">=5.0.0 <=6.0.0",
      // ">5.0.0 <=6.0.0",
      // ">=5.0.0 <6.0.0",
      // ">5.0.0 <6.0.0",
    ]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Range(case.clone()), "{} should be range", case);
    }
  }

  #[test]
  fn range_minor() {
    let cases: Vec<String> = to_strings(vec!["^4.1", "~1.2", ">=5.0", "<=5.0", ">5.0", "<5.0"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::RangeMinor(case.clone()), "{} should be range-minor", case);
    }
  }

  #[test]
  fn tag() {
    let cases: Vec<String> = to_strings(vec!["alpha", "canary", "foo"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Tag(case.clone()), "{} should be tag", case);
    }
  }

  #[test]
  fn unsupported() {
    let cases: Vec<String> = to_strings(vec![
      "@f fo o al/ a d s ;f",
      "@foo/bar",
      "@foo/bar@",
      "/path/to/foo.tar",
      "/path/to/foo.tgz",
      "/path/to/foo",
      "$typescript",
      "1.typo.wat",
      "=v1.2.3",
      "not-git@hostname.com:some/repo",
      "user/foo#1234::path:dist",
      "user/foo#notimplemented:value",
      "user/foo#path:dist",
      "user/foo#semver:^1.2.3",
      "git+file://path/to/repo#1.2.3",
    ]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Unsupported(case.clone()), "{} should be unsupported", case);
    }
  }

  #[test]
  fn url() {
    let cases: Vec<String> = to_strings(vec!["http://insecure.com/foo.tgz", "https://server.com/foo.tgz", "https://server.com/foo.tgz"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::Url(case.clone()), "{} should be url", &case);
    }
  }

  #[test]
  fn workspace_protocol() {
    let cases: Vec<String> = to_strings(vec!["workspace:*", "workspace:^", "workspace:~"]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::WorkspaceProtocol(case.clone()), "{} should be workspace-protocol", case);
    }
  }

  #[test]
  fn complex_range() {
    let cases: Vec<String> = to_strings(vec![
      "1.3.0 || <1.0.0 >2.0.0",
      "<1.0.0 >2.0.0",
      ">1.0.0 <=2.0.0",
      "<1.0.0 >=2.0.0",
      "<1.5.0 || >=1.6.0",
      "<1.6.16 || >=1.7.0 <1.7.11 || >=1.8.0 <1.8.2",
      "<=1.6.16 || >=1.7.0 <1.7.11 || >=1.8.0 <1.8.2",
      ">1.0.0 <1.0.0",
    ]);
    for case in cases {
      let parsed = Specifier::new(&case);
      assert_eq!(parsed, Specifier::RangeComplex(case.clone()), "{} should be range-complex", case);
    }
  }

  #[test]
  fn change_semver_range() {
    let cases: Vec<(&str, &str)> = vec![("^", "^1.2.3"), ("~", "~1.2.3"), (">=", ">=1.2.3"), ("<=", "<=1.2.3"), (">", ">1.2.3"), ("<", "<1.2.3"), ("", "1.2.3")];
    for (_, initial) in &cases {
      let initial = initial.to_string();
      for (range, expected) in &cases {
        let range = SemverRange::new(&range.to_string()).unwrap();
        let expected = expected.to_string();
        let parsed = Specifier::new(&initial);
        assert_eq!(parsed.with_semver_range(&range), Specifier::new(&expected.clone()), "{} + {:?} should produce {}", initial, range, expected);
      }
    }
  }

  #[test]
  fn has_semver_range() {
    let cases: Vec<(&str, &str, bool)> = vec![
      ("^", "^1.2.3", true),
      ("^", "~1.2.3", false),
      ("^", ">=1.2.3", false),
      ("^", "<=1.2.3", false),
      ("^", ">1.2.3", false),
      ("^", "<1.2.3", false),
      ("^", "1.2.3", false),
      ("~", "^1.2.3", false),
      ("~", "~1.2.3", true),
      ("~", ">=1.2.3", false),
      ("~", "<=1.2.3", false),
      ("~", ">1.2.3", false),
      ("~", "<1.2.3", false),
      ("~", "1.2.3", false),
      (">=", "^1.2.3", false),
      (">=", "~1.2.3", false),
      (">=", ">=1.2.3", true),
      (">=", "<=1.2.3", false),
      (">=", ">1.2.3", false),
      (">=", "<1.2.3", false),
      (">=", "1.2.3", false),
      ("<=", "^1.2.3", false),
      ("<=", "~1.2.3", false),
      ("<=", ">=1.2.3", false),
      ("<=", "<=1.2.3", true),
      ("<=", ">1.2.3", false),
      ("<=", "<1.2.3", false),
      ("<=", "1.2.3", false),
      (">", "^1.2.3", false),
      (">", "~1.2.3", false),
      (">", ">=1.2.3", false),
      (">", "<=1.2.3", false),
      (">", ">1.2.3", true),
      (">", "<1.2.3", false),
      (">", "1.2.3", false),
      ("<", "^1.2.3", false),
      ("<", "~1.2.3", false),
      ("<", ">=1.2.3", false),
      ("<", "<=1.2.3", false),
      ("<", ">1.2.3", false),
      ("<", "<1.2.3", true),
      ("<", "1.2.3", false),
      ("", "^1.2.3", false),
      ("", "~1.2.3", false),
      ("", ">=1.2.3", false),
      ("", "<=1.2.3", false),
      ("", ">1.2.3", false),
      ("", "<1.2.3", false),
      ("", "1.2.3", true),
    ];
    for (range, specifier, expected) in cases {
      let range = SemverRange::new(&range.to_string()).unwrap();
      let parsed = Specifier::new(&specifier.to_string());
      assert_eq!(parsed.has_range(&range), expected, "{} has range {:?} should be {}", specifier, range, expected);
    }
  }
}
