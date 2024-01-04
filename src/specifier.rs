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

#[derive(Debug)]
struct Specifier {
  pub type_name: String,
}

impl Specifier {
  pub fn new(specifier: &str) -> Specifier {
    let type_name = if EXACT.is_match(specifier) {
      "exact"
    } else if is_range(specifier) {
      "range"
    } else if specifier == "*" || specifier == "latest" {
      "latest"
    } else if WORKSPACE_PROTOCOL.is_match(specifier) {
      "workspace-protocol"
    } else if ALIAS.is_match(specifier) {
      "alias"
    } else if MAJOR.is_match(specifier) {
      "major"
    } else if MINOR.is_match(specifier) {
      "minor"
    } else if TAG.is_match(specifier) {
      "tag"
    } else if GIT.is_match(specifier) {
      "git"
    } else if URL.is_match(specifier) {
      "url"
    } else if is_range_minor(specifier) {
      "range-minor"
    } else if FILE.is_match(specifier) {
      "file"
    } else {
      "unsupported"
    };

    Specifier {
      type_name: type_name.to_string(),
    }
  }
}

fn is_exact(specifier: &str) -> bool {
  EXACT.is_match(specifier)
}

fn is_caret(specifier: &str) -> bool {
  CARET.is_match(specifier)
}

fn is_tilde(specifier: &str) -> bool {
  TILDE.is_match(specifier)
}

fn is_gt(specifier: &str) -> bool {
  GT.is_match(specifier)
}

fn is_gte(specifier: &str) -> bool {
  GTE.is_match(specifier)
}

fn is_lt(specifier: &str) -> bool {
  LT.is_match(specifier)
}

fn is_lte(specifier: &str) -> bool {
  LTE.is_match(specifier)
}

fn is_range(specifier: &str) -> bool {
  is_caret(specifier)
    || is_tilde(specifier)
    || is_gt(specifier)
    || is_gte(specifier)
    || is_lt(specifier)
    || is_lte(specifier)
}

fn is_caret_minor(specifier: &str) -> bool {
  CARET_MINOR.is_match(specifier)
}

fn is_tilde_minor(specifier: &str) -> bool {
  TILDE_MINOR.is_match(specifier)
}

fn is_gt_minor(specifier: &str) -> bool {
  GT_MINOR.is_match(specifier)
}

fn is_gte_minor(specifier: &str) -> bool {
  GTE_MINOR.is_match(specifier)
}

fn is_lt_minor(specifier: &str) -> bool {
  LT_MINOR.is_match(specifier)
}

fn is_lte_minor(specifier: &str) -> bool {
  LTE_MINOR.is_match(specifier)
}

fn is_range_minor(specifier: &str) -> bool {
  is_caret_minor(specifier)
    || is_tilde_minor(specifier)
    || is_gt_minor(specifier)
    || is_gte_minor(specifier)
    || is_lt_minor(specifier)
    || is_lte_minor(specifier)
}

fn is_latest(specifier: &str) -> bool {
  specifier == "*" || specifier == "latest"
}

fn is_major(specifier: &str) -> bool {
  MAJOR.is_match(specifier)
}

fn is_minor(specifier: &str) -> bool {
  MINOR.is_match(specifier)
}

fn is_alias(specifier: &str) -> bool {
  ALIAS.is_match(specifier)
}

fn is_file(specifier: &str) -> bool {
  FILE.is_match(specifier)
}

fn is_workspace_protocol(specifier: &str) -> bool {
  WORKSPACE_PROTOCOL.is_match(specifier)
}

fn is_url(specifier: &str) -> bool {
  URL.is_match(specifier)
}

fn is_git(specifier: &str) -> bool {
  GIT.is_match(specifier)
}

fn is_tag(specifier: &str) -> bool {
  TAG.is_match(specifier)
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
      assert_eq!(parsed.type_name, "alias", "{} should be alias", case);
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
      assert_eq!(parsed.type_name, "exact", "{} should be exact", case);
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
      assert_eq!(parsed.type_name, "file", "{} should be file", case);
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
      assert_eq!(parsed.type_name, "git", "{} should be git", case);
    }
  }

  #[test]
  fn latest() {
    let cases: Vec<&str> = vec!["latest", "*"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed.type_name, "latest", "{} should be latest", case);
    }
  }

  #[test]
  fn major() {
    let cases: Vec<&str> = vec!["1"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed.type_name, "major", "{} should be major", case);
    }
  }

  #[test]
  fn minor() {
    let cases: Vec<&str> = vec!["1.2"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(parsed.type_name, "minor", "{} should be minor", case);
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
      assert_eq!(parsed.type_name, "range", "{} should be range", case);
    }
  }

  #[test]
  fn range_minor() {
    let cases: Vec<&str> = vec!["^4.1", "~1.2", ">=5.0", "<=5.0", ">5.0", "<5.0"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(
        parsed.type_name, "range-minor",
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
      assert_eq!(parsed.type_name, "tag", "{} should be tag", case);
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
      assert_eq!(
        parsed.type_name, "unsupported",
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
      assert_eq!(parsed.type_name, "url", "{} should be url", case);
    }
  }

  #[test]
  fn workspace_protocol() {
    let cases: Vec<&str> = vec!["workspace:*", "workspace:^", "workspace:~"];
    for case in cases {
      let parsed = Specifier::new(case);
      assert_eq!(
        parsed.type_name, "workspace-protocol",
        "{} should be workspace-protocol",
        case
      );
    }
  }
}
