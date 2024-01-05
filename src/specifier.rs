use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  /// "1.2.3"
  static ref EXACT: Regex = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
  /// "^1.2.3"
  static ref CARET: Regex = Regex::new(r"^\^(\d+\.\d+\.\d+)$").unwrap();
  /// "~1.2.3"
  static ref TILDE: Regex = Regex::new(r"^~(\d+\.\d+\.\d+)$").unwrap();
  /// ">1.2.3"
  static ref GT: Regex = Regex::new(r"^>(\d+\.\d+\.\d+)$").unwrap();
  /// ">=1.2.3"
  static ref GTE: Regex = Regex::new(r"^>=(\d+\.\d+\.\d+)$").unwrap();
  /// "<1.2.3"
  static ref LT: Regex = Regex::new(r"^<(\d+\.\d+\.\d+)$").unwrap();
  /// "<=1.2.3"
  static ref LTE: Regex = Regex::new(r"^<=(\d+\.\d+\.\d+)$").unwrap();
  /// "^1.2"
  static ref CARET_MINOR: Regex = Regex::new(r"^\^(\d+\.\d+)$").unwrap();
  /// "~1.2"
  static ref TILDE_MINOR: Regex = Regex::new(r"^~(\d+\.\d+)$").unwrap();
  /// ">1.2"
  static ref GT_MINOR: Regex = Regex::new(r"^>(\d+\.\d+)$").unwrap();
  /// ">=1.2"
  static ref GTE_MINOR: Regex = Regex::new(r"^>=(\d+\.\d+)$").unwrap();
  /// "<1.2"
  static ref LT_MINOR: Regex = Regex::new(r"^<(\d+\.\d+)$").unwrap();
  /// "<=1.2"
  static ref LTE_MINOR: Regex = Regex::new(r"^<=(\d+\.\d+)$").unwrap();
  /// "1"
  static ref MAJOR: Regex = Regex::new(r"^(\d+)$").unwrap();
  /// "1.2"
  static ref MINOR: Regex = Regex::new(r"^(\d+\.\d+)$").unwrap();
  /// "npm:"
  static ref ALIAS: Regex = Regex::new(r"^npm:").unwrap();
  /// "file:"
  static ref FILE: Regex = Regex::new(r"^file:").unwrap();
  /// "workspace:"
  static ref WORKSPACE_PROTOCOL: Regex = Regex::new(r"^workspace:").unwrap();
  /// "https://"
  static ref URL: Regex = Regex::new(r"^https?://").unwrap();
  /// "git://"
  static ref GIT: Regex = Regex::new(r"^git(\+(ssh|https?))?://").unwrap();
  /// "alpha"
  static ref TAG: Regex = Regex::new(r"^[a-zA-Z0-9-]+$").unwrap();
}

pub const IS_EXACT: SpecifierType = SpecifierType::Semver(Semver::Exact);
pub const IS_LATEST: SpecifierType = SpecifierType::Semver(Semver::Latest);
pub const IS_MAJOR: SpecifierType = SpecifierType::Semver(Semver::Major);
pub const IS_MINOR: SpecifierType = SpecifierType::Semver(Semver::Minor);
pub const IS_RANGE: SpecifierType = SpecifierType::Semver(Semver::Range);
pub const IS_RANGE_MINOR: SpecifierType = SpecifierType::Semver(Semver::RangeMinor);
pub const IS_ALIAS: SpecifierType = SpecifierType::NonSemver(NonSemver::Alias);
pub const IS_FILE: SpecifierType = SpecifierType::NonSemver(NonSemver::File);
pub const IS_GIT: SpecifierType = SpecifierType::NonSemver(NonSemver::Git);
pub const IS_TAG: SpecifierType = SpecifierType::NonSemver(NonSemver::Tag);
pub const IS_UNSUPPORTED: SpecifierType = SpecifierType::NonSemver(NonSemver::Unsupported);
pub const IS_URL: SpecifierType = SpecifierType::NonSemver(NonSemver::Url);
pub const IS_WORKSPACE_PROTOCOL: SpecifierType =
  SpecifierType::NonSemver(NonSemver::WorkspaceProtocol);

