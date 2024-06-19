use node_semver::Range;
use std::{
  collections::{BTreeMap, HashMap, HashSet},
  vec,
};
use version_compare::{compare, Cmp};

use crate::{
  instance::{Instance, InstanceId},
  specifier::Specifier,
  version_group::Variant,
};

/// A reference to a group of instances of the same dependency which all have the
/// same version specifier.
#[derive(Debug)]
pub struct InstanceIdsBySpecifier {
  pub specifier: Specifier,
  pub instance_ids: Vec<InstanceId>,
}

/// A reference to a group of instances of the same dependency which all have the
/// same version specifier.
#[derive(Debug)]
pub struct InstancesBySpecifier<'a> {
  pub specifier: Specifier,
  pub instances: Vec<&'a Instance>,
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

  pub fn add_instance(&mut self, instance: Instance) -> Instance {
    // Track/count all instances
    self.all.push(instance.id.clone());
    // Set local instance
    if instance.is_local {
      self.local_instance_id = Some(instance.id.clone());
    }
    instance
  }

  pub fn get_instances<'a>(&'a self, instances_by_id: &'a InstancesById) -> Vec<&'a Instance> {
    self
      .all
      .iter()
      .map(move |instance_id| instances_by_id.get(instance_id).unwrap())
      .collect()
  }

  pub fn has_local_instance(&self) -> bool {
    self.local_instance_id.is_some()
  }

  pub fn has_preferred_ranges(&self, instances_by_id: &InstancesById) -> bool {
    self
      .get_instances(instances_by_id)
      .iter()
      .any(|instance| instance.prefer_range.is_some())
  }

  pub fn get_local_specifier(&self, instances_by_id: &InstancesById) -> Option<Specifier> {
    self
      .get_instances(instances_by_id)
      .iter()
      .find(|instance| instance.is_local)
      .map(|instance| instance.actual.clone())
  }

  pub fn all_are_semver(&self, instances_by_id: &InstancesById) -> bool {
    self
      .get_instances(instances_by_id)
      .iter()
      .all(|instance| instance.actual.is_semver())
  }

  pub fn get_unique_specifiers(&self, instances_by_id: &InstancesById) -> HashSet<Specifier> {
    self
      .get_instances(instances_by_id)
      .iter()
      .fold(HashSet::new(), |mut uniques, instance| {
        uniques.insert(instance.actual.clone());
        uniques
      })
  }

  /// Is the exact same specifier used by all instances in this group?
  pub fn all_are_identical(&self, instances_by_id: &InstancesById) -> bool {
    let mut previous: Option<&Specifier> = None;
    for instance in self.get_instances(instances_by_id) {
      if let Some(value) = previous {
        if *value != instance.actual {
          return false;
        }
      }
      previous = Some(&instance.expected);
    }
    return true;
  }

  pub fn get_highest_semver(&self, instances_by_id: &InstancesById) -> Option<Specifier> {
    self.get_highest_or_lowest_semver(instances_by_id, Cmp::Gt)
  }

  pub fn get_lowest_semver(&self, instances_by_id: &InstancesById) -> Option<Specifier> {
    self.get_highest_or_lowest_semver(instances_by_id, Cmp::Lt)
  }

  fn get_highest_or_lowest_semver(
    &self,
    instances_by_id: &InstancesById,
    preferred_order: Cmp,
  ) -> Option<Specifier> {
    self
      .get_instances(instances_by_id)
      .iter()
      .fold(None, |highest, instance| match highest {
        None => Some(&instance.expected),
        Some(highest) => match compare(instance.expected.unwrap(), highest.unwrap()) {
          Ok(actual_order) => {
            if actual_order == preferred_order {
              Some(&instance.expected)
            } else {
              Some(highest)
            }
          }
          Err(_) => {
            panic!("Cannot compare {:?} and {:?}", &instance.expected, &highest);
          }
        },
      })
      .map(|specifier| specifier.clone())
  }

  pub fn get_same_range_mismatches<'a>(
    &'a self,
    instances_by_id: &'a InstancesById,
  ) -> Option<Vec<(InstancesBySpecifier, InstancesBySpecifier)>> {
    let mut mismatches: Vec<(InstancesBySpecifier, InstancesBySpecifier)> = vec![];
    let by_specifier = self.group_by_specifier(&instances_by_id);
    by_specifier.iter().for_each(|a| {
      let (specifier_a, instances_a) = a;
      let range_a = specifier_a.unwrap().parse::<Range>().unwrap();
      by_specifier.iter().for_each(|b| {
        let (specifier_b, instances_b) = b;
        if specifier_a == specifier_b {
          return;
        }
        let range_b = specifier_b.unwrap().parse::<Range>().unwrap();
        if range_a.allows_all(&range_b) {
          return;
        }
        mismatches.push((
          InstancesBySpecifier {
            specifier: specifier_a.clone(),
            instances: instances_a.clone(),
          },
          InstancesBySpecifier {
            specifier: specifier_b.clone(),
            instances: instances_b.clone(),
          },
        ));
      })
    });
    Some(mismatches).filter(|vec| !vec.is_empty())
  }

  pub fn get_same_range_mismatches2<'a>(
    &'a self,
    instances_by_id: &'a InstancesById,
  ) -> HashMap<Specifier, Vec<Specifier>> {
    let mut mismatches_by_specifier: HashMap<Specifier, Vec<Specifier>> = HashMap::new();
    let unique_specifiers = self.get_unique_specifiers(&instances_by_id);
    let get_range = |specifier: &Specifier| specifier.unwrap().parse::<Range>().unwrap();
    unique_specifiers.iter().for_each(|specifier_a| {
      let range_a = get_range(specifier_a);
      unique_specifiers.iter().for_each(|specifier_b| {
        if specifier_a == specifier_b {
          return;
        }
        let range_b = get_range(specifier_b);
        if range_a.allows_all(&range_b) {
          return;
        }
        mismatches_by_specifier
          .entry(specifier_a.clone())
          .or_insert(vec![])
          .push(specifier_b.clone());
      });
    });
    mismatches_by_specifier
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
      .iter()
      .fold(HashMap::new(), |mut acc, instance| {
        acc
          .entry(instance.expected.clone())
          .or_insert_with(|| vec![])
          .push(&instance);
        acc
      })
  }
}
