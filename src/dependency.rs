use std::{
  collections::{BTreeMap, HashMap},
  vec,
};

use crate::instance::{Instance, InstanceId};

/// A reference to a group of instances of the same dependency which all have the
/// same version specifier.
#[derive(Debug)]
pub struct InstanceIdsBySpecifier {
  pub specifier: String,
  pub instance_ids: Vec<InstanceId>,
}

/// The location which owns all instances
pub type InstancesById = BTreeMap<InstanceId, Instance>;

#[derive(Debug)]
pub struct Dependency {
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

impl Dependency {
  pub fn new(name: String) -> Dependency {
    Dependency {
      name,
      all: vec![],
      expected_version: None,
      local: None,
      non_semver: vec![],
      semver: vec![],
      by_specifier: HashMap::new(),
    }
  }

  /// Does this group contain a package developed in this repo?
  pub fn is_local_package(&self, instance_id: &String) -> bool {
    self.local.is_some()
  }

  /// Is this instance the .version of a local package?
  pub fn is_local_instance(&self, instance_id: &String) -> bool {
    self
      .local
      .as_ref()
      .filter(|local_id| *local_id == instance_id)
      .is_some()
  }

  /// Iterate over every instance ID and its specifier in this group
  pub fn for_each_instance_id<F>(&self, mut handler: F)
  where
    F: FnMut((&String, &InstanceId)),
  {
    self
      .by_specifier
      .iter()
      .for_each(|(specifier, instance_ids)| {
        instance_ids.iter().for_each(|instance_id| {
          handler((specifier, instance_id));
        });
      });
  }

  /// Iterate over every unique specifier and its instance IDs in this group
  pub fn for_each_specifier<F>(&self, mut handler: F)
  where
    F: FnMut((&String, &Vec<InstanceId>)),
  {
    self
      .by_specifier
      .iter()
      .for_each(|(specifier, instance_ids)| {
        handler((specifier, instance_ids));
      });
  }

  /// Get the IDs of all instances whose version specifier matches the expected
  pub fn get_matching_instance_ids(&self) -> Vec<InstanceId> {
    self
      .expected_version
      .as_ref()
      .and_then(|expected_version| self.by_specifier.get(expected_version))
      .map(|ids| ids.clone())
      .unwrap_or_else(|| vec![])
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
