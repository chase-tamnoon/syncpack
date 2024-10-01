use node_semver::Range;
use std::{
  cell::RefCell,
  cmp::Ordering,
  collections::{HashMap, HashSet},
  rc::Rc,
  vec,
};

use crate::{
  instance::{Instance, InstanceId},
  package_json::PackageJson,
  specifier::{orderable::IsOrderable, Specifier},
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

#[derive(Debug)]
pub struct Dependency {
  /// Every instance of this dependency in this version group.
  pub all_instances: RefCell<Vec<Rc<Instance>>>,
  /// If this dependency is a local package, this is the local instance.
  pub local_instance: RefCell<Option<Rc<Instance>>>,
  /// The name of the dependency
  pub name: String,
  /// The version to pin all instances to when variant is `Pinned`
  pub pinned_specifier: Option<Specifier>,
  /// package.json files developed in the monorepo when variant is `SnappedTo`
  pub snapped_to_packages: Option<Vec<Rc<RefCell<PackageJson>>>>,
  /// What behaviour has this group been configured to exhibit?
  pub variant: Variant,
}

impl Dependency {
  pub fn new(
    name: String,
    variant: Variant,
    pinned_specifier: Option<Specifier>,
    snapped_to_packages: Option<Vec<Rc<RefCell<PackageJson>>>>,
  ) -> Dependency {
    Dependency {
      all_instances: RefCell::new(vec![]),
      local_instance: RefCell::new(None),
      name,
      pinned_specifier,
      snapped_to_packages,
      variant,
    }
  }

  pub fn add_instance(&self, instance: Rc<Instance>) {
    self.all_instances.borrow_mut().push(Rc::clone(&instance));
    if instance.is_local {
      *self.local_instance.borrow_mut() = Some(Rc::clone(&instance));
    }
  }

  pub fn has_local_instance(&self) -> bool {
    self.local_instance.borrow().is_some()
  }

  pub fn has_preferred_ranges(&self) -> bool {
    self
      .all_instances
      .borrow()
      .iter()
      .any(|instance| instance.prefer_range.borrow().is_some())
  }

  pub fn get_local_specifier(&self) -> Option<Specifier> {
    self
      .local_instance
      .borrow()
      .as_ref()
      .map(|instance| instance.actual.clone())
  }

  pub fn all_are_semver(&self) -> bool {
    self
      .all_instances
      .borrow()
      .iter()
      .all(|instance| instance.actual.is_simple_semver())
  }

  pub fn get_unique_expected_and_actual_specifiers(&self) -> HashSet<Specifier> {
    self
      .all_instances
      .borrow()
      .iter()
      .fold(HashSet::new(), |mut uniques, instance| {
        uniques.insert(instance.actual.clone());
        uniques.insert(instance.expected.borrow().clone());
        uniques
      })
  }

  pub fn get_unique_expected_specifiers(&self) -> HashSet<Specifier> {
    self
      .all_instances
      .borrow()
      .iter()
      .fold(HashSet::new(), |mut uniques, instance| {
        uniques.insert(instance.expected.borrow().clone());
        uniques
      })
  }

  /// Is the exact same specifier used by all instances in this group?
  pub fn all_are_identical(&self) -> bool {
    let mut previous: Option<Specifier> = None;
    for instance in self.all_instances.borrow().iter() {
      if let Some(value) = previous {
        if *value.unwrap() != instance.actual.unwrap() {
          return false;
        }
      }
      previous = Some(instance.expected.borrow().clone());
    }
    true
  }

  /// Get the highest or lowest semver specifier in this group.
  ///
  /// We compare the expected (not actual) specifier because we're looking for
  /// what we should suggest as the correct specifier once `fix` is applied
  pub fn get_highest_or_lowest_semver(&self, preferred_order: Ordering) -> Option<Specifier> {
    self
      .all_instances
      .borrow()
      .iter()
      .fold(None, |highest, instance| match highest {
        None => Some(instance.expected.borrow().clone()),
        Some(highest) => {
          let a = instance.expected.borrow().get_orderable();
          let b = highest.get_orderable();
          if a.cmp(&b) == preferred_order {
            Some(instance.expected.borrow().clone())
          } else {
            Some(highest)
          }
        }
      })
  }

  /// Get all semver specifiers which have a range that does not match all of
  /// the other semver specifiers
  ///
  /// We compare the both expected and actual specifiers because we need to know
  /// what is valid right now on disk, but also what would be still be valid or
  /// become invalid once a `fix` is applied and semver group ranges have been
  /// applied.
  ///
  /// We should compare the actual and expected specifier of each instance to
  /// determine what to do
  pub fn get_same_range_mismatches(&self) -> HashMap<Specifier, Vec<Specifier>> {
    let get_range = |specifier: &Specifier| specifier.unwrap().parse::<Range>().unwrap();
    let mut mismatches_by_specifier: HashMap<Specifier, Vec<Specifier>> = HashMap::new();
    let unique_semver_specifiers: Vec<Specifier> = self
      .get_unique_expected_and_actual_specifiers()
      .iter()
      .filter(|specifier| specifier.is_simple_semver())
      .cloned()
      .collect();
    unique_semver_specifiers.iter().for_each(|specifier_a| {
      let range_a = get_range(specifier_a);
      unique_semver_specifiers.iter().for_each(|specifier_b| {
        if specifier_a.unwrap() == specifier_b.unwrap() {
          return;
        }
        let range_b = get_range(specifier_b);
        if range_a.allows_all(&range_b) {
          return;
        }
        mismatches_by_specifier
          .entry(specifier_a.clone())
          .or_default()
          .push(specifier_b.clone());
      });
    });
    mismatches_by_specifier
  }

  /// Return the first instance from the packages which should be snapped to for
  /// a given dependency
  ///
  /// We compare the expected (not actual) specifier because we're looking for
  /// what we should suggest as the correct specifier once `fix` is applied
  ///
  /// Even though the actual specifiers on disk might currently match, we should
  /// suggest it match what we the snapped to specifier should be once fixed
  pub fn get_snapped_to_specifier(&self) -> Option<Specifier> {
    if let Some(snapped_to_packages) = &self.snapped_to_packages {
      for instance in self.all_instances.borrow().iter() {
        if instance.name == *self.name {
          for snapped_to_package in snapped_to_packages {
            if instance.package.borrow().get_name_unsafe() == snapped_to_package.borrow().get_name_unsafe() {
              return Some(instance.expected.borrow().clone());
            }
          }
        }
      }
    }
    None
  }
}
