use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
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
}

pub const EXACT: Specifier = Specifier::Semver(Semver::Exact);
pub const LATEST: Specifier = Specifier::Semver(Semver::Latest);
pub const MAJOR: Specifier = Specifier::Semver(Semver::Major);
pub const MINOR: Specifier = Specifier::Semver(Semver::Minor);
pub const RANGE: Specifier = Specifier::Semver(Semver::Range);
pub const RANGE_MINOR: Specifier = Specifier::Semver(Semver::RangeMinor);
pub const ALIAS: Specifier = Specifier::NonSemver(NonSemver::Alias);
pub const FILE: Specifier = Specifier::NonSemver(NonSemver::File);
pub const GIT: Specifier = Specifier::NonSemver(NonSemver::Git);
pub const TAG: Specifier = Specifier::NonSemver(NonSemver::Tag);
pub const UNSUPPORTED: Specifier = Specifier::NonSemver(NonSemver::Unsupported);
pub const URL: Specifier = Specifier::NonSemver(NonSemver::Url);
pub const WORKSPACE_PROTOCOL: Specifier = Specifier::NonSemver(NonSemver::WorkspaceProtocol);

#[derive(Debug, PartialEq)]
pub enum Semver {
  Exact,
  Latest,
  Major,
  Minor,
  Range,
  RangeMinor,
}

// @TODO: add nested enums of supported or not supported
#[derive(Debug, PartialEq)]
pub enum NonSemver {
  // @TODO: can be considered semver once parsing is improved
  Alias,
  File,
  Git,
  Tag,
  Unsupported,
  Url,
  WorkspaceProtocol,
}

#[derive(Debug, PartialEq)]
pub enum Specifier {
  Semver(Semver),
  NonSemver(NonSemver),
}

impl Specifier {
  pub fn new(specifier: &str) -> Specifier {
    if REGEX_EXACT.is_match(specifier) {
      EXACT
    } else if is_range(specifier) {
      RANGE
    } else if specifier == "*" || specifier == "latest" {
      LATEST
    } else if REGEX_WORKSPACE_PROTOCOL.is_match(specifier) {
      WORKSPACE_PROTOCOL
    } else if REGEX_ALIAS.is_match(specifier) {
      ALIAS
    } else if REGEX_MAJOR.is_match(specifier) {
      MAJOR
    } else if REGEX_MINOR.is_match(specifier) {
      MINOR
    } else if REGEX_TAG.is_match(specifier) {
      TAG
    } else if REGEX_GIT.is_match(specifier) {
      GIT
    } else if REGEX_URL.is_match(specifier) {
      URL
    } else if is_range_minor(specifier) {
      RANGE_MINOR
    } else if REGEX_FILE.is_match(specifier) {
      FILE
    } else {
      UNSUPPORTED
    }
  }
}

fn is_range(specifier: &str) -> bool {
  REGEX_CARET.is_match(specifier)
    || REGEX_TILDE.is_match(specifier)
    || REGEX_GT.is_match(specifier)
    || REGEX_GTE.is_match(specifier)
    || REGEX_LT.is_match(specifier)
    || REGEX_LTE.is_match(specifier)
}

fn is_range_minor(specifier: &str) -> bool {
  REGEX_CARET_MINOR.is_match(specifier)
    || REGEX_TILDE_MINOR.is_match(specifier)
    || REGEX_GT_MINOR.is_match(specifier)
    || REGEX_GTE_MINOR.is_match(specifier)
    || REGEX_LT_MINOR.is_match(specifier)
    || REGEX_LTE_MINOR.is_match(specifier)
}

