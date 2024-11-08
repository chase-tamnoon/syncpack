#[cfg(test)]
#[path = "specifier_test.rs"]
mod specifier_test;

use {
  crate::specifier::{
    non_semver::NonSemver,
    orderable::{IsOrderable, Orderable},
    semver::Semver,
    simple_semver::SimpleSemver,
  },
  semver_range::SemverRange,
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
          SimpleSemver::RangeOnly(_) => "range-only",
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
  pub fn get_raw(&self) -> String {
    match self {
      Self::Semver(simple_semver) => match simple_semver {
        Semver::Simple(variant) => match variant {
          SimpleSemver::Exact(string)
          | SimpleSemver::Latest(string)
          | SimpleSemver::Major(string)
          | SimpleSemver::Minor(string)
          | SimpleSemver::Range(string)
          | SimpleSemver::RangeMajor(string)
          | SimpleSemver::RangeMinor(string)
          | SimpleSemver::RangeOnly(string) => string.clone(),
        },
        Semver::Complex(string) => string.clone(),
      },
      Self::NonSemver(non_semver) => match non_semver {
        NonSemver::Alias(string) | NonSemver::File(string) | NonSemver::Git(string) | NonSemver::Tag(string) | NonSemver::Url(string) | NonSemver::WorkspaceProtocol(string) | NonSemver::Unsupported(string) => string.clone(),
      },
      Self::None => "VERSION_IS_MISSING".to_string(),
    }
  }

  /// Is this specifier semver, without &&s or ||s?
  pub fn is_simple_semver(&self) -> bool {
    matches!(self, Specifier::Semver(Semver::Simple(_)))
  }

  pub fn is_workspace_protocol(&self) -> bool {
    matches!(self, Specifier::NonSemver(NonSemver::WorkspaceProtocol(_)))
  }

  /// If this specifier contains simple semver, return it
  pub fn get_simple_semver(&self) -> Option<SimpleSemver> {
    match self {
      Self::Semver(Semver::Simple(simple_semver)) => Some(simple_semver.clone()),
      Self::NonSemver(NonSemver::WorkspaceProtocol(raw)) => SimpleSemver::new(&raw.replace("workspace:", "")).ok(),
      _ => None,
    }
  }

  /// Get the semver range for this specifier, if it has one
  pub fn get_semver_range(&self) -> Option<SemverRange> {
    if let Specifier::Semver(Semver::Simple(simple_semver)) = self {
      Some(simple_semver.get_range())
    } else {
      None
    }
  }

  /// Does this specifier have the given semver range?
  pub fn has_semver_range_of(&self, range: &SemverRange) -> bool {
    self.get_simple_semver().map_or(false, |s| s.has_semver_range_of(range))
  }

  /// Regardless of the range, does this specifier and the other both have eg.
  /// "1.4.1" as their version?
  pub fn has_same_version_number_as(&self, other: &Self) -> bool {
    match (self.get_simple_semver(), other.get_simple_semver()) {
      (Some(a), Some(b)) => a.has_same_version_number_as(&b),
      _ => false,
    }
  }

  /// Try to parse this specifier into one from the `node_semver` crate
  fn parse_with_node_semver(&self) -> Result<node_semver::Range, node_semver::SemverError> {
    match self {
      Self::NonSemver(NonSemver::WorkspaceProtocol(raw)) => self.get_raw().replace("workspace:", ""),
      _ => self.get_raw(),
    }
    .parse::<node_semver::Range>()
  }

  /// Does this specifier match every one of the given specifiers?
  pub fn satisfies_all(&self, others: Vec<&Self>) -> bool {
    match self.parse_with_node_semver() {
      Ok(a) => others.iter().flat_map(|other| other.parse_with_node_semver()).all(|b| a.allows_any(&b)),
      _ => false,
    }
  }

  /// Does this specifier match the given specifier?
  pub fn satisfies(&self, other: &Self) -> bool {
    self.satisfies_all(vec![other])
  }
}

impl IsOrderable for Specifier {
  /// Return a struct which can be used to check equality or sort specifiers
  fn get_orderable(&self, canonical_specifier: Option<&SimpleSemver>) -> Orderable {
    match self {
      Self::Semver(semver) => semver.get_orderable(canonical_specifier),
      Self::NonSemver(non_semver) => non_semver.get_orderable(canonical_specifier),
      Self::None => Orderable::new(),
    }
  }
}
