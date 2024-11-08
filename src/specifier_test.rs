use {super::*, std::cmp::Ordering};

#[test]
fn parses_node_specifier_strings() {
  let cases: Vec<(&str, &str, bool, Option<SimpleSemver>)> = vec![
    ("*", "latest", true, Some(SimpleSemver::Latest("*".to_string()))),
    ("1", "major", true, Some(SimpleSemver::Major("1".to_string()))),
    ("1.2", "minor", true, Some(SimpleSemver::Minor("1.2".to_string()))),
    // exact semver versions
    ("0.0.0", "exact", true, Some(SimpleSemver::Exact("0.0.0".to_string()))),
    ("1.2.3-alpha", "exact", true, Some(SimpleSemver::Exact("1.2.3-alpha".to_string()))),
    ("1.2.3-rc.1", "exact", true, Some(SimpleSemver::Exact("1.2.3-rc.1".to_string()))),
    ("1.2.3-alpha", "exact", true, Some(SimpleSemver::Exact("1.2.3-alpha".to_string()))),
    ("1.2.3-rc.0", "exact", true, Some(SimpleSemver::Exact("1.2.3-rc.0".to_string()))),
    // complex semver queries
    ("1.3.0 || <1.0.0 >2.0.0", "range-complex", false, None),
    ("<1.0.0 >2.0.0", "range-complex", false, None),
    ("<1.0.0 >=2.0.0", "range-complex", false, None),
    ("<1.5.0 || >=1.6.0", "range-complex", false, None),
    ("<1.6.16 || >=1.7.0 <1.7.11 || >=1.8.0 <1.8.2", "range-complex", false, None),
    ("<=1.6.16 || >=1.7.0 <1.7.11 || >=1.8.0 <1.8.2", "range-complex", false, None),
    (">1.0.0 <1.0.0", "range-complex", false, None),
    (">1.0.0 <=2.0.0", "range-complex", false, None),
    (">=2.3.4 || <=1.2.3", "range-complex", false, None),
    // workspace protocol
    ("workspace:*", "workspace-protocol", false, Some(SimpleSemver::Latest("*".to_string()))),
    ("workspace:^", "workspace-protocol", false, Some(SimpleSemver::RangeOnly("^".to_string()))),
    ("workspace:~", "workspace-protocol", false, Some(SimpleSemver::RangeOnly("~".to_string()))),
    // simple semver with a semver range
    ("<1.2.3-alpha", "range", true, Some(SimpleSemver::Range("<1.2.3-alpha".to_string()))),
    ("<1.2.3-rc.0", "range", true, Some(SimpleSemver::Range("<1.2.3-rc.0".to_string()))),
    ("<=1.2.3-alpha", "range", true, Some(SimpleSemver::Range("<=1.2.3-alpha".to_string()))),
    ("<=1.2.3-rc.0", "range", true, Some(SimpleSemver::Range("<=1.2.3-rc.0".to_string()))),
    (">1.2.3-alpha", "range", true, Some(SimpleSemver::Range(">1.2.3-alpha".to_string()))),
    (">1.2.3-rc.0", "range", true, Some(SimpleSemver::Range(">1.2.3-rc.0".to_string()))),
    (">=1.2.3-alpha", "range", true, Some(SimpleSemver::Range(">=1.2.3-alpha".to_string()))),
    (">=1.2.3-rc.0", "range", true, Some(SimpleSemver::Range(">=1.2.3-rc.0".to_string()))),
    ("^1.2.3", "range", true, Some(SimpleSemver::Range("^1.2.3".to_string()))),
    ("^1.2.3-alpha", "range", true, Some(SimpleSemver::Range("^1.2.3-alpha".to_string()))),
    ("^1.2.3-rc.0", "range", true, Some(SimpleSemver::Range("^1.2.3-rc.0".to_string()))),
    ("~1.2.3-alpha", "range", true, Some(SimpleSemver::Range("~1.2.3-alpha".to_string()))),
    ("~1.2.3-rc.0", "range", true, Some(SimpleSemver::Range("~1.2.3-rc.0".to_string()))),
    // unsupported
    ("$typescript", "unsupported", false, None),
    ("/path/to/foo", "unsupported", false, None),
    ("/path/to/foo.tar", "unsupported", false, None),
    ("/path/to/foo.tgz", "unsupported", false, None),
    ("1.typo.wat", "unsupported", false, None),
    ("=v1.2.3", "unsupported", false, None),
    ("@f fo o al/ a d s ;f", "unsupported", false, None),
    ("@foo/bar", "unsupported", false, None),
    ("@foo/bar@", "unsupported", false, None),
    ("git+file://path/to/repo#1.2.3", "unsupported", false, None),
    ("not-git@hostname.com:some/repo", "unsupported", false, None),
    ("user/foo#1234::path:dist", "unsupported", false, None),
    ("user/foo#notimplemented:value", "unsupported", false, None),
    ("user/foo#path:dist", "unsupported", false, None),
    ("user/foo#semver:^1.2.3", "unsupported", false, None),
    // tags
    ("alpha", "tag", false, None),
    ("beta", "tag", false, None),
    ("canary", "tag", false, None),
    // range major
    ("~1", "range-major", true, Some(SimpleSemver::RangeMajor("~1".to_string()))),
    // range minor
    ("<5.0", "range-minor", true, Some(SimpleSemver::RangeMinor("<5.0".to_string()))),
    ("<=5.0", "range-minor", true, Some(SimpleSemver::RangeMinor("<=5.0".to_string()))),
    (">5.0", "range-minor", true, Some(SimpleSemver::RangeMinor(">5.0".to_string()))),
    (">=5.0", "range-minor", true, Some(SimpleSemver::RangeMinor(">=5.0".to_string()))),
    ("^4.1", "range-minor", true, Some(SimpleSemver::RangeMinor("^4.1".to_string()))),
    ("~1.2", "range-minor", true, Some(SimpleSemver::RangeMinor("~1.2".to_string()))),
    ("~1.2", "range-minor", true, Some(SimpleSemver::RangeMinor("~1.2".to_string()))),
    // npm aliases
    ("npm:@minh.nguyen/plugin-transform-destructuring@^7.5.2", "alias", false, None), // @TODO: extract semver number
    ("npm:@types/selenium-webdriver@4.1.18", "alias", false, None),                   // @TODO: extract semver number
    ("npm:foo@1.2.3", "alias", false, None),                                          // @TODO: extract semver number
    // file paths
    ("file:../path/to/foo", "file", false, None),
    ("file:./path/to/foo", "file", false, None),
    ("file:/../path/to/foo", "file", false, None),
    ("file:/./path/to/foo", "file", false, None),
    ("file:/.path/to/foo", "file", false, None),
    ("file://.", "file", false, None),
    ("file://../path/to/foo", "file", false, None),
    ("file://./path/to/foo", "file", false, None),
    ("file:////path/to/foo", "file", false, None),
    ("file:///path/to/foo", "file", false, None),
    ("file://path/to/foo", "file", false, None),
    ("file:/path/to/foo", "file", false, None),
    ("file:/~path/to/foo", "file", false, None),
    ("file:path/to/directory", "file", false, None),
    ("file:path/to/foo", "file", false, None),
    ("file:path/to/foo.tar.gz", "file", false, None),
    ("file:path/to/foo.tgz", "file", false, None),
    // git urls
    ("git+https://github.com/user/foo", "git", false, None),
    ("git+ssh://git@github.com/user/foo#1.2.3", "git", false, None),         // @TODO: extract semver number
    ("git+ssh://git@github.com/user/foo#semver:^1.2.3", "git", false, None), // @TODO: extract semver number
    ("git+ssh://git@github.com:user/foo#semver:^1.2.3", "git", false, None), // @TODO: extract semver number
    ("git+ssh://git@notgithub.com/user/foo", "git", false, None),
    ("git+ssh://git@notgithub.com/user/foo#1.2.3", "git", false, None),         // @TODO: extract semver number
    ("git+ssh://git@notgithub.com/user/foo#semver:^1.2.3", "git", false, None), // @TODO: extract semver number
    ("git+ssh://git@notgithub.com:user/foo", "git", false, None),
    ("git+ssh://git@notgithub.com:user/foo#1.2.3", "git", false, None),         // @TODO: extract semver number
    ("git+ssh://git@notgithub.com:user/foo#semver:^1.2.3", "git", false, None), // @TODO: extract semver number
    ("git+ssh://github.com/user/foo", "git", false, None),
    ("git+ssh://github.com/user/foo#1.2.3", "git", false, None),         // @TODO: extract semver number
    ("git+ssh://github.com/user/foo#semver:^1.2.3", "git", false, None), // @TODO: extract semver number
    ("git+ssh://mydomain.com:1234#1.2.3", "git", false, None),           // @TODO: extract semver number
    ("git+ssh://mydomain.com:1234/hey", "git", false, None),
    ("git+ssh://mydomain.com:1234/hey#1.2.3", "git", false, None), // @TODO: extract semver number
    ("git+ssh://mydomain.com:foo", "git", false, None),
    ("git+ssh://mydomain.com:foo#1.2.3", "git", false, None),     // @TODO: extract semver number
    ("git+ssh://mydomain.com:foo/bar#1.2.3", "git", false, None), // @TODO: extract semver number
    ("git+ssh://notgithub.com/user/foo", "git", false, None),
    ("git+ssh://notgithub.com/user/foo#1.2.3", "git", false, None),                  // @TODO: extract semver number
    ("git+ssh://notgithub.com/user/foo#semver:^1.2.3", "git", false, None),          // @TODO: extract semver number
    ("git+ssh://username:password@mydomain.com:1234/hey#1.2.3", "git", false, None), // @TODO: extract semver number
    ("git://github.com/user/foo", "git", false, None),
    ("git://github.com/user/foo#1.2.3", "git", false, None),         // @TODO: extract semver number
    ("git://github.com/user/foo#semver:^1.2.3", "git", false, None), // @TODO: extract semver number
    ("git://notgithub.com/user/foo", "git", false, None),
    ("git://notgithub.com/user/foo#1.2.3", "git", false, None),         // @TODO: extract semver number
    ("git://notgithub.com/user/foo#semver:^1.2.3", "git", false, None), // @TODO: extract semver number
    // urls
    ("http://insecure.com/foo.tgz", "url", false, None),
    ("https://server.com/foo.tgz", "url", false, None),
    ("https://server.com/foo.tgz", "url", false, None),
  ];
  for (value, expected_id, expected_is_simple_semver, expected_get_simple_semver) in cases {
    let spec = Specifier::new(value);
    assert_eq!(spec.get_config_identifier(), expected_id, "{value} should have ID of {expected_id}");
    assert_eq!(spec.get_raw(), value, "{value} should unwrap to {value}");
    assert_eq!(spec.is_simple_semver(), expected_is_simple_semver, "{value} is_simple_semver should be {expected_is_simple_semver}");
    assert_eq!(spec.get_simple_semver(), expected_get_simple_semver, "{value} get_simple_semver should be {expected_get_simple_semver:?}");
  }
}

