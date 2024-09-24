use serde::Deserialize;
use std::{collections::BTreeMap, vec};

use crate::{
  dependency::Dependency, group_selector::GroupSelector, instance::Instance, specifier::Specifier,
};

/// What behaviour has this group been configured to exhibit?
#[derive(Clone, Debug)]
pub enum Variant {
  Banned,
  HighestSemver,
  Ignored,
  LowestSemver,
  Pinned,
  SameRange,
  SnappedTo,
}

#[derive(Debug)]
pub struct VersionGroup {
  /// What behaviour has this group been configured to exhibit?
  pub variant: Variant,
  /// Data to determine which instances should be added to this group
  pub selector: GroupSelector,
  /// Group instances of each dependency together for comparison.
  pub dependencies: BTreeMap<String, Dependency>,
  /// The version to pin all instances to when variant is `Pinned`
  pub pin_version: Option<Specifier>,
  /// `name` properties of package.json files developed in the monorepo when variant is `SnappedTo`
  pub snap_to: Option<Vec<String>>,
}

impl VersionGroup {
  /// Create a default/catch-all group which would apply to any instance
  pub fn get_catch_all() -> VersionGroup {
    VersionGroup {
      variant: Variant::HighestSemver,
      selector: GroupSelector::new(
        /*include_dependencies:*/ vec![],
        /*include_dependency_types:*/ vec![],
        /*label:*/ "Default Version Group".to_string(),
        /*include_packages:*/ vec![],
        /*include_specifier_types:*/ vec![],
      ),
      dependencies: BTreeMap::new(),
      pin_version: None,
      snap_to: None,
    }
  }

  /// Lazily create a dependency if it doesn't already exist
  pub fn get_or_create_dependency(&mut self, instance: &Instance) -> &mut Dependency {
    self
      .dependencies
      .entry(instance.name.clone())
      .or_insert_with(|| {
        Dependency::new(
          /*name:*/ instance.name.clone(),
          /*variant:*/ self.variant.clone(),
          /*pin_version:*/ self.pin_version.clone(),
          /*snap_to:*/ self.snap_to.clone(),
        )
      })
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
        variant: Variant::Banned,
        selector,
        dependencies: BTreeMap::new(),
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(true) = group.is_ignored {
      return VersionGroup {
        variant: Variant::Ignored,
        selector,
        dependencies: BTreeMap::new(),
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(pin_version) = &group.pin_version {
      return VersionGroup {
        variant: Variant::Pinned,
        selector,
        dependencies: BTreeMap::new(),
        pin_version: Some(Specifier::new(pin_version)),
        snap_to: None,
      };
    }
    if let Some(policy) = &group.policy {
      if policy == "sameRange" {
        return VersionGroup {
          variant: Variant::SameRange,
          selector,
          dependencies: BTreeMap::new(),
          pin_version: None,
          snap_to: None,
        };
      } else {
        panic!("Unrecognised version group policy: {}", policy);
      }
    }
    if let Some(snap_to) = &group.snap_to {
      return VersionGroup {
        variant: Variant::SnappedTo,
        selector,
        dependencies: BTreeMap::new(),
        pin_version: None,
        snap_to: Some(snap_to.clone()),
      };
    }
    if let Some(prefer_version) = &group.prefer_version {
      return VersionGroup {
        variant: if prefer_version == "lowestSemver" {
          Variant::LowestSemver
        } else {
          Variant::HighestSemver
        },
        selector,
        dependencies: BTreeMap::new(),
        pin_version: None,
        snap_to: None,
      };
    }
    VersionGroup {
      variant: Variant::HighestSemver,
      selector,
      dependencies: BTreeMap::new(),
      pin_version: None,
      snap_to: None,
    }
  }
}

struct SnapToMismatches {
  pub instance_ids: Vec<String>,
  pub actual_specifier: Specifier,
  pub expected_specifier: Specifier,
  pub snap_to_instance_id: String,
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
