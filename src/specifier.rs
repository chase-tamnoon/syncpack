use semver::Semver;

use crate::specifier::{semver_range::SemverRange, specifier_tree::SpecifierTree};

pub mod non_semver;
pub mod parser;
pub mod regexes;
pub mod semver;
pub mod semver_range;
pub mod simple_semver;
pub mod specifier_tree;

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
    parser::parse(specifier, false)
  }

  // @TODO: impl Eq
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

  #[deprecated]
  pub fn get_semver_range(&self) -> Option<SemverRange> {
    let specifier = self.unwrap();
    if specifier == "*" {
      return Some(SemverRange::Any);
    }
    if regexes::EXACT.is_match(specifier) || regexes::EXACT_TAG.is_match(specifier) {
      return Some(SemverRange::Exact);
    }
    if regexes::CARET.is_match(specifier) || regexes::CARET_TAG.is_match(specifier) {
      return Some(SemverRange::Minor);
    }
    if regexes::TILDE.is_match(specifier) || regexes::TILDE_TAG.is_match(specifier) {
      return Some(SemverRange::Patch);
    }
    if regexes::GT.is_match(specifier) || regexes::GT_TAG.is_match(specifier) {
      return Some(SemverRange::Gt);
    }
    if regexes::GTE.is_match(specifier) || regexes::GTE_TAG.is_match(specifier) {
      return Some(SemverRange::Gte);
    }
    if regexes::LT.is_match(specifier) || regexes::LT_TAG.is_match(specifier) {
      return Some(SemverRange::Lt);
    }
    if regexes::LTE.is_match(specifier) || regexes::LTE_TAG.is_match(specifier) {
      return Some(SemverRange::Lte);
    }
    return None;
  }

  /// Get the specifier with the given range applied if it is valid to do so,
  /// otherwise return the specifier unchanged
  pub fn with_range_if_semver(&self, range: &SemverRange) -> Self {
    let tree = SpecifierTree::new(&self);
    if let SpecifierTree::Semver(Semver::Simple(simple_semver_specifier)) = tree {
      simple_semver_specifier.with_range(&range).to_specifier()
    } else {
      self.clone()
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
}

#[cfg(test)]
mod tests {
  use super::*;

  fn to_strings(specifiers: Vec<&str>) -> Vec<String> {
    specifiers.iter().map(|s| s.to_string()).collect()
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
        assert_eq!(parsed.with_range_if_semver(&range), Specifier::new(&expected.clone()), "{} + {:?} should produce {}", initial, range, expected);
      }
    }
  }
}