#[test]
fn normalises_some_node_specifier_strings() {
  let cases: Vec<(&str, &str, bool, &str)> = vec![("latest", "latest", true, "*"), ("x", "latest", true, "*"), ("", "missing", false, "VERSION_IS_MISSING")];
  for (value, expected_id, expected_is_simple_semver, expected_normalisation) in cases {
    let spec = Specifier::new(value);
    assert_eq!(spec.get_config_identifier(), expected_id);
    assert_eq!(spec.get_raw(), expected_normalisation);
    assert_eq!(spec.is_simple_semver(), expected_is_simple_semver);
    assert_eq!(spec.get_simple_semver().is_some(), expected_is_simple_semver);
  }
}

#[test]
fn compares_simple_semver_specifiers_according_to_highest_version_and_greediest_range() {
  let cases: Vec<(&str, &str, Ordering)> = vec![
    /* normal versions */
    ("0.0.0", "0.0.1", Ordering::Less),
    ("0.0.0", "0.1.0", Ordering::Less),
    ("0.0.0", "1.0.0", Ordering::Less),
    ("0.0.0", "0.0.0", Ordering::Equal),
    ("0.0.1", "0.0.0", Ordering::Greater),
    ("0.1.0", "0.0.0", Ordering::Greater),
    ("1.0.0", "0.0.0", Ordering::Greater),
    /* range versions where versions differ */
    ("0.0.0", "~0.0.1", Ordering::Less),
    ("0.0.0", "~0.1.0", Ordering::Less),
    ("0.0.0", "~1.0.0", Ordering::Less),
    ("0.0.1", "~0.0.0", Ordering::Greater),
    ("0.1.0", "~0.0.0", Ordering::Greater),
    ("1.0.0", "~0.0.0", Ordering::Greater),
    ("0.0.0", "^0.0", Ordering::Less),
    ("0", "~0.0", Ordering::Greater),
    ("0", "^0.0", Ordering::Greater),
    /* range greediness applies only when versions are equal */
    ("0.0.0", "~0.0.0", Ordering::Less),
    ("0.0.0", "~0.0", Ordering::Less),
    ("0.0.0", "^0.0.0", Ordering::Less),
    ("0.0", "~0.0", Ordering::Less),
    ("0.0", "^0.0", Ordering::Less),
    ("~0", "^0", Ordering::Less),
    ("0", "~0", Ordering::Less),
    ("0", "^0", Ordering::Less),
    ("0.0.0", ">0.0.0", Ordering::Less),
    ("0.0.0", ">=0.0.0", Ordering::Less),
    ("0.0.0", "<=0.0.0", Ordering::Greater),
    ("0.0.0", "<0.0.0", Ordering::Greater),
    ("0.0.0", "*", Ordering::Less),
    ("^0.0.0", "*", Ordering::Less),
    ("~0.0.0", "*", Ordering::Less),
    (">0.0.0", "*", Ordering::Less),
    (">=0.0.0", "*", Ordering::Less),
    ("<=0.0.0", "*", Ordering::Less),
    ("<0.0.0", "*", Ordering::Less),
    /* an empty or missing specifier is always bottom rank */
    ("", "0.0.0", Ordering::Less),
    ("", "<0.0.0", Ordering::Equal),
    /* stable should be older than tagged */
    ("0.0.0", "0.0.0-alpha", Ordering::Less),
    /* equal tags should not affect comparison */
    ("0.0.0-alpha", "0.0.0-alpha", Ordering::Equal),
    ("0.0.0-alpha", "0.1.0-alpha", Ordering::Less),
    ("0.0.0-alpha", "1.0.0-alpha", Ordering::Less),
    ("0.0.0-alpha", "0.0.0-alpha", Ordering::Equal),
    ("0.0.1-alpha", "0.0.0-alpha", Ordering::Greater),
    ("0.1.0-alpha", "0.0.0-alpha", Ordering::Greater),
    ("1.0.0-alpha", "0.0.0-alpha", Ordering::Greater),
    /* preleases should matter when version is equal */
    ("0.0.0-rc.0.0.0", "0.0.0-rc.0.0.0", Ordering::Equal),
    ("0.0.0-rc.0.0.0", "0.0.0-rc.0.1.0", Ordering::Less),
    ("0.0.0-rc.0.0.0", "0.0.0-rc.1.0.0", Ordering::Less),
    ("0.0.0-rc.0.0.0", "0.0.0-rc.0.0.0", Ordering::Equal),
    ("0.0.0-rc.0.0.1", "0.0.0-rc.0.0.0", Ordering::Greater),
    ("0.0.0-rc.0.1.0", "0.0.0-rc.0.0.0", Ordering::Greater),
    ("0.0.0-rc.1.0.0", "0.0.0-rc.0.0.0", Ordering::Greater),
    /* preleases should not matter when version is greater */
    ("0.1.0-rc.0.0.0", "0.0.0-rc.0.1.0", Ordering::Greater),
    /* compares tags a-z */
    ("0.0.0-alpha", "0.0.0-alpha", Ordering::Equal),
    ("0.0.0-alpha", "0.0.0-beta", Ordering::Less),
    ("0.0.0-beta", "0.0.0-alpha", Ordering::Greater),
    /* range greediness is the same on prereleases */
    ("0.0.0-rc.0", "~0.0.1-rc.0", Ordering::Less),
    ("0.0.0-rc.0", "~0.1.0-rc.0", Ordering::Less),
    ("0.0.0-rc.0", "~1.0.0-rc.0", Ordering::Less),
    ("0.0.0-rc.0", "~0.0.0-rc.0", Ordering::Less),
    ("0.0.1-rc.0", "~0.0.0-rc.0", Ordering::Greater),
    ("0.1.0-rc.0", "~0.0.0-rc.0", Ordering::Greater),
    ("1.0.0-rc.0", "~0.0.0-rc.0", Ordering::Greater),
    ("0.0.0-rc.0", "~0.0.0-rc.0", Ordering::Less),
    ("0.0.0-rc.0", "^0.0.0-rc.0", Ordering::Less),
    ("0.0.0-rc.0", ">0.0.0-rc.0", Ordering::Less),
    ("0.0.0-rc.0", ">=0.0.0-rc.0", Ordering::Less),
    ("0.0.0-rc.0", "<=0.0.0-rc.0", Ordering::Greater),
    ("0.0.0-rc.0", "<0.0.0-rc.0", Ordering::Greater),
  ];
  for (str_a, str_b, expected) in cases {
    let a = Specifier::new(str_a);
    let b = Specifier::new(str_b);
    let orderable_a = a.get_orderable(None);
    let orderable_b = b.get_orderable(None);
    let ordering = orderable_a.cmp(&orderable_b);
    assert_eq!(ordering, expected, "{a:?} should be {expected:?} {b:?} ({orderable_a:#?} {orderable_b:#?})");
  }
}

