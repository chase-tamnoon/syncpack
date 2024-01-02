use serde::Deserialize;

use crate::group_selector;

#[derive(Debug)]
pub struct DisabledSemverGroup {
  pub selector: group_selector::GroupSelector,
  pub is_disabled: bool,
}

#[derive(Debug)]
pub struct IgnoredSemverGroup {
  pub selector: group_selector::GroupSelector,
  pub is_ignored: bool,
}

#[derive(Debug)]
pub struct WithRangeSemverGroup {
  pub selector: group_selector::GroupSelector,
  pub range: String,
}

#[derive(Debug)]
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

impl AnySemverGroup {
  pub fn create(&self) -> SemverGroup {
    let selector = group_selector::GroupSelector {
      dependencies: self.dependencies.clone(),
      dependency_types: self.dependency_types.clone(),
      label: self.label.clone(),
      packages: self.packages.clone(),
      specifier_types: self.specifier_types.clone(),
    };

    if let Some(is_disabled) = self.is_disabled {
      SemverGroup::Disabled(DisabledSemverGroup {
        selector,
        is_disabled,
      })
    } else if let Some(is_ignored) = self.is_ignored {
      SemverGroup::Ignored(IgnoredSemverGroup {
        selector,
        is_ignored,
      })
    } else if let Some(range) = &self.range {
      SemverGroup::WithRange(WithRangeSemverGroup {
        selector,
        range: range.clone(),
      })
    } else {
      panic!("Invalid semver group");
    }
  }
}
