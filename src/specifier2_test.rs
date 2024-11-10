use node_semver::Identifier;

use crate::{
  specifier::semver_range::SemverRange,
  specifier2::{Protocol, SemverSpecifier},
};

#[test]
fn parses_and_serialises_npm_version_specifiers() {
  let cases: Vec<(&str, SemverSpecifier)> = vec![
    (
      "*",
      SemverSpecifier {
        range: Some(SemverRange::Any),
        ..SemverSpecifier::new()
      },
    ),
    (
      "^",
      SemverSpecifier {
        range: Some(SemverRange::Minor),
        ..SemverSpecifier::new()
      },
    ),
    (
      "~",
      SemverSpecifier {
        range: Some(SemverRange::Patch),
        ..SemverSpecifier::new()
      },
    ),
    (
      "workspace:*",
      SemverSpecifier {
        range: Some(SemverRange::Any),
        protocol: Some(Protocol::Workspace),
        ..SemverSpecifier::new()
      },
    ),
    (
      "workspace:^",
      SemverSpecifier {
        range: Some(SemverRange::Minor),
        protocol: Some(Protocol::Workspace),
        ..SemverSpecifier::new()
      },
    ),
    (
      "1.2.3",
      SemverSpecifier {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        ..SemverSpecifier::new()
      },
    ),
    (
      "workspace:1.2.3",
      SemverSpecifier {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        protocol: Some(Protocol::Workspace),
        ..SemverSpecifier::new()
      },
    ),
    ("1", SemverSpecifier { major: Some(1), ..SemverSpecifier::new() }),
    (
      "^1",
      SemverSpecifier {
        range: Some(SemverRange::Minor),
        major: Some(1),
        ..SemverSpecifier::new()
      },
    ),
    (
      "1.2",
      SemverSpecifier {
        major: Some(1),
        minor: Some(2),
        ..SemverSpecifier::new()
      },
    ),
    (
      "~1.2",
      SemverSpecifier {
        range: Some(SemverRange::Patch),
        major: Some(1),
        minor: Some(2),
        ..SemverSpecifier::new()
      },
    ),
    (
      "1.2.3-alpha",
      SemverSpecifier {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: vec![Identifier::AlphaNumeric("alpha".to_string())],
        ..SemverSpecifier::new()
      },
    ),
    (
      "1.2.3-rc.0",
      SemverSpecifier {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: vec![Identifier::AlphaNumeric("rc".to_string()), Identifier::Numeric(0)],
        ..SemverSpecifier::new()
      },
    ),
    (
      "<1.2.3-alpha",
      SemverSpecifier {
        range: Some(SemverRange::Lt),
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: vec![Identifier::AlphaNumeric("alpha".to_string())],
        ..SemverSpecifier::new()
      },
    ),
    (
      "<=1.2.3-alpha",
      SemverSpecifier {
        range: Some(SemverRange::Lte),
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: vec![Identifier::AlphaNumeric("alpha".to_string())],
        ..SemverSpecifier::new()
      },
    ),
    (
      "<1.2",
      SemverSpecifier {
        range: Some(SemverRange::Lt),
        major: Some(1),
        minor: Some(2),
        ..SemverSpecifier::new()
      },
    ),
    (
      "<=1.2",
      SemverSpecifier {
        range: Some(SemverRange::Lte),
        major: Some(1),
        minor: Some(2),
        ..SemverSpecifier::new()
      },
    ),
  ];
  for (raw, expected) in cases {
    let actual = SemverSpecifier::parse(raw);
    assert_eq!(actual, expected, "'{raw}' {actual:#?} should equal {expected:#?}");
    assert_eq!(actual.to_raw(), raw.to_string(), "should be able to serialise {raw}");
  }
}