#[test]
fn compares_workspace_protocol_using_its_semver_specifier_when_available() {
  let cases: Vec<(&str, &str, Ordering)> = vec![
    ("workspace:<1.2.3", "<1.2.3", Ordering::Equal),
    ("workspace:1.2.3", "<1.2.3", Ordering::Greater),
    ("workspace:>1.2.3", "<1.2.3", Ordering::Greater),
    ("workspace:<=1.2.3", "<1.2.3", Ordering::Greater),
  ];
  for (str_a, str_b, expected) in cases {
    let a = Specifier::new(str_a);
    let b = Specifier::new(str_b);
    let orderable_a = a.get_orderable(None);
    let orderable_b = b.get_orderable(None);
    let ordering = orderable_a.cmp(&orderable_b);
    assert_eq!(ordering, expected, "{a:?} should be {expected:?} {b:?} ({orderable_a:#?} {orderable_b:#?})");
  }
}

#[test]
fn compares_workspace_protocol_using_its_local_instance_specifier_when_its_semver_specifier_is_incomplete() {
  let local_specifier = SimpleSemver::new("1.2.3").unwrap();
  let cases: Vec<(&str, &str, Ordering)> = vec![
    ("workspace:*", "*", Ordering::Less),
    ("workspace:*", "1.2.3", Ordering::Greater),
    ("workspace:*", ">1.2.3", Ordering::Greater),
    ("workspace:1", "1.2.3", Ordering::Equal),
    ("workspace:1.1", "1.2.3", Ordering::Less),
    ("workspace:1.2", "1.2.3", Ordering::Equal),
    ("workspace:1.2.3", "1.2.3", Ordering::Equal),
    ("workspace:1.2.3", "1.2.4", Ordering::Less),
    ("workspace:1.2.3", "<1.2.3", Ordering::Greater),
    ("workspace:1.2.3", "<1.2.4", Ordering::Less),
    ("workspace:1.2.3", ">1.2.3", Ordering::Less),
    ("workspace:1.2.3", ">1.2.4", Ordering::Less),
    ("workspace:1.2.4", "1.2.3", Ordering::Greater),
    ("workspace:1.3", "1.2.3", Ordering::Greater),
    ("workspace:<=1", "1.2.3", Ordering::Less),
    ("workspace:^", "<1.2.3", Ordering::Greater),
    ("workspace:^", ">1.2.3", Ordering::Less),
    ("workspace:^", "^1.2.3", Ordering::Equal),
    ("workspace:~", "<1.2.3", Ordering::Greater),
    ("workspace:~", ">1.2.3", Ordering::Less),
    ("workspace:~", "~1.2.3", Ordering::Equal),
    ("workspace:~1", "1.2.3", Ordering::Greater),
  ];
  for (str_a, str_b, expected) in cases {
    let a = Specifier::new(str_a);
    let b = Specifier::new(str_b);
    let orderable_a = a.get_orderable(Some(&local_specifier));
    let orderable_b = b.get_orderable(None);
    let ordering = orderable_a.cmp(&orderable_b);
    assert_eq!(ordering, expected, "{a:?} should be {expected:?} {b:?} ({orderable_a:#?} {orderable_b:#?})");
  }
}

