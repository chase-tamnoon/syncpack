use std::collections::HashSet;
use std::vec;

use crate::instance::Instance;

#[derive(Debug)]
pub struct InstanceGroup<'a> {
  /// Every instance of this dependency in this version group.
  pub all: Vec<&'a Instance<'a>>,
  /// The version specifier which all instances in this group should have
  pub expected_version: Option<String>,
  /// If this dependency is a local package, this is the local instance.
  pub local: Option<&'a Instance<'a>>,
  /// All instances with `SpecifierType::NonSemver` versions
  pub non_semver: Vec<&'a Instance<'a>>,
  /// All instances with `SpecifierType::Semver` versions
  pub semver: Vec<&'a Instance<'a>>,
  /// Raw version specifiers for each dependency. If there is more than one
  /// unique version, then we have mismatching versions
  pub unique_specifiers: HashSet<String>,
}

impl<'a> InstanceGroup<'a> {
  pub fn new() -> InstanceGroup<'a> {
    InstanceGroup {
      all: vec![],
      expected_version: None,
      local: None,
      non_semver: vec![],
      semver: vec![],
      unique_specifiers: HashSet::new(),
    }
  }

  pub fn is_mismatch(&self, specifier: &String) -> bool {
    match &self.expected_version {
      Some(expected_version) => specifier != expected_version,
      None => false,
    }
  }
}
