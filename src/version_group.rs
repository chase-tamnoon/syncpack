use log::debug;
use serde::Deserialize;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, vec};

use crate::{
  dependency::Dependency, group_selector::GroupSelector, instance::Instance, package_json::PackageJson,
  packages::Packages, specifier::Specifier,
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
  pub dependencies: RefCell<BTreeMap<String, Dependency>>,
  /// The version to pin all instances to when variant is `Pinned`
  pub pin_version: Option<Specifier>,
  /// `name` properties of package.json files developed in the monorepo when variant is `SnappedTo`
  pub snap_to: Option<Vec<Rc<RefCell<PackageJson>>>>,
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
      dependencies: RefCell::new(BTreeMap::new()),
      pin_version: None,
      snap_to: None,
    }
  }

  pub fn add_instance(&self, instance: Rc<Instance>) {
    let mut dependencies = self.dependencies.borrow_mut();
    let dependency = dependencies.entry(instance.name.clone()).or_insert_with(|| {
      Dependency::new(
        /*name:*/ instance.name.clone(),
        /*variant:*/ self.variant.clone(),
        /*pin_version:*/ self.pin_version.clone(),
        /*snap_to:*/ self.snap_to.clone(),
      )
    });
    dependency.add_instance(Rc::clone(&instance));
    std::mem::drop(dependencies);
  }

  /// Create a single version group from a config item from the rcfile.
  pub fn from_config(group: &AnyVersionGroup, packages: &Packages) -> VersionGroup {
    let selector = GroupSelector::new(
      /*include_dependencies:*/
      with_resolved_keywords(&group.dependencies, packages),
      /*include_dependency_types:*/ group.dependency_types.clone(),
      /*label:*/ group.label.clone(),
      /*include_packages:*/ group.packages.clone(),
      /*include_specifier_types:*/ group.specifier_types.clone(),
    );

    if let Some(true) = group.is_banned {
      return VersionGroup {
        variant: Variant::Banned,
        selector,
        dependencies: RefCell::new(BTreeMap::new()),
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(true) = group.is_ignored {
      return VersionGroup {
        variant: Variant::Ignored,
        selector,
        dependencies: RefCell::new(BTreeMap::new()),
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(pin_version) = &group.pin_version {
      return VersionGroup {
        variant: Variant::Pinned,
        selector,
        dependencies: RefCell::new(BTreeMap::new()),
        pin_version: Some(Specifier::new(pin_version)),
        snap_to: None,
      };
    }
    if let Some(policy) = &group.policy {
      if policy == "sameRange" {
        return VersionGroup {
          variant: Variant::SameRange,
          selector,
          dependencies: RefCell::new(BTreeMap::new()),
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
        dependencies: RefCell::new(BTreeMap::new()),
        pin_version: None,
        snap_to: Some(
          snap_to
            .iter()
            .flat_map(|name| {
              packages
                .by_name
                .get(name)
                .inspect(|x| debug!("snapTo package '{name}' not found"))
                .map(Rc::clone)
            })
            .collect(),
        ),
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
        dependencies: RefCell::new(BTreeMap::new()),
        pin_version: None,
        snap_to: None,
      };
    }
    VersionGroup {
      variant: Variant::HighestSemver,
      selector,
      dependencies: RefCell::new(BTreeMap::new()),
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
fn with_resolved_keywords(dependency_names: &[String], packages: &Packages) -> Vec<String> {
  let mut resolved_dependencies: Vec<String> = vec![];
  for dependency_name in dependency_names.iter() {
    match dependency_name.as_str() {
      "$LOCAL" => {
        for package_name in packages.by_name.keys() {
          resolved_dependencies.push(package_name.clone());
        }
      }
      "!$LOCAL" => {
        for package_name in packages.by_name.keys() {
          resolved_dependencies.push(format!("!{}", package_name));
        }
      }
      _ => {
        resolved_dependencies.push(dependency_name.clone());
      }
    }
  }
  resolved_dependencies
}
