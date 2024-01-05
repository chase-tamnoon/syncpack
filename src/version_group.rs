use std::collections::HashMap;
use std::collections::HashSet;
use std::vec;

use serde::Deserialize;

use crate::config;
use crate::group_selector::GroupSelector;
use crate::instance::Instance;

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
  /// The highest or lowest version to use if all are valid, or the local
  /// version if this is a package developed in this repo.
  pub preferred_version: Option<String>,
  /// Raw version specifiers for each dependency.
  pub unique_specifiers: HashSet<String>,
}

impl<'a> InstanceGroup<'a> {
  pub fn new() -> InstanceGroup<'a> {
    InstanceGroup {
      all: vec![],
      local: None,
      preferred_version: None,
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
        if group.selector.can_add(instance) == false {
          return false;
        }
        if !group.instances_by_name.contains_key(&instance.name) {
          group
            .instances_by_name
            .insert(instance.name.clone(), InstanceGroup::new());
        }

        let instances = group.instances_by_name.get_mut(&instance.name).unwrap();
        instances.all.push(instance);
        instances
          .unique_specifiers
          .insert(instance.specifier.clone());

        match &instances.preferred_version {
          Some(version) => {
            print!("{} ", version);
            if group.prefer_version == "lowestSemver" {
              // @TODO: if this version is lower, set it as the preferred version
            } else {
              // @TODO: if this version is higher, set it as the preferred version
            }
          }
          None => {}
        }

        if instance.dependency_type.name == "local" {
          instances.local = Some(instance);
        }

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
