use node_semver::Range;
use serde::Deserialize;
use std::{collections::BTreeMap, vec};
use version_compare::{compare, Cmp};

use crate::{
  dependency::{Dependency, InstanceIdsBySpecifier, InstancesById},
  effects::{Effects, Event, InstanceEvent},
  group_selector::GroupSelector,
  instance::Instance,
  packages::Packages,
  semver_group::SemverGroup,
};

#[derive(Debug)]
pub enum PreferVersion {
  LowestSemver,
  HighestSemver,
}

#[derive(Debug)]
pub enum VersionGroupVariant {
  Banned,
  Ignored,
  Pinned,
  SameRange,
  SnappedTo,
  Standard,
}

#[derive(Debug)]
pub struct VersionGroup {
  /// What behaviour has this group been configured to exhibit?
  pub variant: VersionGroupVariant,
  /// Data to determine which instances should be added to this group
  pub selector: GroupSelector,
  /// Group instances of each dependency together for comparison.
  pub dependencies_by_name: BTreeMap<String, Dependency>,
  /// Which version to use when variant is `Standard`
  pub prefer_version: Option<PreferVersion>,
  /// The version to pin all instances to when variant is `Pinned`
  pub pin_version: Option<String>,
  /// `name` properties of package.json files developed in the monorepo when variant is `SnappedTo`
  pub snap_to: Option<Vec<String>>,
}

impl VersionGroup {
  /// Create a default/catch-all group which would apply to any instance
  pub fn get_catch_all_group() -> VersionGroup {
    VersionGroup {
      variant: VersionGroupVariant::Standard,
      selector: GroupSelector::new(
        /*include_dependencies:*/ vec![],
        /*include_dependency_types:*/ vec![],
        /*label:*/ "Default Version Group".to_string(),
        /*include_packages:*/ vec![],
        /*include_specifier_types:*/ vec![],
      ),
      dependencies_by_name: BTreeMap::new(),
      prefer_version: Some(PreferVersion::HighestSemver),
      pin_version: None,
      snap_to: None,
    }
  }

  /// Add an instance to this version group if it is eligible, and return
  /// whether it was added.
  pub fn add_instance(&mut self, instance: &Instance, semver_group: Option<&SemverGroup>) {
    // Ensure that a group exists for this dependency name.
    if !self.dependencies_by_name.contains_key(&instance.name) {
      self.dependencies_by_name.insert(
        instance.name.clone(),
        Dependency::new(instance.name.clone()),
      );
    }

    // Get the group for this dependency name.
    let dependency = self.dependencies_by_name.get_mut(&instance.name).unwrap();

    // Track/count instances
    dependency.all.push(instance.id.clone());

    // Track/count unique version specifiers and which instances use them
    // 1. Ensure that a group exists for this specifier.
    if !dependency.by_specifier.contains_key(&instance.specifier) {
      dependency
        .by_specifier
        .insert(instance.specifier.clone(), vec![]);
    }

    // 2. Add this instance against its specifier
    dependency
      .by_specifier
      .get_mut(&instance.specifier)
      .unwrap()
      .push(instance.id.clone());

    // Track/count what specifier types we have encountered
    if instance.specifier_type.is_semver() {
      dependency.semver.push(instance.id.clone());
    } else {
      dependency.non_semver.push(instance.id.clone());
    }

    if matches!(self.variant, VersionGroupVariant::Pinned) {
      dependency.expected_version = self.pin_version.clone();
      return;
    }

    if matches!(self.variant, VersionGroupVariant::Standard) {
      // If this is the original source of a locally-developed package, keep a
      // reference to it and set it as the preferred version
      if instance.dependency_type.name == "local" {
        dependency.local = Some(instance.id.clone());
        dependency.expected_version = Some(instance.specifier.clone());
      }

      // A locally-developed package version overrides every other, so if one
      // has not been found, we need to look at the usages of it for a preferred
      // version
      if dependency.local.is_none() {
        if instance.specifier_type.is_semver() && dependency.non_semver.len() == 0 {
          // Have we set a preferred version yet for these instances?
          match &mut dependency.expected_version {
            // No, this is the first candidate.
            None => {
              dependency.expected_version = Some(instance.specifier.clone());
            }
            // Yes, compare this candidate with the previous one
            Some(expected_version) => {
              let this_version = &instance.specifier;
              let prefer_lowest = matches!(&self.prefer_version, Some(PreferVersion::LowestSemver));
              let preferred_order = if prefer_lowest { Cmp::Lt } else { Cmp::Gt };
              match compare(this_version, &expected_version) {
                Ok(actual_order) => {
                  if preferred_order == actual_order {
                    dependency.expected_version = Some(instance.specifier.clone());
                  }
                }
                Err(_) => {
                  panic!("Cannot compare {} and {}", &this_version, &expected_version);
                }
              };
            }
          }
        } else {
          // clear any previous preferred version if we encounter a non-semver
          dependency.expected_version = None;
        }
      }
    }
  }