pub fn get_specifier_type_name(specifier_type: &Specifier) -> String {
  match specifier_type {
    &EXACT => "exact".to_string(),
    &LATEST => "latest".to_string(),
    &MAJOR => "major".to_string(),
    &MINOR => "minor".to_string(),
    &RANGE => "range".to_string(),
    &RANGE_MINOR => "range-minor".to_string(),
    &ALIAS => "alias".to_string(),
    &FILE => "file".to_string(),
    &GIT => "git".to_string(),
    &TAG => "tag".to_string(),
    &UNSUPPORTED => "unsupported".to_string(),
    &URL => "url".to_string(),
    &WORKSPACE_PROTOCOL => "workspace-protocol".to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn alias() {
    let cases: Vec<&str> = vec![
      "npm:@minh.nguyen/plugin-transform-destructuring@^7.5.2",
      "npm:@types/selenium-webdriver@4.1.18",
      "npm:foo@1.2.3",
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(
        parsed,
        Specifier::NonSemver(NonSemver::Alias),
        "{} should be alias",
        case
      );
    }
  }

  #[test]
  fn exact() {
    let cases: Vec<&str> = vec![
      "1.2.3",
      // @TODO: how to support postfix?
      // "1.2.3-alpha.1",
      // "1.2.3-alpha.1+build.123",
      // "1.2.3+build.123",
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, EXACT, "{} should be exact", case);
    }
  }

  #[test]
  fn file() {
    let cases: Vec<&str> = vec![
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
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, FILE, "{} should be file", case);
    }
  }

  #[test]
  fn git() {
    let cases: Vec<&str> = vec![
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
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, GIT, "{} should be git", case);
    }
  }

  #[test]
  fn latest() {
    let cases: Vec<&str> = vec!["latest", "*"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, LATEST, "{} should be latest", case);
    }
  }

  #[test]
  fn major() {
    let cases: Vec<&str> = vec!["1"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, MAJOR, "{} should be major", case);
    }
  }

  #[test]
  fn minor() {
    let cases: Vec<&str> = vec!["1.2"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, MINOR, "{} should be minor", case);
    }
  }

  #[test]
  fn range() {
    let cases: Vec<&str> = vec![
      "^4.1.1", "~1.2.1", ">=5.0.0", "<=5.0.0", ">5.0.0",
      "<5.0.0",
      // ">=5.0.0 <6.0.0",
      // ">5.0.0 <6.0.0",
      // ">=5.0.0 <=6.0.0",
      // ">5.0.0 <=6.0.0",
      // ">=5.0.0 <6.0.0",
      // ">5.0.0 <6.0.0",
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, RANGE, "{} should be range", case);
    }
  }

  #[test]
  fn range_minor() {
    let cases: Vec<&str> = vec!["^4.1", "~1.2", ">=5.0", "<=5.0", ">5.0", "<5.0"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, RANGE_MINOR, "{} should be range-minor", case);
    }
  }

  #[test]
  fn tag() {
    let cases: Vec<&str> = vec!["alpha", "canary", "foo"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, TAG, "{} should be tag", case);
    }
  }

  #[test]
  fn unsupported() {
    let cases: Vec<&str> = vec![
      "@f fo o al/ a d s ;f",
      "@foo/bar",
      "@foo/bar@",
      "/path/to/foo.tar",
      "/path/to/foo.tgz",
      "/path/to/foo",
      "$typescript",
      "1.typo.wat",
      " 1.2 ",
      " 1.2.3 ",
      "=v1.2.3",
      "not-git@hostname.com:some/repo",
      "user/foo#1234::path:dist",
      "user/foo#notimplemented:value",
      "user/foo#path:dist",
      "user/foo#semver:^1.2.3",
      "git+file://path/to/repo#1.2.3",
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, UNSUPPORTED, "{} should be unsupported", case);
    }
  }

  #[test]
  fn url() {
    let cases: Vec<&str> = vec![
      "http://insecure.com/foo.tgz",
      "https://server.com/foo.tgz",
      "https://server.com/foo.tgz",
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, URL, "{} should be url", case);
    }
  }

  #[test]
  fn workspace_protocol() {
    let cases: Vec<&str> = vec!["workspace:*", "workspace:^", "workspace:~"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(
        parsed, WORKSPACE_PROTOCOL,
        "{} should be workspace-protocol",
        case
      );
    }
  }
}
