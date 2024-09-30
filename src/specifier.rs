#[cfg(test)]
#[path = "specifier_test.rs"]
mod specifier_test;

use crate::specifier::{
  non_semver::NonSemver,
  orderable::{IsOrderable, Orderable},
  semver::Semver,
  simple_semver::SimpleSemver,
};

pub mod non_semver;
pub mod orderable;
pub mod parser;
pub mod regexes;
pub mod semver;
pub mod semver_range;
pub mod simple_semver;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Specifier {
  None,
  Semver(Semver),
  NonSemver(NonSemver),
}

impl Specifier {
  pub fn new(specifier: &str) -> Self {
    let str = parser::sanitise(specifier);
    let string = str.to_string();
    if specifier.is_empty() {
      Self::None
    } else if let Ok(semver) = Semver::new(str) {
      Self::Semver(semver)
    } else {
      Self::NonSemver(NonSemver::new(str))
    }
  }

  /// Get the `specifier_type` name as used in config files.
  pub fn get_config_identifier(&self) -> String {
    match self {
      Self::Semver(simple_semver) => match simple_semver {
        Semver::Simple(variant) => match variant {
          SimpleSemver::Exact(_) => "exact",
          SimpleSemver::Latest(_) => "latest",
          SimpleSemver::Major(_) => "major",
          SimpleSemver::Minor(_) => "minor",
          SimpleSemver::Range(_) => "range",
          SimpleSemver::RangeMajor(_) => "range-major",
          SimpleSemver::RangeMinor(_) => "range-minor",
        },
        Semver::Complex(_) => "range-complex",
      },
      Self::NonSemver(non_semver) => match non_semver {
        NonSemver::Alias(_) => "alias",
        NonSemver::File(_) => "file",
        NonSemver::Git(_) => "git",
        NonSemver::Tag(_) => "tag",
        NonSemver::Url(_) => "url",
        NonSemver::WorkspaceProtocol(_) => "workspace-protocol",
        NonSemver::Unsupported(_) => "unsupported",
      },
      Self::None => "missing",
    }
    .to_string()
  }

  /// Get the raw string value of the specifier, eg "^1.4.1"
  pub fn unwrap(&self) -> String {
    match self {
      Self::Semver(simple_semver) => match simple_semver {
        Semver::Simple(variant) => match variant {
          SimpleSemver::Exact(string) => string.clone(),
          SimpleSemver::Latest(string) => string.clone(),
          SimpleSemver::Major(string) => string.clone(),
          SimpleSemver::Minor(string) => string.clone(),
          SimpleSemver::Range(string) => string.clone(),
          SimpleSemver::RangeMajor(string) => string.clone(),
          SimpleSemver::RangeMinor(string) => string.clone(),
        },
        Semver::Complex(string) => string.clone(),
      },
      Self::NonSemver(non_semver) => match non_semver {
        NonSemver::Alias(string) => string.clone(),
        NonSemver::File(string) => string.clone(),
        NonSemver::Git(string) => string.clone(),
        NonSemver::Tag(string) => string.clone(),
        NonSemver::Url(string) => string.clone(),
        NonSemver::WorkspaceProtocol(string) => string.clone(),
        NonSemver::Unsupported(string) => string.clone(),
      },
      Self::None => "VERSION_IS_MISSING".to_string(),
    }
  }

  /// Is this specifier semver, without &&s or ||s?
  pub fn is_simple_semver(&self) -> bool {
    matches!(self, Specifier::Semver(Semver::Simple(_)))
  }

  /// If this specifier is a simple semver, return it
  pub fn get_simple_semver(&self) -> Option<SimpleSemver> {
    if let Specifier::Semver(Semver::Simple(simple_semver)) = self {
      Some(simple_semver.clone())
    } else {
      None
    }
  }
}

impl IsOrderable for Specifier {
  /// Return a struct which can be used to check equality or sort specifiers
  fn get_orderable(&self) -> Orderable {
    match self {
      Self::Semver(semver) => semver.get_orderable(),
      Self::NonSemver(non_semver) => non_semver.get_orderable(),
      Self::None => Orderable::new(),
    }
  }
}