  /// Create a single version group from a config item from the rcfile.
  pub fn from_config(group: &AnyVersionGroup, local_package_names: &Vec<String>) -> VersionGroup {
    let selector = GroupSelector::new(
      /*include_dependencies:*/
      with_resolved_keywords(&group.dependencies, local_package_names),
      /*include_dependency_types:*/ group.dependency_types.clone(),
      /*label:*/ group.label.clone(),
      /*include_packages:*/ group.packages.clone(),
      /*include_specifier_types:*/ group.specifier_types.clone(),
    );

    if let Some(true) = group.is_banned {
      return VersionGroup {
        variant: VersionGroupVariant::Banned,
        selector,
        dependencies_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(true) = group.is_ignored {
      return VersionGroup {
        variant: VersionGroupVariant::Ignored,
        selector,
        dependencies_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(pin_version) = &group.pin_version {
      return VersionGroup {
        variant: VersionGroupVariant::Pinned,
        selector,
        dependencies_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: Some(pin_version.clone()),
        snap_to: None,
      };
    }
    if let Some(policy) = &group.policy {
      if policy == "sameRange" {
        return VersionGroup {
          variant: VersionGroupVariant::SameRange,
          selector,
          dependencies_by_name: BTreeMap::new(),
          prefer_version: None,
          pin_version: None,
          snap_to: None,
        };
      } else {
        panic!("Unrecognised version group policy: {}", policy);
      }
    }
    if let Some(snap_to) = &group.snap_to {
      return VersionGroup {
        variant: VersionGroupVariant::SnappedTo,
        selector,
        dependencies_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: Some(snap_to.clone()),
      };
    }
    if let Some(prefer_version) = &group.prefer_version {
      return VersionGroup {
        variant: VersionGroupVariant::Standard,
        selector,
        dependencies_by_name: BTreeMap::new(),
        prefer_version: Some(if prefer_version == "lowestSemver" {
          PreferVersion::LowestSemver
        } else {
          PreferVersion::HighestSemver
        }),
        pin_version: None,
        snap_to: None,
      };
    }
    VersionGroup {
      variant: VersionGroupVariant::Standard,
      selector,
      dependencies_by_name: BTreeMap::new(),
      prefer_version: Some(PreferVersion::HighestSemver),
      pin_version: None,
      snap_to: None,
    }
  }

  pub fn visit(
    &self,
    // needed by same range groups, every instance in the project
    instances_by_id: &mut InstancesById,
    // when fixing, we write to the package.json files
    packages: &mut Packages,
    // chosen strategy to lint, fix, use different log output, etc
    effects: &mut impl Effects,
  ) {
    match self.variant {
      VersionGroupVariant::Ignored => {
        effects.on_event(Event::GroupVisited(&self.selector));
        self.dependencies_by_name.values().for_each(|dependency| {
          effects.on_event(Event::DependencyIgnored(dependency));
        });
      }
      VersionGroupVariant::Banned => {
        effects.on_event(Event::GroupVisited(&self.selector));
        self.dependencies_by_name.values().for_each(|dependency| {
          effects.on_event(Event::DependencyBanned(dependency));
          dependency
            .by_specifier
            .iter()
            .for_each(|instances_by_specifier| {
              effects.on_event(Event::InstanceBanned(&mut InstanceEvent {
                instances_by_id,
                dependency,
                // @TODO: use None
                mismatches_with: ("".to_string(), vec![]),
                packages,
                target: (
                  instances_by_specifier.0.clone(),
                  instances_by_specifier.1.clone(),
                ),
              }));
            });
        });
      }
      VersionGroupVariant::Pinned => {
        effects.on_event(Event::GroupVisited(&self.selector));
        self.dependencies_by_name.values().for_each(|dependency| {
          if !dependency.has_identical_specifiers() {
            effects.on_event(Event::DependencyMismatchesPinnedVersion(dependency));
            let pinned_version = dependency.expected_version.clone().unwrap();
            dependency
              .by_specifier
              .iter()
              .for_each(|instances_by_specifier| {
                if dependency.is_mismatch(&instances_by_specifier.0) {
                  effects.on_event(Event::InstanceMismatchesPinnedVersion(&mut InstanceEvent {
                    instances_by_id,
                    dependency,
                    mismatches_with: (pinned_version.clone(), vec![]),
                    packages,
                    target: (
                      instances_by_specifier.0.clone(),
                      instances_by_specifier.1.clone(),
                    ),
                  }));
                }
              });
          } else {
            effects.on_event(Event::DependencyMatchesPinnedVersion(dependency));
          };
        });
      }
      VersionGroupVariant::SameRange => {
        effects.on_event(Event::GroupVisited(&self.selector));
        self.dependencies_by_name.values().for_each(|dependency| {
          let mut mismatches: Vec<(InstanceIdsBySpecifier, InstanceIdsBySpecifier)> = vec![];
          dependency.by_specifier.iter().for_each(|a| {
            let range_a = a.0.parse::<Range>().unwrap();
            dependency.by_specifier.iter().for_each(|b| {
              if a.0 == b.0 {
                return;
              }
              let range_b = b.0.parse::<Range>().unwrap();
              if range_a.allows_all(&range_b) {
                return;
              }
              let target_instances = (a.0.clone(), a.1.clone());
              let mismatches_with = (b.0.clone(), b.1.clone());
              mismatches.push((target_instances, mismatches_with));
            })
          });
          if mismatches.len() == 0 {
            effects.on_event(Event::DependencyMatchesRange(dependency));
          } else {
            effects.on_event(Event::DependencyMismatchesRange(dependency));
            mismatches
              .into_iter()
              .for_each(|(target_instance_id, mismatches_with)| {
                effects.on_event(Event::InstanceMismatchesRange(&mut InstanceEvent {
                  instances_by_id,
                  dependency,
                  mismatches_with,
                  packages,
                  target: target_instance_id,
                }));
              });
          }
        });
      }
      VersionGroupVariant::SnappedTo => {
        effects.on_event(Event::GroupVisited(&self.selector));
        if let Some(snap_to) = &self.snap_to {
          self.dependencies_by_name.values().for_each(|dependency| {
            let mismatches = get_snap_to_mismatches(snap_to, instances_by_id, dependency);
            if mismatches.len() == 0 {
              effects.on_event(Event::DependencyMatchesSnapTo(dependency));
            } else {
              effects.on_event(Event::DependencyMismatchesSnapTo(dependency));
              mismatches
                .into_iter()
                .for_each(|(target_instance_id, mismatches_with)| {
                  effects.on_event(Event::InstanceMismatchesSnapTo(&mut InstanceEvent {
                    instances_by_id,
                    dependency,
                    mismatches_with,
                    packages,
                    target: target_instance_id,
                  }));
                });
            }
          });
        }
      }
      VersionGroupVariant::Standard => {
        effects.on_event(Event::GroupVisited(&self.selector));
        self.dependencies_by_name.values().for_each(|dependency| {
          if !dependency.has_identical_specifiers() {
            effects.on_event(Event::DependencyMismatchesStandard(dependency));
            dependency.by_specifier.iter().for_each(|target_instances| {
              if dependency.is_mismatch(&target_instances.0) {
                if let Some(local_id) = dependency.local.clone() {
                  let local = instances_by_id.get(&local_id);
                  let specifier = local.unwrap().specifier.clone();
                  effects.on_event(Event::InstanceMismatchesLocalVersion(&mut InstanceEvent {
                    instances_by_id,
                    dependency,
                    mismatches_with: (specifier, vec![local_id]),
                    packages,
                    target: (target_instances.0.clone(), target_instances.1.clone()),
                  }));
                } else if dependency.non_semver.len() > 0 {
                  effects.on_event(Event::InstanceUnsupportedMismatch(&mut InstanceEvent {
                    instances_by_id,
                    dependency,
                    mismatches_with: ("".to_string(), vec![]),
                    packages,
                    target: (target_instances.0.clone(), target_instances.1.clone()),
                  }));
                } else if let Some(PreferVersion::LowestSemver) = self.prefer_version {
                  if let Some(expected) = dependency.expected_version.clone() {
                    if let Some(instances_with_expected) = dependency.by_specifier.get(&expected) {
                      effects.on_event(Event::InstanceMismatchesLowestVersion(
                        &mut InstanceEvent {
                          instances_by_id,
                          dependency,
                          mismatches_with: (
                            expected.clone(),
                            instances_with_expected.to_owned().clone(),
                          ),
                          packages,
                          target: (target_instances.0.clone(), target_instances.1.clone()),
                        },
                      ));
                    }
                  }
                } else {
                  if let Some(expected) = dependency.expected_version.clone() {
                    if let Some(instances_with_expected) = dependency.by_specifier.get(&expected) {
                      effects.on_event(Event::InstanceMismatchesHighestVersion(
                        &mut InstanceEvent {
                          instances_by_id,
                          dependency: &dependency,
                          mismatches_with: (
                            expected.clone(),
                            instances_with_expected.to_owned().clone(),
                          ),
                          packages,
                          target: (target_instances.0.clone(), target_instances.1.clone()),
                        },
                      ));
                    }
                  }
                }
              }
            });
          } else {
            effects.on_event(Event::DependencyMatchesStandard(dependency));
          };
        });
      }
    };
  }
}

/// Return the first instance from the packages which should be snapped to for a
/// given dependency.
fn get_snap_to_instance<'a>(
  snap_to: &Vec<String>,
  dependency_name: &String,
  instances_by_id: &'a mut InstancesById,
) -> Option<&'a Instance> {
  for instance in instances_by_id.values() {
    if instance.name == *dependency_name {
      for snapped_to_package_name in snap_to {
        if instance.package_name == *snapped_to_package_name {
          return Some(instance);
        }
      }
    }
  }
  return None;
}

/// Find all instances which have and do not match their corresponding snap_to
/// instance
fn get_snap_to_mismatches(
  snap_to: &Vec<String>,
  instances_by_id: &mut InstancesById,
  dependency: &Dependency,
) -> Vec<(InstanceIdsBySpecifier, InstanceIdsBySpecifier)> {
  let mut mismatches: Vec<(InstanceIdsBySpecifier, InstanceIdsBySpecifier)> = vec![];
  let dependency_name = &dependency.name;
  if let Some(snappable_instance) = get_snap_to_instance(snap_to, dependency_name, instances_by_id)
  {
    let expected = &snappable_instance.specifier;
    dependency
      .by_specifier
      .iter()
      .filter(|(actual, _)| *actual != expected)
      .for_each(|target_instances| {
        let mismatches_with = (expected.clone(), vec![snappable_instance.id.clone()]);
        let target_instances = (target_instances.0.clone(), target_instances.1.clone());
        mismatches.push((target_instances, mismatches_with));
      });
  }
  mismatches
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  pub is_banned: Option<bool>,
  pub is_ignored: Option<bool>,
  pub pin_version: Option<String>,
  pub policy: Option<String>,
  pub snap_to: Option<Vec<String>>,
  pub prefer_version: Option<String>,
}

/// Resolve keywords such as `$LOCAL` and `!$LOCAL` to their actual values.
fn with_resolved_keywords(
  dependency_names: &Vec<String>,
  local_package_names: &Vec<String>,
) -> Vec<String> {
  let mut resolved_dependencies: Vec<String> = vec![];
  for dependency in dependency_names.iter() {
    match dependency.as_str() {
      "$LOCAL" => {
        for package_name in local_package_names.iter() {
          resolved_dependencies.push(package_name.clone());
        }
      }
      "!$LOCAL" => {
        for package_name in local_package_names.iter() {
          resolved_dependencies.push(format!("!{}", package_name));
        }
      }
      _ => {
        resolved_dependencies.push(dependency.clone());
      }
    }
  }
  resolved_dependencies
}
