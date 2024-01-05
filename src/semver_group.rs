use std::collections::HashMap;
use serde::Deserialize;

use crate::config;
use crate::group_selector;
use crate::instance::Instance;

#[derive(Debug)]
pub struct DisabledSemverGroup<'a> {
  pub selector: group_selector::GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub is_disabled: bool,
}

#[derive(Debug)]
pub struct IgnoredSemverGroup<'a> {
  pub selector: group_selector::GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub is_ignored: bool,
}

#[derive(Debug)]
pub struct WithRangeSemverGroup<'a> {
  pub selector: group_selector::GroupSelector,
  pub instances_by_name: HashMap<String, &'a Instance<'a>>,
  pub range: String,
}

#[derive(Debug)]
pub enum SemverGroup<'a> {
  Disabled(DisabledSemverGroup<'a>),
  Ignored(IgnoredSemverGroup<'a>),
  WithRange(WithRangeSemverGroup<'a>),
}

impl SemverGroup<'_> {
  pub fn from_rcfile(rcfile: &config::Rcfile) -> Vec<SemverGroup> {
    rcfile
      .semver_groups
      .iter()
      .map(|group| SemverGroup::from_config(group))
      .collect()
  }

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
        instances_by_name: HashMap::new(),
        is_disabled: true,
      })
    } else if let Some(true) = group.is_ignored {
      SemverGroup::Ignored(IgnoredSemverGroup {
        selector,
        instances_by_name: HashMap::new(),
        is_ignored: true,
      })
    } else if let Some(range) = &group.range {
      SemverGroup::WithRange(WithRangeSemverGroup {
        selector,
        instances_by_name: HashMap::new(),
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
