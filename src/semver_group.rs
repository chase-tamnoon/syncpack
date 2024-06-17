use serde::Deserialize;

use crate::{group_selector::GroupSelector, instance::Instance, semver_range::SemverRange};

#[derive(Debug)]
pub enum SemverGroupVariant {
  Disabled,
  Ignored,
  WithRange,
}

#[derive(Debug)]
pub struct SemverGroup {
  /// What behaviour has this group been configured to exhibit?
  pub variant: SemverGroupVariant,
  /// Data to determine which instances should be added to this group
  pub selector: GroupSelector,
  /// The Semver Range which all instances in this group should use
  pub range: Option<SemverRange>,
}

impl SemverGroup {
  /// Create a default/catch-all group which would apply to any instance
  pub fn get_catch_all() -> SemverGroup {
    SemverGroup {
      variant: SemverGroupVariant::Disabled,
      selector: GroupSelector::new(
        /*include_dependencies:*/ vec![],
        /*include_dependency_types:*/ vec![],
        /*label:*/ "Default Semver Group".to_string(),
        /*include_packages:*/ vec![],
        /*include_specifier_types:*/ vec![],
      ),
      range: None,
    }
  }

  /// Add an instance to this version group
  pub fn add_instance(&mut self, instance: &mut Instance) {
    if matches!(self.variant, SemverGroupVariant::WithRange) {
      instance.prefer_range = self.range.clone();
      // apply the expected semver range. This updated value will be read
      // further down the chain when it is visited by version groups
      if instance.specifier.is_semver() {
        let range = self.range.as_ref().unwrap();
        instance.specifier = instance.specifier.with_semver_range(range);
      }
    }
  }

  /// Create a single version group from a config item from the rcfile.
  pub fn from_config(group: &AnySemverGroup) -> SemverGroup {
    let selector = GroupSelector::new(
      /*include_dependencies:*/ group.dependencies.clone(),
      /*include_dependency_types:*/ group.dependency_types.clone(),
      /*label:*/ group.label.clone(),
      /*include_packages:*/ group.packages.clone(),
      /*include_specifier_types:*/ group.specifier_types.clone(),
    );

    if let Some(true) = group.is_disabled {
      SemverGroup {
        variant: SemverGroupVariant::Disabled,
        selector,
        range: None,
      }
    } else if let Some(true) = group.is_ignored {
      SemverGroup {
        variant: SemverGroupVariant::Ignored,
        selector,
        range: None,
      }
    } else if let Some(range) = &group.range {
      SemverGroup {
        variant: SemverGroupVariant::WithRange,
        selector,
        range: SemverRange::new(range),
      }
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
