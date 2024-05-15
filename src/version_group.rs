use std::collections::BTreeMap;
use std::vec;

use node_semver::Range;
use serde::Deserialize;
use version_compare::{compare, Cmp};

use crate::config;
use crate::effects::{Effects, InstanceEvent};
use crate::group_selector::GroupSelector;
use crate::instance::Instance;
use crate::instance_group::{InstanceGroup, InstanceIdsBySpecifier, InstancesById};
use crate::package_json::Packages;
use crate::semver_group::SemverGroup;

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
  pub instance_groups_by_name: BTreeMap<String, InstanceGroup>,
  /// Which version to use when variant is `Standard`
  pub prefer_version: Option<PreferVersion>,
  /// The version to pin all instances to when variant is `Pinned`
  pub pin_version: Option<String>,
  /// `name` properties of package.json files developed in the monorepo when variant is `SnappedTo`
  pub snap_to: Option<Vec<String>>,
}

impl VersionGroup {
  /// Add an instance to this version group if it is eligible, and return
  /// whether it was added.
  pub fn add_instance(&mut self, instance: &Instance, semver_group: Option<&SemverGroup>) {
    // Ensure that a group exists for this dependency name.
    if !self.instance_groups_by_name.contains_key(&instance.name) {
      self.instance_groups_by_name.insert(
        instance.name.clone(),
        InstanceGroup::new(instance.name.clone()),
      );
    }

    // Get the group for this dependency name.
    let instance_group = self
      .instance_groups_by_name
      .get_mut(&instance.name)
      .unwrap();

    // Track/count instances
    instance_group.all.push(instance.id.clone());

    // Track/count unique version specifiers and which instances use them
    // 1. Ensure that a group exists for this specifier.
    if !instance_group
      .by_specifier
      .contains_key(&instance.specifier)
    {
      instance_group
        .by_specifier
        .insert(instance.specifier.clone(), vec![]);
    }

    // 2. Add this instance against its specifier
    instance_group
      .by_specifier
      .get_mut(&instance.specifier)
      .unwrap()
      .push(instance.id.clone());

    // Track/count what specifier types we have encountered
    if instance.specifier_type.is_semver() {
      instance_group.semver.push(instance.id.clone());
    } else {
      instance_group.non_semver.push(instance.id.clone());
    }

    if matches!(self.variant, VersionGroupVariant::Pinned) {
      instance_group.expected_version = self.pin_version.clone();
      return;
    }

    if matches!(self.variant, VersionGroupVariant::Standard) {
      // If this is the original source of a locally-developed package, keep a
      // reference to it and set it as the preferred version
      if instance.dependency_type.name == "local" {
        instance_group.local = Some(instance.id.clone());
        instance_group.expected_version = Some(instance.specifier.clone());
      }

      // A locally-developed package version overrides every other, so if one
      // has not been found, we need to look at the usages of it for a preferred
      // version
      if instance_group.local.is_none() {
        if instance.specifier_type.is_semver() && instance_group.non_semver.len() == 0 {
          // Have we set a preferred version yet for these instances?
          match &mut instance_group.expected_version {
            // No, this is the first candidate.
            None => {
              instance_group.expected_version = Some(instance.specifier.clone());
            }
            // Yes, compare this candidate with the previous one
            Some(expected_version) => {
              let this_version = &instance.specifier;
              let prefer_lowest = matches!(&self.prefer_version, Some(PreferVersion::LowestSemver));
              let preferred_order = if prefer_lowest { Cmp::Lt } else { Cmp::Gt };
              match compare(this_version, &expected_version) {
                Ok(actual_order) => {
                  if preferred_order == actual_order {
                    instance_group.expected_version = Some(instance.specifier.clone());
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
          instance_group.expected_version = None;
        }
      }
    }
  }

  /// Create every version group defined in the rcfile.
  pub fn from_rcfile(
    rcfile: &config::Rcfile,
    local_package_names: &Vec<String>,
  ) -> Vec<VersionGroup> {
    let mut user_groups: Vec<VersionGroup> = rcfile
      .version_groups
      .iter()
      .map(|group| VersionGroup::from_config(group, local_package_names))
      .collect();
    let catch_all_group = VersionGroup {
      variant: VersionGroupVariant::Standard,
      selector: GroupSelector::new(
        /*include_dependencies:*/ vec![],
        /*include_dependency_types:*/ vec![],
        /*label:*/ "Default Version Group".to_string(),
        /*include_packages:*/ vec![],
        /*include_specifier_types:*/ vec![],
      ),
      instance_groups_by_name: BTreeMap::new(),
      prefer_version: Some(PreferVersion::HighestSemver),
      pin_version: None,
      snap_to: None,
    };
    user_groups.push(catch_all_group);
    user_groups
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
        instance_groups_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(true) = group.is_ignored {
      return VersionGroup {
        variant: VersionGroupVariant::Ignored,
        selector,
        instance_groups_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(pin_version) = &group.pin_version {
      return VersionGroup {
        variant: VersionGroupVariant::Pinned,
        selector,
        instance_groups_by_name: BTreeMap::new(),
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
          instance_groups_by_name: BTreeMap::new(),
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
        instance_groups_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: Some(snap_to.clone()),
      };
    }
    if let Some(prefer_version) = &group.prefer_version {
      return VersionGroup {
        variant: VersionGroupVariant::Standard,
        selector,
        instance_groups_by_name: BTreeMap::new(),
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
      instance_groups_by_name: BTreeMap::new(),
      prefer_version: Some(PreferVersion::HighestSemver),
      pin_version: None,
      snap_to: None,
    }
  }

  pub fn visit(
    &self,
    // needed by same range groups, every instance in the project
    instances_by_id: &mut InstancesById,
    // chosen strategy to lint, fix, use different log output, etc
    effects: &impl Effects,
    // when fixing, we write to the package.json files
    packages: &mut Packages,
  ) -> bool {
    // @TODO: return a Vec of Result<GoodEnum, BadEnum>?
    let mut lint_is_valid = true;
    match self.variant {
      VersionGroupVariant::Ignored => {
        effects.on_group(&self.selector);
        self
          .instance_groups_by_name
          .values()
          .for_each(|instance_group| {
            effects.on_ignored_instance_group(instance_group);
          });
      }
      VersionGroupVariant::Banned => {
        effects.on_group(&self.selector);
        self
          .instance_groups_by_name
          .values()
          .for_each(|instance_group| {
            effects.on_banned_instance_group(instance_group);
            instance_group
              .by_specifier
              .iter()
              .for_each(|instances_by_specifier| {
                lint_is_valid = false;
                effects.on_banned_instance(&mut InstanceEvent {
                  instances_by_id,
                  instance_group,
                  // @TODO: use None
                  mismatches_with: ("".to_string(), vec![]),
                  packages,
                  target: (
                    instances_by_specifier.0.clone(),
                    instances_by_specifier.1.clone(),
                  ),
                });
              });
          });
      }
      VersionGroupVariant::Pinned => {
        effects.on_group(&self.selector);
        self
          .instance_groups_by_name
          .values()
          .for_each(|instance_group| {
            if !instance_group.has_identical_specifiers() {
              effects.on_invalid_pinned_instance_group(instance_group);
              let pinned_version = instance_group.expected_version.clone().unwrap();
              instance_group
                .by_specifier
                .iter()
                .for_each(|instances_by_specifier| {
                  if instance_group.is_mismatch(&instances_by_specifier.0) {
                    lint_is_valid = false;
                    effects.on_pinned_version_mismatch(&mut InstanceEvent {
                      instances_by_id,
                      instance_group,
                      mismatches_with: (pinned_version.clone(), vec![]),
                      packages,
                      target: (
                        instances_by_specifier.0.clone(),
                        instances_by_specifier.1.clone(),
                      ),
                    });
                  }
                });
            } else {
              effects.on_valid_pinned_instance_group(instance_group);
            };
          });
      }
      VersionGroupVariant::SameRange => {
        effects.on_group(&self.selector);
        self
          .instance_groups_by_name
          .values()
          .for_each(|instance_group| {
            let mut mismatches: Vec<(InstanceIdsBySpecifier, InstanceIdsBySpecifier)> = vec![];
            instance_group.by_specifier.iter().for_each(|a| {
              let range_a = a.0.parse::<Range>().unwrap();
              instance_group.by_specifier.iter().for_each(|b| {
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
              effects.on_valid_same_range_instance_group(instance_group);
            } else {
              lint_is_valid = false;
              effects.on_invalid_same_range_instance_group(instance_group);
              mismatches
                .into_iter()
                .for_each(|(target_instance_id, mismatches_with)| {
                  effects.on_same_range_mismatch(&mut InstanceEvent {
                    instances_by_id,
                    instance_group,
                    mismatches_with,
                    packages,
                    target: target_instance_id,
                  });
                });
            }
          });
      }
      VersionGroupVariant::SnappedTo => {
        effects.on_group(&self.selector);
        if let Some(snap_to) = &self.snap_to {
          self
            .instance_groups_by_name
            .values()
            .for_each(|instance_group| {
              let mismatches = get_snap_to_mismatches(snap_to, instances_by_id, instance_group);
              if mismatches.len() == 0 {
                effects.on_valid_snap_to_instance_group(instance_group);
              } else {
                lint_is_valid = false;
                effects.on_invalid_snap_to_instance_group(instance_group);
                mismatches
                  .into_iter()
                  .for_each(|(target_instance_id, mismatches_with)| {
                    effects.on_snap_to_mismatch(&mut InstanceEvent {
                      instances_by_id,
                      instance_group,
                      mismatches_with,
                      packages,
                      target: target_instance_id,
                    });
                  });
              }
            });
        }
      }
      VersionGroupVariant::Standard => {
        effects.on_group(&self.selector);
        self
          .instance_groups_by_name
          .values()
          .for_each(|instance_group| {
            if !instance_group.has_identical_specifiers() {
              effects.on_invalid_standard_instance_group(instance_group);
              instance_group
                .by_specifier
                .iter()
                .for_each(|target_instances| {
                  if instance_group.is_mismatch(&target_instances.0) {
                    lint_is_valid = false;
                    if let Some(local_id) = instance_group.local.clone() {
                      let local = instances_by_id.get(&local_id);
                      let specifier = local.unwrap().specifier.clone();
                      effects.on_local_version_mismatch(&mut InstanceEvent {
                        instances_by_id,
                        instance_group,
                        mismatches_with: (specifier, vec![local_id]),
                        packages,
                        target: (target_instances.0.clone(), target_instances.1.clone()),
                      });
                    } else if instance_group.non_semver.len() > 0 {
                      effects.on_unsupported_mismatch(&mut InstanceEvent {
                        instances_by_id,
                        instance_group,
                        mismatches_with: ("".to_string(), vec![]),
                        packages,
                        target: (target_instances.0.clone(), target_instances.1.clone()),
                      });
                    } else if let Some(PreferVersion::LowestSemver) = self.prefer_version {
                      if let Some(expected) = instance_group.expected_version.clone() {
                        if let Some(instances_with_expected) =
                          instance_group.by_specifier.get(&expected)
                        {
                          effects.on_lowest_version_mismatch(&mut InstanceEvent {
                            instances_by_id,
                            instance_group,
                            mismatches_with: (
                              expected.clone(),
                              instances_with_expected.to_owned().clone(),
                            ),
                            packages,
                            target: (target_instances.0.clone(), target_instances.1.clone()),
                          });
                        }
                      }
                    } else {
                      if let Some(expected) = instance_group.expected_version.clone() {
                        if let Some(instances_with_expected) =
                          instance_group.by_specifier.get(&expected)
                        {
                          effects.on_highest_version_mismatch(&mut InstanceEvent {
                            instances_by_id,
                            instance_group: &instance_group,
                            mismatches_with: (
                              expected.clone(),
                              instances_with_expected.to_owned().clone(),
                            ),
                            packages,
                            target: (target_instances.0.clone(), target_instances.1.clone()),
                          });
                        }
                      }
                    }
                  }
                });
            } else {
              effects.on_valid_standard_instance_group(instance_group);
            };
          });
      }
    };
    lint_is_valid
  }
}

/// Return the first instance from the packages which should be snapped to for a
/// given dependency.
fn get_snap_to_instance<'a>(
  snap_to: &Vec<String>,
  dependency_name: &String,
  instances_by_id: &'a mut InstancesById,
) -> Option<&'a Instance> {
  for (id, instance) in instances_by_id {
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
  instance_group: &InstanceGroup,
) -> Vec<(InstanceIdsBySpecifier, InstanceIdsBySpecifier)> {
  let mut mismatches: Vec<(InstanceIdsBySpecifier, InstanceIdsBySpecifier)> = vec![];
  let dependency_name = &instance_group.name;
  if let Some(snappable_instance) = get_snap_to_instance(snap_to, dependency_name, instances_by_id)
  {
    let expected = &snappable_instance.specifier;
    instance_group
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
