use log::debug;
use serde::Deserialize;

use crate::config;
use crate::group_selector;
use crate::group_selector::GroupSelector;
use crate::instance::Instance;

#[derive(Debug)]
pub struct DisabledSemverGroup<'a> {
  pub selector: group_selector::GroupSelector,
  pub instances: Vec<&'a Instance<'a>>,
  pub is_disabled: bool,
}

#[derive(Debug)]
pub struct IgnoredSemverGroup<'a> {
  pub selector: group_selector::GroupSelector,
  pub instances: Vec<&'a Instance<'a>>,
  pub is_ignored: bool,
}

#[derive(Debug)]
pub struct WithRangeSemverGroup<'a> {
  pub selector: group_selector::GroupSelector,
  pub instances: Vec<&'a Instance<'a>>,
  pub range: String,
}

#[derive(Debug)]
pub enum SemverGroup<'a> {
  Disabled(DisabledSemverGroup<'a>),
  Ignored(IgnoredSemverGroup<'a>),
  WithRange(WithRangeSemverGroup<'a>),
}

impl<'a> SemverGroup<'a> {
  /// Add an instance to this version group if it is eligible, and return
  /// whether it was added.
  pub fn add_instance(&self, instance: &'a mut Instance) -> bool {
    match self {
      SemverGroup::Disabled(group) => {
        return false;
      }
      SemverGroup::Ignored(group) => {
        return false;
      }
      SemverGroup::WithRange(group) => {
        // If this instance is not eligible for this group, reject it so it can
        // continue to compare itself against the next group.
        if !group.selector.can_add(instance) {
          return false;
        }

        // group.instances.push(instance);
        instance.expected_range = Some(group.range.clone());

        true
      }
    }
  }

  /// When valid, give the value back.
  /// When invalid, return an error with a reason.
  pub fn get_fixed(&self, specifier: &String) -> Result<String, ()> {
    debug!("@TODO: implement SemverGroup::get_fixed");
    Ok(specifier.clone())
  }

  /// Create every version group defined in the rcfile.
  pub fn from_rcfile(rcfile: &config::Rcfile) -> Vec<SemverGroup> {
    let mut user_groups: Vec<SemverGroup> = rcfile
      .semver_groups
      .iter()
      .map(|group| SemverGroup::from_config(group))
      .collect();
    let catch_all_group = SemverGroup::WithRange(WithRangeSemverGroup {
      selector: GroupSelector {
        dependencies: vec![],
        dependency_types: vec![],
        label: "default".to_string(),
        packages: vec![],
        specifier_types: vec![],
      },
      instances: vec![],
      range: "".to_string(),
    });
    user_groups.push(catch_all_group);
    user_groups
  }

  /// Create a single version group from a config item from the rcfile.
  pub fn from_config(group: &AnySemverGroup) -> SemverGroup {
    let selector = group_selector::GroupSelector {
      dependencies: group.dependencies.clone(),
      dependency_types: group.dependency_types.clone(),
      label: group.label.clone(),
      packages: group.packages.clone(),
      specifier_types: group.specifier_types.clone(),
    };

    if let Some(true) = group.is_disabled {
      SemverGroup::Disabled(DisabledSemverGroup {
        selector,
        instances: vec![],
        is_disabled: true,
      })
    } else if let Some(true) = group.is_ignored {
      SemverGroup::Ignored(IgnoredSemverGroup {
        selector,
        instances: vec![],
        is_ignored: true,
      })
    } else if let Some(range) = &group.range {
      SemverGroup::WithRange(WithRangeSemverGroup {
        selector,
        instances: vec![],
        range: range.clone(),
      })
    } else {
      panic!("Invalid semver group");
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnySemverGroup {
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
  pub is_disabled: Option<bool>,
  pub is_ignored: Option<bool>,
  pub range: Option<String>,
}
