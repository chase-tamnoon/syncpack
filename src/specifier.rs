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
  /// a logical OR in a semver range
  static ref REGEX_OR_OPERATOR:Regex = Regex::new(r" ?\|\| ?").unwrap();
}

#[derive(Debug, PartialEq)]
pub enum Specifier {
  // Semver
  Exact,
  Latest,
  Major,
  Minor,
  Range,
  RangeComplex,
  RangeMinor,
  // Non Semver
  Alias,
  File,
  Git,
  Tag,
  Unsupported,
  Url,
  WorkspaceProtocol,
}

impl Specifier {
  pub fn is_semver(&self) -> bool {
    match self {
      Self::Exact => true,
      Self::Latest => true,
      Self::Major => true,
      Self::Minor => true,
      Self::Range => true,
      Self::RangeComplex => true,
      Self::RangeMinor => true,
      Self::Alias => false,
      Self::File => false,
      Self::Git => false,
      Self::Tag => false,
      Self::Unsupported => false,
      Self::Url => false,
      Self::WorkspaceProtocol => false,
    }
  }
}

impl Specifier {
  pub fn new(specifier: &str) -> Self {
    parse_specifier(specifier, false)
  }
}

pub fn parse_specifier(specifier: &str, is_recursive: bool) -> Specifier {
  if REGEX_EXACT.is_match(specifier) {
    Specifier::Exact
  } else if is_range(specifier) {
    Specifier::Range
  } else if specifier == "*" || specifier == "latest" || specifier == "x" {
    Specifier::Latest
  } else if REGEX_WORKSPACE_PROTOCOL.is_match(specifier) {
    Specifier::WorkspaceProtocol
  } else if REGEX_ALIAS.is_match(specifier) {
    Specifier::Alias
  } else if REGEX_MAJOR.is_match(specifier) {
    Specifier::Major
  } else if REGEX_MINOR.is_match(specifier) {
    Specifier::Minor
  } else if REGEX_TAG.is_match(specifier) {
    Specifier::Tag
  } else if REGEX_GIT.is_match(specifier) {
    Specifier::Git
  } else if REGEX_URL.is_match(specifier) {
    Specifier::Url
  } else if is_range_minor(specifier) {
    Specifier::RangeMinor
  } else if REGEX_FILE.is_match(specifier) {
    Specifier::File
  } else if !is_recursive && is_complex_range(specifier) {
    Specifier::RangeComplex
  } else {
    Specifier::Unsupported
  }
}

/// Is this a semver range containing multiple parts?
/// Such as OR (" || ") or AND (" ")
fn is_complex_range(specifier: &str) -> bool {
  REGEX_OR_OPERATOR
    .split(specifier)
    .map(|str| str.trim())
    .filter(|str| str.len() > 0)
    .all(|or_condition| {
      or_condition
        .split(" ")
        .map(|str| str.trim())
        .filter(|str| str.len() > 0)
        .all(|and_condition| parse_specifier(and_condition, true).is_semver())
    })
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
    &Specifier::Exact => "exact".to_string(),
    &Specifier::Latest => "latest".to_string(),
    &Specifier::Major => "major".to_string(),
    &Specifier::Minor => "minor".to_string(),
    &Specifier::Range => "range".to_string(),
    &Specifier::RangeMinor => "range-minor".to_string(),
    &Specifier::RangeComplex => "range-complex".to_string(),
    &Specifier::Alias => "alias".to_string(),
    &Specifier::File => "file".to_string(),
    &Specifier::Git => "git".to_string(),
    &Specifier::Tag => "tag".to_string(),
    &Specifier::Unsupported => "unsupported".to_string(),
    &Specifier::Url => "url".to_string(),
    &Specifier::WorkspaceProtocol => "workspace-protocol".to_string(),
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
      assert_eq!(parsed, Specifier::Alias, "{} should be alias", case);
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
      assert_eq!(parsed, Specifier::Exact, "{} should be exact", case);
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
      assert_eq!(parsed, Specifier::File, "{} should be file", case);
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
      assert_eq!(parsed, Specifier::Git, "{} should be git", case);
    }
  }

  #[test]
  fn latest() {
    let cases: Vec<&str> = vec!["latest", "*"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, Specifier::Latest, "{} should be latest", case);
    }
  }

  #[test]
  fn major() {
    let cases: Vec<&str> = vec!["1"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, Specifier::Major, "{} should be major", case);
    }
  }

  #[test]
  fn minor() {
    let cases: Vec<&str> = vec!["1.2"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, Specifier::Minor, "{} should be minor", case);
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
      assert_eq!(parsed, Specifier::Range, "{} should be range", case);
    }
  }

  #[test]
  fn range_minor() {
    let cases: Vec<&str> = vec!["^4.1", "~1.2", ">=5.0", "<=5.0", ">5.0", "<5.0"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(
        parsed,
        Specifier::RangeMinor,
        "{} should be range-minor",
        case
      );
    }
  }

  #[test]
  fn tag() {
    let cases: Vec<&str> = vec!["alpha", "canary", "foo"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed, Specifier::Tag, "{} should be tag", case);
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
      assert_eq!(
        parsed,
        Specifier::Unsupported,
        "{} should be unsupported",
        case
      );
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
      assert_eq!(parsed, Specifier::Url, "{} should be url", case);
    }
  }

  #[test]
  fn workspace_protocol() {
    let cases: Vec<&str> = vec!["workspace:*", "workspace:^", "workspace:~"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(
        parsed,
        Specifier::WorkspaceProtocol,
        "{} should be workspace-protocol",
        case
      );
    }
  }

  #[test]
  fn complex_range() {
    let cases: Vec<&str> = vec![
      "1.3.0 || <1.0.0 >2.0.0",
      "<1.0.0 >2.0.0",
      ">1.0.0 <=2.0.0",
      "<1.0.0 >=2.0.0",
      "<1.5.0 || >=1.6.0",
      "<1.6.16 || >=1.7.0 <1.7.11 || >=1.8.0 <1.8.2",
      "<=1.6.16 || >=1.7.0 <1.7.11 || >=1.8.0 <1.8.2",
      ">1.0.0 <1.0.0",
    ];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(
        parsed,
        Specifier::RangeComplex,
        "{} should be range-complex",
        case
      );
    }
  }
}