#[test]
fn sorts_simple_semver_specifiers_according_to_highest_version_and_greediest_range() {
  fn to_specifiers(specifiers: Vec<&str>) -> Vec<Specifier> {
    specifiers.iter().map(|r| Specifier::new(r)).collect()
  }
  let mut specifiers = to_specifiers(vec!["0.0.0", "<0.0.0", "*", ">0.0.0", ">=0.0.0", "<=0.0.0", "^0.0.0", "~0.0.0"]);
  let expected = to_specifiers(vec!["<0.0.0", "<=0.0.0", "0.0.0", "~0.0.0", "^0.0.0", ">=0.0.0", ">0.0.0", "*"]);

  specifiers.sort_by_key(|s| s.get_orderable(None));
  assert_eq!(specifiers, expected, "{specifiers:?}, {expected:?}");
}

#[test]
fn states_whether_specifier_satisfies_other_specifiers() {
  let cases: Vec<(&str, Vec<&str>, bool)> = vec![
    ("*", vec!["1.4.2"], true),
    ("^1.4.2", vec!["1.4.2"], true),
    ("1.4.2", vec!["1.4.2"], true),
    (">1.4.2", vec!["1.4.2"], false),
    (">=1.4.2", vec!["1.4.2"], true),
    ("<1.4.2", vec!["1.4.2"], false),
    ("<=1.4.2", vec!["1.4.2"], true),
    ("~1.4.2", vec!["1.4.2"], true),
    ("^1.0.0", vec!["1.4.2"], true),
    ("~1.0.0", vec!["1.4.2"], false),
    ("", vec!["1.4.2"], false),
    ("~1.4.2 || ^1.4.2", vec!["1.4.2"], true),
    ("~1.0.0 || ^1.0.0", vec!["1.4.2"], true),
  ];
  for (value, others, expected) in cases {
    let spec = Specifier::new(value);
    let other_specs: Vec<Specifier> = others.iter().map(|r| Specifier::new(r)).collect();
    let refs_to_other_specs: Vec<&Specifier> = other_specs.iter().collect();
    assert_eq!(spec.satisfies_all(refs_to_other_specs), expected, "'{value}'.satisfies_all({others:?}) should be {expected}");
  }
}
