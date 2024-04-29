use std::collections::BTreeMap;
use std::vec;

use log::{debug, error};
use serde::Deserialize;
use version_compare::{compare, Cmp};

use crate::config;
use crate::group_selector::GroupSelector;
use crate::instance::Instance;
use crate::instance_group::InstanceGroup;
use crate::specifier::SpecifierType;

#[derive(Debug)]
pub struct BannedVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: BTreeMap<String, InstanceGroup<'a>>,
  pub is_banned: bool,
}

#[derive(Debug)]
pub struct IgnoredVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: BTreeMap<String, InstanceGroup<'a>>,
  pub is_ignored: bool,
}

#[derive(Debug)]
pub struct PinnedVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: BTreeMap<String, InstanceGroup<'a>>,
  pub pin_version: String,
}

#[derive(Debug)]
pub struct SameRangeVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: BTreeMap<String, InstanceGroup<'a>>,
  pub policy: String,
}

#[derive(Debug)]
pub struct SnappedToVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: BTreeMap<String, InstanceGroup<'a>>,
  pub snap_to: Vec<String>,
}

#[derive(Debug)]
pub struct StandardVersionGroup<'a> {
  pub selector: GroupSelector,
  /// Group instances of each dependency together for comparison.
  pub instances_by_name: BTreeMap<String, InstanceGroup<'a>>,
  /// As defined in the rcfile: "lowestSemver" or "highestSemver".
  pub prefer_version: String,
}

