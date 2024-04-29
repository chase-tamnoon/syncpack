use std::collections::HashSet;
use std::vec;

use crate::instance::Instance;

#[derive(Debug)]
pub struct InstanceGroup<'a> {
  /// Every instance of this dependency in this version group.
  pub all: Vec<&'a Instance<'a>>,
  /// If this dependency is a local package, this is the local instance.
  pub local: Option<&'a Instance<'a>>,
  /// All instances with `SpecifierType::NonSemver` versions
  pub non_semver: Vec<&'a Instance<'a>>,
  /// The highest or lowest version to use if all are valid, or the local
  /// version if this is a package developed in this repo.
  pub preferred_version: Option<String>,
  /// All instances with `SpecifierType::Semver` versions
  pub semver: Vec<&'a Instance<'a>>,
  /// Raw version specifiers for each dependency.
  pub unique_specifiers: HashSet<String>,
}

impl<'a> InstanceGroup<'a> {
  pub fn new() -> InstanceGroup<'a> {
    InstanceGroup {
      all: vec![],
      local: None,
      non_semver: vec![],
      preferred_version: None,
      semver: vec![],
      unique_specifiers: HashSet::new(),
    }
  }

  pub fn is_mismatch(&self, specifier: &String) -> bool {
    match &self.preferred_version {
      Some(preferred_version) => specifier != preferred_version,
      None => false,
    }
  }
}
