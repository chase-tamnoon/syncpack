use std::collections::BTreeMap;
use std::vec;

use serde::Deserialize;
use version_compare::{compare, Cmp};

use crate::config;
use crate::group_selector::GroupSelector;
use crate::instance::Instance;
use crate::instance_group::InstanceGroup;
use crate::semver_group::SemverGroup;
use crate::specifier::SpecifierType;

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
  pub fn add_instance(&mut self, instance: &'a Instance, semver_group: &'a SemverGroup) {
    // Ensure that a group exists for this dependency name.
    if !self.instance_groups_by_name.contains_key(&instance.name) {
      self
        .instance_groups_by_name
        .insert(instance.name.clone(), InstanceGroup::new());
    }

    // Get the group for this dependency name.
    let instance_group = self
      .instance_groups_by_name
      .get_mut(&instance.name)
      .unwrap();

    // Track/count instances
    instance_group.all.push(instance);

    if matches!(self.variant, VersionGroupVariant::Standard) {
      // Track/count unique version specifiers
      instance_group
        .unique_specifiers
        .insert(instance.specifier.clone());

      // Track/count what specifier types we have encountered
      match &instance.specifier_type {
        SpecifierType::NonSemver(specifier_type) => {
          instance_group.non_semver.push(instance);
        }
        SpecifierType::Semver(specifier_type) => {
          instance_group.semver.push(instance);
        }
      }

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
        if let SpecifierType::Semver(specifier_type) = &instance.specifier_type {
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
        }
      }
    }
  }

  /// Create every version group defined in the rcfile.
  pub fn from_rcfile(rcfile: &config::Rcfile) -> Vec<VersionGroup> {
    let mut user_groups: Vec<VersionGroup> = rcfile
      .version_groups
      .iter()
      .map(|group| VersionGroup::from_config(group))
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
  pub fn from_config(group: &AnyVersionGroup) -> VersionGroup {
    let selector = GroupSelector::new(
      /*include_dependencies:*/ group.dependencies.clone(),
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
      return VersionGroup {
        variant: VersionGroupVariant::SameRange,
        selector,
        instance_groups_by_name: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: None,
      };
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
