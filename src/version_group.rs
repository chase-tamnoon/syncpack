use std::collections::{BTreeMap, HashSet};
use std::vec;

use node_semver::Range;
use serde::Deserialize;
use version_compare::{compare, Cmp};

use crate::config;
use crate::effects::Effects;
use crate::group_selector::GroupSelector;
use crate::instance::Instance;
use crate::instance_group::InstanceGroup;
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
pub struct VersionGroup<'a> {
  /// What behaviour has this group been configured to exhibit?
  pub variant: VersionGroupVariant,
  /// Data to determine which instances should be added to this group
  pub selector: GroupSelector,
  /// Group instances of each dependency together for comparison.
  pub instance_groups_by_name: BTreeMap<String, InstanceGroup<'a>>,
  /// Which version to use when variant is `Standard`
  pub prefer_version: Option<PreferVersion>,
  /// The version to pin all instances to when variant is `Pinned`
  pub pin_version: Option<String>,
  /// `name` properties of package.json files developed in the monorepo when variant is `SnappedTo`
  pub snap_to: Option<Vec<String>>,
}

impl<'a> VersionGroup<'a> {
  /// Add an instance to this version group if it is eligible, and return
  /// whether it was added.
  pub fn add_instance(&mut self, instance: &'a Instance, semver_group: Option<&'a SemverGroup>) {
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
    instance_group.all.push(instance);

    // Track/count unique version specifiers
    instance_group
      .unique_specifiers
      .insert(instance.specifier.clone());

    // Track/count what specifier types we have encountered
    if instance.specifier_type.is_semver() {
      instance_group.semver.push(instance);
    } else {
      instance_group.non_semver.push(instance);
    }

    if matches!(self.variant, VersionGroupVariant::Pinned) {
      instance_group.expected_version = self.pin_version.clone();
      return;
    }

    if matches!(self.variant, VersionGroupVariant::Standard) {
      // If this is the original source of a locally-developed package, keep a
      // reference to it and set it as the preferred version
      if instance.dependency_type.name == "local" {
        instance_group.local = Some(instance);
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
  ) -> Vec<VersionGroup<'a>> {
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
  pub fn from_config(
    group: &AnyVersionGroup,
    local_package_names: &Vec<String>,
  ) -> VersionGroup<'a> {
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

  pub fn visit(&self, all_instances: &Vec<Instance>, effects: &impl Effects) -> bool {
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
            instance_group.unique_specifiers.iter().for_each(|actual| {
              lint_is_valid = false;
              effects.on_banned_instance(actual, instance_group);
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
              instance_group
                .unique_specifiers
                .iter()
                .for_each(|actual_specifier| {
                  if instance_group.is_mismatch(actual_specifier) {
                    lint_is_valid = false;
                    effects.on_pinned_version_mismatch(actual_specifier, instance_group);
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
            // @TODO: use (Instance, Instance)
            let mut mismatches: HashSet<(String, String)> = HashSet::new();
            instance_group.unique_specifiers.iter().for_each(|a| {
              let range_a = a.parse::<Range>().unwrap();
              instance_group.unique_specifiers.iter().for_each(|b| {
                if a == b {
                  return;
                }
                let range_b = b.parse::<Range>().unwrap();
                if range_a.allows_all(&range_b) {
                  return;
                }
                mismatches.insert((a.clone(), b.clone()));
              })
            });
            if mismatches.len() == 0 {
              effects.on_valid_same_range_instance_group(instance_group);
            } else {
              lint_is_valid = false;
              effects.on_invalid_same_range_instance_group(instance_group);
              mismatches.iter().for_each(|mismatching_ranges| {
                effects.on_same_range_mismatch(&mismatching_ranges, instance_group);
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
              let mismatches = get_snap_to_mismatches(snap_to, all_instances, instance_group);
              if mismatches.len() == 0 {
                effects.on_valid_snap_to_instance_group(instance_group);
              } else {
                lint_is_valid = false;
                effects.on_invalid_snap_to_instance_group(instance_group);
                mismatches.iter().for_each(|mismatching_versions| {
                  effects.on_snap_to_mismatch(&mismatching_versions, instance_group);
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
              instance_group.unique_specifiers.iter().for_each(|actual| {
                if instance_group.is_mismatch(actual) {
                  lint_is_valid = false;
                  if instance_group.local.is_some() {
                    effects.on_local_version_mismatch(instance_group, actual);
                  } else if instance_group.non_semver.len() > 0 {
                    effects.on_unsupported_mismatch(actual, instance_group);
                  } else if let Some(PreferVersion::LowestSemver) = self.prefer_version {
                    effects.on_lowest_version_mismatch(actual, instance_group);
                  } else {
                    effects.on_highest_version_mismatch(actual, instance_group);
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

fn get_snap_to_mismatches(
  snap_to: &Vec<String>,
  all_instances: &Vec<Instance>,
  instance_group: &InstanceGroup,
) -> HashSet<(String, String)> {
  // @TODO: use (Instance, Instance)
  let mut mismatches: HashSet<(String, String)> = HashSet::new();
  snap_to.iter().any(|snapped_to_package_name| {
    let maybe_snappable_instance = &all_instances.iter().find(|instance| {
      instance.name == instance_group.name
        && instance.package_json.get_name() == *snapped_to_package_name
    });
    match maybe_snappable_instance {
      Some(snappable_instance) => {
        let expected = &snappable_instance.specifier;
        instance_group
          .unique_specifiers
          .iter()
          .filter(|actual| *actual != expected)
          .for_each(|actual| {
            mismatches.insert((actual.clone(), expected.clone()));
          });
        // stop searching
        true
      }
      None => {
        // keep searching
        false
      }
    }
  });
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
