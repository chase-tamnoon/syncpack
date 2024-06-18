use std::{
  collections::{BTreeMap, HashMap},
  vec,
};

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
  /// What behaviour has this group been configured to exhibit?
  pub variant: Variant,
  /// The name of the dependency
  pub name: String,
  /// Every instance of this dependency in this version group.
  pub all: Vec<InstanceId>,
  /// The version specifier which all instances in this group should have
  pub expected_version: Option<Specifier>,
  /// If this dependency is a local package, this is the local instance.
  pub local: Option<InstanceId>,
  /// All instances with `Specifier::NonSemver` versions
  pub non_semver: Vec<InstanceId>,
  /// All instances with `Specifier::Semver` versions
  pub semver: Vec<InstanceId>,
  /// Each key is a unique raw version specifier for each dependency.
  /// The values are each instance which has that version specifier.
  ///
  /// If there is more than one unique version, then we have mismatches
  pub by_initial_specifier: HashMap<Specifier, Vec<InstanceId>>,
  /// The version to pin all instances to when variant is `Pinned`
  pub pin_version: Option<Specifier>,
  /// `name` properties of package.json files developed in the monorepo when variant is `SnappedTo`
  pub snap_to: Option<Vec<String>>,
}

impl Dependency {
  pub fn new(group: &VersionGroup, name: String) -> Dependency {
    Dependency {
      variant: group.variant.clone(),
      name,
      all: vec![],
      expected_version: None,
      local: None,
      non_semver: vec![],
      semver: vec![],
      by_initial_specifier: HashMap::new(),
      pin_version: group.pin_version.clone(),
      snap_to: group.snap_to.clone(),
    }
  }

  pub fn add_instance(&mut self, instance: Instance) -> Instance {
    // Track/count all instances
    self.all.push(instance.id.clone());
    // Store by initial specifier
    self
      .by_initial_specifier
      .entry(instance.initial_specifier.clone())
      .or_insert_with(|| vec![])
      .push(instance.id.clone());
    // Set local instance
    if instance.is_local {
      self.local = Some(instance.id.clone());
    }
    // Track/count what specifier types we have encountered
    if instance.initial_specifier.is_semver() {
      self.semver.push(instance.id.clone());
    } else {
      self.non_semver.push(instance.id.clone());
    }

    if matches!(self.variant, Variant::Pinned) {
      self.expected_version = self.pin_version.clone();
      return;
    }

    if matches!(self.variant, Variant::Standard) {
      // If this is the original source of a locally-developed package, set it
      // as the preferred version
      if &instance.dependency_type.name == "local" {
        self.expected_version = Some(instance.specifier.clone());
      }

      // A locally-developed package version overrides every other, so if one
      // has not been found, we need to look at the usages of it for a preferred
      // version
      if self.local.is_none() {
        if instance.specifier.is_semver() && self.non_semver.len() == 0 {
          // Have we set a preferred version yet for these instances?
          match &mut self.expected_version {
            // No, this is the first candidate.
            None => {
              self.expected_version = Some(instance.specifier.clone());
            }
            // Yes, compare this candidate with the previous one
            Some(expected_version) => {
              let this_version = &instance.specifier;
              let prefer_lowest = matches!(&self.prefer_version, Some(PreferVersion::LowestSemver));
              let preferred_order = if prefer_lowest { Cmp::Lt } else { Cmp::Gt };
              match compare(this_version.unwrap(), &expected_version.unwrap()) {
                Ok(actual_order) => {
                  if preferred_order == actual_order {
                    self.expected_version = Some(instance.specifier.clone());
                  }
                }
                Err(_) => {
                  panic!(
                    "Cannot compare {:?} and {:?}",
                    &this_version, &expected_version
                  );
                }
              };
            }
          }
        } else {
          // clear any previous preferred version if we encounter a non-semver
          self.expected_version = None;
        }
      }
    }

    instance
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
    F: FnMut((&Specifier, &InstanceId)),
  {
    self
      .by_initial_specifier
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
    F: FnMut((&Specifier, &Vec<InstanceId>)),
  {
    self
      .by_initial_specifier
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
      .and_then(|expected_version| self.by_initial_specifier.get(expected_version))
      .map(|ids| ids.clone())
      .unwrap_or_else(|| vec![])
  }

  /// Is the exact same specifier used by all instances in this group?
  pub fn has_identical_specifiers(&self) -> bool {
    self.by_initial_specifier.len() == (1 as usize)
  }

  pub fn is_version_mismatch(&self, actual: &Specifier) -> bool {
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
