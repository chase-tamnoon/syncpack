use serde::Deserialize;

use crate::group_selector;

pub struct DisabledSemverGroup {
  pub selector: group_selector::GroupSelector,
  pub is_disabled: bool,
}

pub struct IgnoredSemverGroup {
  pub selector: group_selector::GroupSelector,
  pub is_ignored: bool,
}

pub struct WithRangeSemverGroup {
  pub selector: group_selector::GroupSelector,
  pub range: String,
}

pub enum SemverGroup {
  Disabled(DisabledSemverGroup),
  Ignored(IgnoredSemverGroup),
  WithRange(WithRangeSemverGroup),
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