#[derive(Debug, PartialEq)]
pub enum Semver {
  Exact,
  Latest,
  Major,
  Minor,
  Range,
  RangeMinor,
}

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
pub enum SpecifierType {
  Semver(Semver),
  NonSemver(NonSemver),
}

// @TODO: add nested enums of semver or not

impl SpecifierType {
  pub fn new(specifier: &str) -> SpecifierType {
    if EXACT.is_match(specifier) {
      IS_EXACT
    } else if is_range(specifier) {
      IS_RANGE
    } else if specifier == "*" || specifier == "latest" {
      IS_LATEST
    } else if WORKSPACE_PROTOCOL.is_match(specifier) {
      IS_WORKSPACE_PROTOCOL
    } else if ALIAS.is_match(specifier) {
      IS_ALIAS
    } else if MAJOR.is_match(specifier) {
      IS_MAJOR
    } else if MINOR.is_match(specifier) {
      IS_MINOR
    } else if TAG.is_match(specifier) {
      IS_TAG
    } else if GIT.is_match(specifier) {
      IS_GIT
    } else if URL.is_match(specifier) {
      IS_URL
    } else if is_range_minor(specifier) {
      IS_RANGE_MINOR
    } else if FILE.is_match(specifier) {
      IS_FILE
    } else {
      IS_UNSUPPORTED
    }
  }
}

fn is_range(specifier: &str) -> bool {
  CARET.is_match(specifier)
    || TILDE.is_match(specifier)
    || GT.is_match(specifier)
    || GTE.is_match(specifier)
    || LT.is_match(specifier)
    || LTE.is_match(specifier)
}

fn is_range_minor(specifier: &str) -> bool {
  CARET_MINOR.is_match(specifier)
    || TILDE_MINOR.is_match(specifier)
    || GT_MINOR.is_match(specifier)
    || GTE_MINOR.is_match(specifier)
    || LT_MINOR.is_match(specifier)
    || LTE_MINOR.is_match(specifier)
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
      let parsed = SpecifierType::new(case);
      assert_eq!(
        parsed,
        SpecifierType::NonSemver(NonSemver::Alias),
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
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_EXACT, "{} should be exact", case);
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
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_FILE, "{} should be file", case);
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
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_GIT, "{} should be git", case);
    }
  }

  #[test]
  fn latest() {
    let cases: Vec<&str> = vec!["latest", "*"];
    for case in cases {
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_LATEST, "{} should be latest", case);
    }
  }

  #[test]
  fn major() {
    let cases: Vec<&str> = vec!["1"];
    for case in cases {
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_MAJOR, "{} should be major", case);
    }
  }

  #[test]
  fn minor() {
    let cases: Vec<&str> = vec!["1.2"];
    for case in cases {
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_MINOR, "{} should be minor", case);
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
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_RANGE, "{} should be range", case);
    }
  }

  #[test]
  fn range_minor() {
    let cases: Vec<&str> = vec!["^4.1", "~1.2", ">=5.0", "<=5.0", ">5.0", "<5.0"];
    for case in cases {
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_RANGE_MINOR, "{} should be range-minor", case);
    }
  }

  #[test]
  fn tag() {
    let cases: Vec<&str> = vec!["alpha", "canary", "foo"];
    for case in cases {
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_TAG, "{} should be tag", case);
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
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_UNSUPPORTED, "{} should be unsupported", case);
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
      let parsed = SpecifierType::new(case);
      assert_eq!(parsed, IS_URL, "{} should be url", case);
    }
  }

  #[test]
  fn workspace_protocol() {
    let cases: Vec<&str> = vec!["workspace:*", "workspace:^", "workspace:~"];
    for case in cases {
      let parsed = SpecifierType::new(case);
      assert_eq!(
        parsed, IS_WORKSPACE_PROTOCOL,
        "{} should be workspace-protocol",
        case
      );
    }
  }
}
