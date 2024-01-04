#[derive(Debug)]
struct Specifier {
  pub type_name: String,
}

fn is_exact(specifier: &str) -> bool {
  regex::Regex::new(r"^\d+\.\d+\.\d+$")
    .unwrap()
    .is_match(specifier)
}

fn is_caret(specifier: &str) -> bool {
  regex::Regex::new(r"^\^(\d+\.\d+\.\d+)$")
    .unwrap()
    .is_match(specifier)
}

fn is_tilde(specifier: &str) -> bool {
  regex::Regex::new(r"^~(\d+\.\d+\.\d+)$")
    .unwrap()
    .is_match(specifier)
}

fn is_latest(specifier: &str) -> bool {
  specifier == "*" || specifier == "latest"
}

fn is_major(specifier: &str) -> bool {
  regex::Regex::new(r"^(\d+)$").unwrap().is_match(specifier)
}

fn is_minor(specifier: &str) -> bool {
  regex::Regex::new(r"^(\d+\.\d+)$")
    .unwrap()
    .is_match(specifier)
}

fn is_alias(specifier: &str) -> bool {
  regex::Regex::new(r"^npm:").unwrap().is_match(specifier)
}

fn is_file(specifier: &str) -> bool {
  regex::Regex::new(r"^file:").unwrap().is_match(specifier)
}

fn is_workspace_protocol(specifier: &str) -> bool {
  regex::Regex::new(r"^workspace:")
    .unwrap()
    .is_match(specifier)
}

fn is_url(specifier: &str) -> bool {
  regex::Regex::new(r"^https?://")
    .unwrap()
    .is_match(specifier)
}

fn is_git(specifier: &str) -> bool {
  regex::Regex::new(r"^git(\+(ssh|https?))?://")
    .unwrap()
    .is_match(specifier)
}

fn resolve(name: &str, specifier: &str) -> Specifier {
  if is_exact(specifier) {
    Specifier {
      type_name: "exact".to_string(),
    }
  } else if is_caret(specifier) {
    Specifier {
      type_name: "range".to_string(),
    }
  } else if is_tilde(specifier) {
    Specifier {
      type_name: "range".to_string(),
    }
  } else if is_major(specifier) {
    Specifier {
      type_name: "major".to_string(),
    }
  } else if is_minor(specifier) {
    Specifier {
      type_name: "minor".to_string(),
    }
  } else if is_latest(specifier) {
    Specifier {
      type_name: "latest".to_string(),
    }
  } else if is_file(specifier) {
    Specifier {
      type_name: "file".to_string(),
    }
  } else if is_workspace_protocol(specifier) {
    Specifier {
      type_name: "workspace-protocol".to_string(),
    }
  } else if is_alias(specifier) {
    Specifier {
      type_name: "alias".to_string(),
    }
  } else if is_url(specifier) {
    Specifier {
      type_name: "url".to_string(),
    }
  } else if is_git(specifier) {
    Specifier {
      type_name: "git".to_string(),
    }
  } else {
    Specifier {
      type_name: "unsupported".to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug)]
  struct Scenario<'a> {
    name: &'a str,
    specifier: &'a str,
    expected_type_name: &'a str,
  }

  #[test]
  fn alias() {
    let cases: Vec<Scenario> = vec![
      Scenario {
        name: "foo",
        specifier: "npm:@minh.nguyen/plugin-transform-destructuring@^7.5.2",
        expected_type_name: "alias",
      },
      Scenario {
        name: "foo",
        specifier: "npm:@types/selenium-webdriver@4.1.18",
        expected_type_name: "alias",
      },
      Scenario {
        name: "foo",
        specifier: "npm:foo@1.2.3",
        expected_type_name: "alias",
      },
    ];
    for case in cases {
      let parsed = resolve(case.name, case.specifier);
      assert_eq!(parsed.type_name, case.expected_type_name);
    }
  }

  #[test]
  fn exact() {
    let cases: Vec<&str> = vec![
      "1.2.3",
      "1.2.3-alpha.1",
      "1.2.3-alpha.1+build.123",
      "1.2.3+build.123",
    ];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "exact");
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
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "file");
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
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "git");
    }
  }

  #[test]
  fn latest() {
    let cases: Vec<&str> = vec!["latest", "*"];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "latest");
    }
  }

  #[test]
  fn major() {
    let cases: Vec<&str> = vec!["1"];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "major");
    }
  }

  #[test]
  fn minor() {
    let cases: Vec<&str> = vec!["1.2"];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "minor");
    }
  }

  #[test]
  fn range() {
    let cases: Vec<&str> = vec!["^4.1.1", ">=5.0.0", "~1.2.1"];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "range");
    }
  }

  #[test]
  fn range_minor() {
    let cases: Vec<&str> = vec!["~1.2", "^1.2", "~1.2"];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "range-minor");
    }
  }

  #[test]
  fn tag() {
    let cases: Vec<&str> = vec!["alpha", "canary", "foo"];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "tag");
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
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "unsupported");
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
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "url");
    }
  }

  #[test]
  fn workspace_protocol() {
    let cases: Vec<&str> = vec!["workspace:*", "workspace:^", "workspace:~"];
    for case in cases {
      let parsed = resolve("foo", case);
      assert_eq!(parsed.type_name, "workspace-protocol");
    }
  }
}
