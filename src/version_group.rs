use std::collections::HashMap;
use std::collections::HashSet;
use std::vec;

use serde::Deserialize;
use version_compare::{compare, Cmp};

use crate::config;
use crate::group_selector::GroupSelector;
use crate::instance::Instance;
use crate::specifier::SpecifierType;

#[derive(Debug)]
pub struct BannedVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub is_banned: bool,
}

#[derive(Debug)]
pub struct IgnoredVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub is_ignored: bool,
}

#[derive(Debug)]
pub struct PinnedVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub pin_version: String,
}

#[derive(Debug)]
pub struct SameRangeVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub policy: String,
}

#[derive(Debug)]
pub struct SnappedToVersionGroup<'a> {
  pub selector: GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub snap_to: Vec<String>,
}

#[derive(Debug)]
pub struct StandardVersionGroup<'a> {
  pub selector: GroupSelector,
  /// Group instances of each dependency together for comparison.
  pub instances_by_name: HashMap<String, InstanceGroup<'a>>,
  /// As defined in the rcfile: "lowestSemver" or "highestSemver".
  pub prefer_version: String,
}

#[derive(Debug)]
pub struct InstanceGroup<'a> {
  /// Every instance of this dependency in this version group.
  pub all: Vec<&'a Instance<'a>>,
  /// If this dependency is a local package, this is the local instance.
  pub local: Option<&'a Instance<'a>>,
  /// All instances with `SpecifierType::NonSemver` versions
  pub non_semver: Vec<&'a Instance<'a>>,
  /// The highest or lowest version to use if all are valid, or the local
  /// version if this is a package developed in this repo.
  pub preferred_version: Option<String>,
  /// All instances with `SpecifierType::Semver` versions
  pub semver: Vec<&'a Instance<'a>>,
  /// Raw version specifiers for each dependency.
  pub unique_specifiers: HashSet<String>,
}

impl<'a> InstanceGroup<'a> {
  pub fn new() -> InstanceGroup<'a> {
    InstanceGroup {
      all: vec![],
      local: None,
      non_semver: vec![],
      preferred_version: None,
      semver: vec![],
      unique_specifiers: HashSet::new(),
    }
  }
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
  pub fn add_instance(&mut self, instance: &'a Instance) -> bool {
    match self {
      VersionGroup::Banned(group) => {
        return false;
      }
      VersionGroup::Ignored(group) => {
        return false;
      }
      VersionGroup::Pinned(group) => {
        return false;
      }
      VersionGroup::SameRange(group) => {
        return false;
      }
      VersionGroup::SnappedTo(group) => {
        return false;
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
                let preference = if &group.prefer_version == "lowestSemver" {
                  Cmp::Lt
                } else {
                  Cmp::Gt
                };

                if compare(this_version, current_preferred_version) == Ok(preference) {
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
      selector: GroupSelector {
        dependencies: vec![],
        dependency_types: vec![],
        label: "default".to_string(),
        packages: vec![],
        specifier_types: vec![],
      },
      instances_by_name: HashMap::new(),
      prefer_version: "highestSemver".to_string(),
    });
    user_groups.push(catch_all_group);
    user_groups
  }

  /// Create a single version group from a config item from the rcfile.
  pub fn from_config(group: &AnyVersionGroup) -> VersionGroup {
    let selector = GroupSelector {
      dependencies: group.dependencies.clone(),
      dependency_types: group.dependency_types.clone(),
      label: group.label.clone(),
      packages: group.packages.clone(),
      specifier_types: group.specifier_types.clone(),
    };

    if let Some(true) = group.is_banned {
      return VersionGroup::Banned(BannedVersionGroup {
        selector,
        instances_by_name: HashMap::new(),
        is_banned: true,
      });
    }
    if let Some(true) = group.is_ignored {
      return VersionGroup::Ignored(IgnoredVersionGroup {
        selector,
        instances_by_name: HashMap::new(),
        is_ignored: true,
      });
    }
    if let Some(pin_version) = &group.pin_version {
      return VersionGroup::Pinned(PinnedVersionGroup {
        selector,
        instances_by_name: HashMap::new(),
        pin_version: pin_version.clone(),
      });
    }
    if let Some(policy) = &group.policy {
      return VersionGroup::SameRange(SameRangeVersionGroup {
        selector,
        instances_by_name: HashMap::new(),
        policy: policy.clone(),
      });
    }
    if let Some(snap_to) = &group.snap_to {
      return VersionGroup::SnappedTo(SnappedToVersionGroup {
        selector,
        instances_by_name: HashMap::new(),
        snap_to: snap_to.clone(),
      });
    }
    if let Some(prefer_version) = &group.prefer_version {
      return VersionGroup::Standard(StandardVersionGroup {
        selector,
        instances_by_name: HashMap::new(),
        prefer_version: prefer_version.clone(),
      });
    }
    panic!("Invalid version group");
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
  if let Some(expected_range) = &instance.expected_range {
    println!("@TODO fix semver range");
    let with_fixed_semver_range: Result<String, std::io::Error> =
      Ok(next_preferred_version.clone());
    if let Ok(fixed_version) = with_fixed_semver_range {
      println!("Fixed version to {}", &fixed_version);
      instances.preferred_version = Some(fixed_version);
    } else {
      println!("Failed to get fixed version for {:?}", instance);
    }
  }
}
