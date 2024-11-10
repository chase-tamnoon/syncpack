#[cfg(test)]
#[path = "specifier2_test.rs"]
mod specifier_test;

use {
  crate::specifier::{parser, semver_range::SemverRange},
  node_semver::{Identifier, Version},
};

pub enum SemverType {
  /// eg. `*`
  Latest,
  /// eg. `1`
  Major,
  /// eg. `1.2`
  Minor,
  /// eg. `>1.2.3`
  Range,
  /// eg. `^`
  RangeOnly,
  /// eg. `>1`
  RangeMajor,
  /// eg. `^1.2`
  RangeMinor,
}

#[derive(Debug, PartialEq)]
pub enum Protocol {
  /// eg. `workspace:`
  Workspace,
}

impl Protocol {
  /// Get the string representation of the range
  pub fn get_raw(&self) -> &str {
    match self {
      Protocol::Workspace => "workspace:",
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct SemverSpecifier {
  pub range: Option<SemverRange>,
  pub major: Option<u64>,
  pub minor: Option<u64>,
  pub patch: Option<u64>,
  pub build: Vec<Identifier>,
  pub pre_release: Vec<Identifier>,
  pub canonical: Option<Version>,
  pub protocol: Option<Protocol>,
}

impl SemverSpecifier {
  pub fn new() -> Self {
    Self {
      range: None,
      major: None,
      minor: None,
      patch: None,
      build: vec![],
      pre_release: vec![],
      canonical: None,
      protocol: None,
    }
  }

  pub fn parse(raw: &str) -> Self {
    if parser::is_workspace_protocol(raw) {
      let mut specifier = Self::parse(&raw.replace("workspace:", ""));
      specifier.protocol = Some(Protocol::Workspace);
      return specifier;
    }
    let raw = parser::sanitise(raw);
    if parser::is_latest(raw) {
      let mut specifier = Self::new();
      specifier.range = Some(SemverRange::Any);
      return specifier;
    }
    if parser::is_range_only(raw) {
      let mut specifier = Self::new();
      specifier.range = SemverRange::new(raw);
      return specifier;
    }
    let (range, tail) = if let Some((range, tail)) = SemverRange::split(raw) { (Some(range), tail) } else { (None, raw) };
    if parser::is_major(tail) {
      let mut specifier = Self::new();
      specifier.range = range;
      specifier.major = tail.parse::<u64>().ok();
      return specifier;
    }
    if parser::is_minor(tail) {
      let mut specifier = Self::new();
      specifier.range = range;
      let mut parts = tail.split('.');
      let mut parse_next = || parts.next().and_then(|n| n.parse::<u64>().ok());
      specifier.major = parse_next();
      specifier.minor = parse_next();
      return specifier;
    }
    if let Ok(version) = Version::parse(tail) {
      let mut specifier = Self::new();
      specifier.range = range;
      specifier.major = Some(version.major);
      specifier.minor = Some(version.minor);
      specifier.patch = Some(version.patch);
      specifier.build = version.build;
      specifier.pre_release = version.pre_release;
      return specifier;
    }
    Self::new()
  }

  pub fn to_raw(&self) -> String {
    let protocol = self.protocol.as_ref().map_or("", |r| r.get_raw());
    let range = self.range.as_ref().map_or("", |r| r.get_raw());
    let semver = match (self.major, self.minor, self.patch) {
      (Some(major), Some(minor), Some(patch)) => Version {
        major,
        minor,
        patch,
        build: self.build.clone(),
        pre_release: self.pre_release.clone(),
      }
      .to_string(),
      (Some(major), Some(minor), None) => format!("{}.{}", major, minor),
      (Some(major), None, None) => format!("{}", major),
      _ => String::from(""),
    };
    format!("{protocol}{range}{semver}")
  }
}
