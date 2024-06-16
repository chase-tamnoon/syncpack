use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
  config::Config,
  dependency::{Dependency, InstancesById},
  effects::{Effects, Event, MatchEvent, MismatchEvent},
  group_selector::GroupSelector,
  instance::Instance,
  packages::Packages,
  semver_range::SemverRange,
};

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
  /// Group instances of each dependency together for comparison.
  pub dependencies_by_name: BTreeMap<String, Dependency>,
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
      dependencies_by_name: BTreeMap::new(),
      range: None,
    }
  }

  /// Add an instance to this version group
  pub fn add_instance(&mut self, instance: &Instance) {
    // Ensure that a group exists for this dependency name.
    if !self.dependencies_by_name.contains_key(&instance.name) {
      self.dependencies_by_name.insert(
        instance.name.clone(),
        Dependency::new(instance.name.clone()),
      );
    }

    // Get the group for this dependency name.
    let dependency = self.dependencies_by_name.get_mut(&instance.name).unwrap();

    // Track/count instances
    dependency.all.push(instance.id.clone());

    // Track/count unique version specifiers and which instances use them
    // 1. Ensure that a group exists for this specifier.
    if !dependency.by_specifier.contains_key(&instance.specifier) {
      dependency
        .by_specifier
        .insert(instance.specifier.clone(), vec![]);
    }

    // 2. Add this instance against its specifier
    dependency
      .by_specifier
      .get_mut(&instance.specifier)
      .unwrap()
      .push(instance.id.clone());

    // If this is the original source of a locally-developed package, keep a
    // reference to it
    if instance.dependency_type.name == "local" {
      dependency.local = Some(instance.id.clone());
    }

    // Track/count what specifier types we have encountered
    if instance.specifier.is_semver() {
      dependency.semver.push(instance.id.clone());
    } else {
      dependency.non_semver.push(instance.id.clone());
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
        dependencies_by_name: BTreeMap::new(),
        range: None,
      }
    } else if let Some(true) = group.is_ignored {
      SemverGroup {
        variant: SemverGroupVariant::Ignored,
        selector,
        dependencies_by_name: BTreeMap::new(),
        range: None,
      }
    } else if let Some(range) = &group.range {
      SemverGroup {
        variant: SemverGroupVariant::WithRange,
        selector,
        dependencies_by_name: BTreeMap::new(),
        range: SemverRange::new(range),
      }
    } else {
      panic!("Invalid semver group");
    }
  }

  pub fn visit(
    &self,
    config: &Config,
    // needed by same range groups, every instance in the project
    instances_by_id: &mut InstancesById,
    // when fixing, we write to the package.json files
    packages: &mut Packages,
    // chosen strategy to lint, fix, use different log output, etc
    effects: &mut impl Effects,
  ) {
    effects.on(Event::GroupVisited(&self.selector));

    let lint_ranges = &config.cli.options.ranges;
    let lint_versions = &config.cli.options.versions;

    match self.variant {
      SemverGroupVariant::Disabled => {}
      SemverGroupVariant::Ignored => {
        self.dependencies_by_name.values().for_each(|dependency| {
          effects.on(Event::DependencyIgnored(dependency));
        });
      }
      SemverGroupVariant::WithRange => {
        self.dependencies_by_name.values().for_each(|dependency| {
          let expected_range = self.range.as_ref().unwrap();
          let has_mismatch = dependency
            .by_specifier
            .iter()
            .any(|(specifier, instance_ids)| {
              // @TODO: should non-semver be classed as a mismatch?
              if !specifier.is_semver() {
                return false;
              }
              return !specifier.has_range(&expected_range);
            });

          if has_mismatch {
            effects.on(Event::DependencyMismatchesWithRange(dependency));
          } else {
            effects.on(Event::DependencyMatchesWithRange(dependency));
          }

          dependency.for_each_specifier(|(actual_specifier, instance_ids)| {
            if actual_specifier.is_semver() {
              if actual_specifier.has_range(&expected_range) {
                instance_ids.iter().for_each(|instance_id| {
                  effects.on(Event::InstanceMatchesWithRange(&MatchEvent {
                    instance_id: instance_id.clone(),
                    dependency,
                    specifier: actual_specifier.clone(),
                  }));
                });
              } else {
                let expected_specifier = actual_specifier.with_semver_range(expected_range);
                let expected_specifier = expected_specifier.unwrap();
                instance_ids.iter().for_each(|instance_id| {
                  let mismatch_event = &mut MismatchEvent {
                    instance_id: instance_id.clone(),
                    dependency,
                    expected_specifier: expected_specifier.clone(),
                    actual_specifier: actual_specifier.clone(),
                    matching_instance_ids: vec![], // @TODO: not needed
                    instances_by_id,
                    packages,
                  };
                  if dependency.is_local_instance(instance_id) {
                    effects.on(Event::InstanceMismatchCorruptsLocalVersion(mismatch_event));
                  } else {
                    effects.on(Event::InstanceMismatchesWithRange(mismatch_event));
                  }
                });
              }
            } else {
              println!("@TODO: Non-semver specifier: {:?}", actual_specifier);
            }
          });
        });
      }
    };
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
