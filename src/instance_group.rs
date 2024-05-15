use std::collections::{BTreeMap, HashMap};
use std::vec;

use crate::instance::{Instance, InstanceId};

/// A reference to a group of instances of the same dependency which all have the
/// same version specifier.
pub type InstanceIdsBySpecifier = (String, Vec<InstanceId>);

/// The location which owns all instances
pub type InstancesById = BTreeMap<InstanceId, Instance>;

#[derive(Debug)]
pub struct InstanceGroup {
  /// The name of the dependency
  pub name: String,
  /// Every instance of this dependency in this version group.
  pub all: Vec<InstanceId>,
  /// The version specifier which all instances in this group should have
  pub expected_version: Option<String>,
  /// If this dependency is a local package, this is the local instance.
  pub local: Option<InstanceId>,
  /// All instances with `Specifier::NonSemver` versions
  pub non_semver: Vec<InstanceId>,
  /// All instances with `Specifier::Semver` versions
  pub semver: Vec<InstanceId>,
  /// Each key is a unique raw version specifier for each dependency. The values
  /// are each instance which has that version specifier.
  ///
  /// If there is more than one unique version, then we have mismatches
  pub by_specifier: HashMap<String, Vec<InstanceId>>,
}

impl InstanceGroup {
  pub fn new(name: String) -> InstanceGroup {
    InstanceGroup {
      name,
      all: vec![],
      expected_version: None,
      local: None,
      non_semver: vec![],
      semver: vec![],
      by_specifier: HashMap::new(),
    }
  }

  /// Is the exact same specifier used by all instances in this group?
  pub fn has_identical_specifiers(&self) -> bool {
    self.by_specifier.len() == (1 as usize)
  }

  pub fn is_mismatch(&self, actual: &String) -> bool {
    // if we determined an expected version... (such as the highest semver version,
    // the local dependency version, or a pinned version)
    match &self.expected_version {
      // ...we can just check if this one matches it
      Some(expected) => actual != expected,
      // if no expected version was suggested, this is because...
      None => match self.non_semver.len() {
        // ...something went badly wrong
        0 => panic!("An expected version was not set for a group with no non-semver versions"),
        // ...or we have an `UnsupportedMismatch`
        _ => true,
      },
    }
  }
}
