use itertools::Itertools;
use std::{
  collections::{BTreeMap, HashMap},
  vec,
};
use version_compare::{compare, Cmp};

use crate::{
  instance::{Instance, InstanceId},
  specifier::Specifier,
  version_group::{Variant, VersionGroup},
};

/// A reference to a group of instances of the same dependency which all have the
/// same version specifier.
#[derive(Debug)]
pub struct InstanceIdsBySpecifier {
  pub specifier: Specifier,
  pub instance_ids: Vec<InstanceId>,
}

/// The location which owns all instances
pub type InstancesById = BTreeMap<InstanceId, Instance>;

#[derive(Debug)]
pub struct Dependency {
  /// Every instance of this dependency in this version group.
  pub all: Vec<InstanceId>,
  /// If this dependency is a local package, this is the local instance.
  pub local_instance_id: Option<InstanceId>,
  /// The name of the dependency
  pub name: String,
  /// The version to pin all instances to when variant is `Pinned`
  pub pinned_specifier: Option<Specifier>,
  /// `name` properties of package.json files developed in the monorepo when variant is `SnappedTo`
  pub snapped_to_package_names: Option<Vec<String>>,
  /// What behaviour has this group been configured to exhibit?
  pub variant: Variant,
}

impl Dependency {
  pub fn new(
    name: String,
    variant: Variant,
    pinned_specifier: Option<Specifier>,
    snapped_to_package_names: Option<Vec<String>>,
  ) -> Dependency {
    Dependency {
      all: vec![],
      local_instance_id: None,
      name,
      pinned_specifier,
      snapped_to_package_names,
      variant,
    }
  }

  pub fn get_instances<'a>(
    &'a self,
    instances_by_id: &'a InstancesById,
  ) -> impl Iterator<Item = &'a Instance> {
    self
      .all
      .iter()
      .map(move |instance_id| instances_by_id.get(instance_id).unwrap())
  }

  pub fn has_local_instance(&self) -> bool {
    self.local_instance_id.is_some()
  }

  pub fn get_local_specifier(&self, instances_by_id: &InstancesById) -> Option<Specifier> {
    self
      .get_instances(instances_by_id)
      .find(|instance| instance.is_local)
      .map(|instance| instance.specifier.clone())
  }

  pub fn all_are_semver(&self, instances_by_id: &InstancesById) -> bool {
    self
      .get_instances(instances_by_id)
      .all(|instance| instance.specifier.is_semver())
  }

  pub fn get_highest_semver(&self, instances_by_id: &InstancesById) -> Option<Specifier> {
    self.get_preferred_semver(instances_by_id, Cmp::Gt)
  }

  pub fn get_lowest_semver(&self, instances_by_id: &InstancesById) -> Option<Specifier> {
    self.get_preferred_semver(instances_by_id, Cmp::Lt)
  }

  pub fn get_preferred_semver(
    &self,
    instances_by_id: &InstancesById,
    preferred_order: Cmp,
  ) -> Option<Specifier> {
    self
      .get_instances(instances_by_id)
      .fold(None, |highest, instance| match highest {
        None => Some(&instance.specifier),
        Some(highest) => match compare(instance.specifier.unwrap(), highest.unwrap()) {
          Ok(actual_order) => {
            if actual_order == preferred_order {
              Some(&instance.specifier)
            } else {
              Some(highest)
            }
          }
          Err(_) => {
            panic!(
              "Cannot compare {:?} and {:?}",
              &instance.specifier, &highest
            );
          }
        },
      })
      .map(|specifier| specifier.clone())
  }

  /// Each key is a unique raw version specifier for each dependency.
  /// The values are each instance which has that version specifier.
  ///
  /// If there is more than one unique version, then we have mismatches
  pub fn group_by_specifier<'a>(
    &'a self,
    instances_by_id: &'a InstancesById,
  ) -> HashMap<Specifier, Vec<&'a Instance>> {
    self
      .get_instances(instances_by_id)
      .fold(HashMap::new(), |mut acc, instance| {
        acc
          .entry(instance.specifier.clone())
          .or_insert_with(|| vec![])
          .push(&instance);
        acc
      })
  }

  pub fn add_instance(&mut self, instance: Instance) -> Instance {
    // Track/count all instances
    self.all.push(instance.id.clone());
    // Set local instance
    if instance.is_local {
      self.local_instance_id = Some(instance.id.clone());
    }
    instance
  }

  /// Does this group contain a package developed in this repo?
  // pub fn is_local_package(&self, instance_id: &String) -> bool {
  //   self.local_instance_id.is_some()
  // }

  // /// Is this instance the .version of a local package?
  // pub fn is_local_instance(&self, instance_id: &String) -> bool {
  //   self
  //     .local_instance_id
  //     .as_ref()
  //     .filter(|local_id| *local_id == instance_id)
  //     .is_some()
  // }

  /// Iterate over every instance ID and its specifier in this group
  // pub fn for_each_instance_id<F>(&self, mut handler: F)
  // where
  //   F: FnMut((&Specifier, &InstanceId)),
  // {
  //   self
  //     .by_initial_specifier
  //     .iter()
  //     .for_each(|(specifier, instance_ids)| {
  //       instance_ids.iter().for_each(|instance_id| {
  //         handler((specifier, instance_id));
  //       });
  //     });
  // }

  /// Iterate over every unique specifier and its instance IDs in this group
  // pub fn for_each_specifier<F>(&self, mut handler: F)
  // where
  //   F: FnMut((&Specifier, &Vec<InstanceId>)),
  // {
  //   self
  //     .by_initial_specifier
  //     .iter()
  //     .for_each(|(specifier, instance_ids)| {
  //       handler((specifier, instance_ids));
  //     });
  // }

  /// Get the IDs of all instances whose version specifier matches the expected
  // pub fn get_matching_instance_ids(&self) -> Vec<InstanceId> {
  //   self
  //     .expected_version
  //     .as_ref()
  //     .and_then(|expected_version| self.by_initial_specifier.get(expected_version))
  //     .map(|ids| ids.clone())
  //     .unwrap_or_else(|| vec![])
  // }

  /// Is the exact same specifier used by all instances in this group?
  pub fn all_specifiers_are_identical(&self, instances_by_id: &InstancesById) -> bool {
    let mut previous: Option<&Specifier> = None;
    for instance in self.get_instances(instances_by_id) {
      if let Some(value) = previous {
        if *value != instance.specifier {
          return false;
        }
      }
      previous = Some(&instance.specifier);
    }
    return true;
  }

  // pub fn is_version_mismatch(&self, actual: &Specifier) -> bool {
  //   // if we determined an expected version... (such as the highest semver version,
  //   // the local dependency version, or a pinned version)
  //   match &self.expected_version {
  //     // ...we can just check if this one matches it
  //     Some(expected) => actual != expected,
  //     // if no expected version was suggested, this is because...
  //     None => match self.non_semver.len() {
  //       // ...something went badly wrong
  //       0 => panic!("An expected version was not set for a group with no non-semver versions"),
  //       // ...or we have an `UnsupportedMismatch`
  //       _ => true,
  //     },
  //   }
  // }
}