#[derive(Debug)]
pub enum VersionGroup<'a> {
  Banned(BannedVersionGroup<'a>),
  Ignored(IgnoredVersionGroup<'a>),
  Pinned(PinnedVersionGroup<'a>),
  SameRange(SameRangeVersionGroup<'a>),
  SnappedTo(SnappedToVersionGroup<'a>),
  Standard(StandardVersionGroup<'a>),
}

impl<'a> VersionGroup<'a> {
  /// Add an instance to this version group if it is eligible, and return
  /// whether it was added.
  pub fn add_instance_if_eligible(&mut self, instance: &'a Instance) -> bool {
    match self {
      VersionGroup::Banned(group) => {
        // If this instance is not eligible for this group, reject it so it can
        // continue to compare itself against the next group.
        if !group.selector.can_add(instance) {
          return false;
        }

        // Ensure that a group exists for this dependency name.
        if !group.instances_by_name.contains_key(&instance.name) {
          group
            .instances_by_name
            .insert(instance.name.clone(), InstanceGroup::new());
        }

        // Get the group for this dependency name.
        let instance_group = group.instances_by_name.get_mut(&instance.name).unwrap();

        instance_group.all.push(instance);

        // Claim this instance so it can't be claimed by another group.
        return true;
      }
      VersionGroup::Ignored(group) => {
        // If this instance is not eligible for this group, reject it so it can
        // continue to compare itself against the next group.
        if !group.selector.can_add(instance) {
          return false;
        }

        // Ensure that a group exists for this dependency name.
        if !group.instances_by_name.contains_key(&instance.name) {
          group
            .instances_by_name
            .insert(instance.name.clone(), InstanceGroup::new());
        }

        // Get the group for this dependency name.
        let instance_group = group.instances_by_name.get_mut(&instance.name).unwrap();

        instance_group.all.push(instance);

        // Claim this instance so it can't be claimed by another group.
        return true;
      }
      VersionGroup::Pinned(group) => {
        // If this instance is not eligible for this group, reject it so it can
        // continue to compare itself against the next group.
        if !group.selector.can_add(instance) {
          return false;
        }

        // Ensure that a group exists for this dependency name.
        if !group.instances_by_name.contains_key(&instance.name) {
          group
            .instances_by_name
            .insert(instance.name.clone(), InstanceGroup::new());
        }

        // Get the group for this dependency name.
        let instance_group = group.instances_by_name.get_mut(&instance.name).unwrap();

        instance_group.all.push(instance);

        // Claim this instance so it can't be claimed by another group.
        return true;
      }
      VersionGroup::SameRange(group) => {
        // If this instance is not eligible for this group, reject it so it can
        // continue to compare itself against the next group.
        if !group.selector.can_add(instance) {
          return false;
        }

        // Ensure that a group exists for this dependency name.
        if !group.instances_by_name.contains_key(&instance.name) {
          group
            .instances_by_name
            .insert(instance.name.clone(), InstanceGroup::new());
        }

        // Get the group for this dependency name.
        let instance_group = group.instances_by_name.get_mut(&instance.name).unwrap();

        instance_group.all.push(instance);

        // Claim this instance so it can't be claimed by another group.
        return true;
      }
      VersionGroup::SnappedTo(group) => {
        // If this instance is not eligible for this group, reject it so it can
        // continue to compare itself against the next group.
        if !group.selector.can_add(instance) {
          return false;
        }

        // Ensure that a group exists for this dependency name.
        if !group.instances_by_name.contains_key(&instance.name) {
          group
            .instances_by_name
            .insert(instance.name.clone(), InstanceGroup::new());
        }

        // Get the group for this dependency name.
        let instance_group = group.instances_by_name.get_mut(&instance.name).unwrap();

        instance_group.all.push(instance);

        // Claim this instance so it can't be claimed by another group.
        return true;
      }
      VersionGroup::Standard(group) => {
        // If this instance is not eligible for this group, reject it so it can
        // continue to compare itself against the next group.
        if !group.selector.can_add(instance) {
          return false;
        }

        // Ensure that a group exists for this dependency name.
        if !group.instances_by_name.contains_key(&instance.name) {
          group
            .instances_by_name
            .insert(instance.name.clone(), InstanceGroup::new());
        }

        // Get the group for this dependency name.
        let instance_group = group.instances_by_name.get_mut(&instance.name).unwrap();

        instance_group.all.push(instance);

        // If there is more than one version in this list, then we have
        // mismatching versions.
        instance_group
          .unique_specifiers
          .insert(instance.specifier.clone());

        // If this is a local package
        if instance.dependency_type.name == "local" {
          // keep track of it for use when analysing
          instance_group.local = Some(instance);
          // and set this as the preferred version, since it is the originating
          // package where this dependency is being developed.
          let local_version = &instance.specifier;
          instance_group.preferred_version = Some(local_version.clone());
        }

        match &instance.specifier_type {
          SpecifierType::NonSemver(specifier_type) => {
            instance_group.non_semver.push(instance);
          }
          SpecifierType::Semver(specifier_type) => {
            instance_group.semver.push(instance);
          }
        }

        if instance_group.local.is_none() {
          // If we have a valid semver specifier, it can be a candidate for being
          // suggested as the preferred version.
          if let SpecifierType::Semver(specifier_type) = &instance.specifier_type {
            match &mut instance_group.preferred_version {
              // If there is already a preferred version we should keep whichever
              // is the highest or lowest version depending on the group's
              // preference.
              Some(current_preferred_version) => {
                let this_version = &instance.specifier;
                let prefer_lowest = &group.prefer_version == "lowestSemver";
                let preferred = if prefer_lowest { Cmp::Lt } else { Cmp::Gt };
                let actual = compare(this_version, current_preferred_version);
                let is_preferred = actual == Ok(preferred);
                if is_preferred {
                  set_preferred_version(instance, instance_group, this_version.clone());
                }
              }
              // If there's no preferred version yet, this is the first candidate.
              None => {
                let this_version = &instance.specifier;
                set_preferred_version(instance, instance_group, this_version.clone());
              }
            }
          }
        }

        // Claim this instance so it can't be claimed by another group.
        return true;
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
    let catch_all_group = VersionGroup::Standard(StandardVersionGroup {
      selector: GroupSelector::new(
        /*include_dependencies:*/ vec![],
        /*include_dependency_types:*/ vec![],
        /*label:*/ "Default Version Group".to_string(),
        /*include_packages:*/ vec![],
        /*include_specifier_types:*/ vec![],
      ),
      instances_by_name: BTreeMap::new(),
      prefer_version: "highestSemver".to_string(),
    });
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
      return VersionGroup::Banned(BannedVersionGroup {
        selector,
        instances_by_name: BTreeMap::new(),
        is_banned: true,
      });
    }
    if let Some(true) = group.is_ignored {
      return VersionGroup::Ignored(IgnoredVersionGroup {
        selector,
        instances_by_name: BTreeMap::new(),
        is_ignored: true,
      });
    }
    if let Some(pin_version) = &group.pin_version {
      return VersionGroup::Pinned(PinnedVersionGroup {
        selector,
        instances_by_name: BTreeMap::new(),
        pin_version: pin_version.clone(),
      });
    }
    if let Some(policy) = &group.policy {
      return VersionGroup::SameRange(SameRangeVersionGroup {
        selector,
        instances_by_name: BTreeMap::new(),
        policy: policy.clone(),
      });
    }
    if let Some(snap_to) = &group.snap_to {
      return VersionGroup::SnappedTo(SnappedToVersionGroup {
        selector,
        instances_by_name: BTreeMap::new(),
        snap_to: snap_to.clone(),
      });
    }
    if let Some(prefer_version) = &group.prefer_version {
      return VersionGroup::Standard(StandardVersionGroup {
        selector,
        instances_by_name: BTreeMap::new(),
        prefer_version: prefer_version.clone(),
      });
    }
    VersionGroup::Standard(StandardVersionGroup {
      selector,
      instances_by_name: BTreeMap::new(),
      prefer_version: "highestSemver".to_string(),
    })
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

fn set_preferred_version(
  instance: &Instance,
  instances: &mut InstanceGroup,
  next_preferred_version: String,
) {
  debug!(
    target: "set_preferred_version",
    "{}: {:?} → {} ({:?})",
    &instance.name, &instances.preferred_version, &next_preferred_version, &instance.expected_range
  );

  if let Some(expected_range) = &instance.expected_range {
    // debug!(
    //   "@TODO apply preferred semver range ('{}') to preferred version",
    //   expected_range
    // );
    let with_fixed_semver_range: Result<String, std::io::Error> =
      Ok(next_preferred_version.clone());
    if let Ok(fixed_version) = with_fixed_semver_range {
      // println!("Fixed version to {}", &fixed_version);
      instances.preferred_version = Some(fixed_version);
    } else {
      error!("Failed to get fixed version for {:?}", instance);
    }
  }
}
