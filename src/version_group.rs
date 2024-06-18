use log::info;
use node_semver::Range;
use serde::Deserialize;
use std::{collections::BTreeMap, vec};
use version_compare::{compare, Cmp};

use crate::{
  config::Config,
  dependency::{Dependency, InstanceIdsBySpecifier, InstancesById},
  effects::{
    BannedEvent, Effects, Event, MatchEvent, MismatchEvent, SameRangeMismatchEvent,
    SnapToMismatchEvent, UnsupportedMismatchEvent,
  },
  group_selector::GroupSelector,
  instance::Instance,
  packages::Packages,
  specifier::Specifier,
};

/// What behaviour has this group been configured to exhibit?
#[derive(Clone, Debug)]
pub enum Variant {
  Banned,
  Ignored,
  Pinned,
  SameRange,
  SnappedTo,
  HighestSemver,
  LowestSemver,
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

  ///
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
        prefer_version: None,
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(true) = group.is_ignored {
      return VersionGroup {
        variant: Variant::Ignored,
        selector,
        dependencies: BTreeMap::new(),
        prefer_version: None,
        pin_version: None,
        snap_to: None,
      };
    }
    if let Some(pin_version) = &group.pin_version {
      return VersionGroup {
        variant: Variant::Pinned,
        selector,
        dependencies: BTreeMap::new(),
        prefer_version: None,
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
          prefer_version: None,
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
        prefer_version: None,
        pin_version: None,
        snap_to: Some(snap_to.clone()),
      };
    }
    if let Some(prefer_version) = &group.prefer_version {
      return VersionGroup {
        variant: Variant::Standard,
        selector,
        dependencies: BTreeMap::new(),
        prefer_version: Some(if prefer_version == "lowestSemver" {
          PreferVersion::LowestSemver
        } else {
          PreferVersion::HighestSemver
        }),
        pin_version: None,
        snap_to: None,
      };
    }
    VersionGroup {
      variant: Variant::Standard,
      selector,
      dependencies: BTreeMap::new(),
      prefer_version: Some(PreferVersion::HighestSemver),
      pin_version: None,
      snap_to: None,
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

    let lint_versions = &config.cli.options.versions;

    match self.variant {
      Variant::Ignored => {
        self.dependencies.values().for_each(|dependency| {
          effects.on(Event::DependencyIgnored(dependency));
        });
      }
      Variant::Banned => {
        self.dependencies.values().for_each(|dependency| {
          effects.on(Event::DependencyBanned(dependency));
          dependency.for_each_instance_id(|(specifier, instance_id)| {
            effects.on(Event::InstanceBanned(&mut BannedEvent {
              instance_id: instance_id.clone(),
              dependency,
              specifier: specifier.clone(),
              instances_by_id,
              packages,
            }));
          });
        });
      }
      Variant::Pinned => {
        self.dependencies.values().for_each(|dependency| {
          info!("TODO: versions could be identical but not match the pinVersion");
          if !dependency.all_specifiers_are_identical() {
            effects.on(Event::DependencyMismatchesPinnedVersion(dependency));
            let expected_version = dependency.expected_version.as_ref().unwrap();
            let matching_instance_ids = dependency.get_matching_instance_ids();
            dependency.for_each_specifier(|(actual_specifier, instance_ids)| {
              if dependency.is_version_mismatch(actual_specifier) {
                instance_ids.iter().for_each(|instance_id| {
                  let mismatch_event = &mut MismatchEvent {
                    instance_id: instance_id.clone(),
                    dependency,
                    expected_specifier: expected_version.clone(),
                    actual_specifier: actual_specifier.clone(),
                    matching_instance_ids: matching_instance_ids.clone(),
                    instances_by_id,
                    packages,
                  };
                  if dependency.is_local_instance(instance_id) {
                    effects.on(Event::InstanceMismatchCorruptsLocalVersion(mismatch_event));
                  } else {
                    effects.on(Event::InstanceMismatchesPinnedVersion(mismatch_event));
                  }
                });
              }
            });
          } else {
            effects.on(Event::DependencyMatchesPinnedVersion(dependency));
          };
        });
      }
      Variant::SameRange => {
        self.dependencies.values().for_each(|dependency| {
          let mut mismatches: Vec<(InstanceIdsBySpecifier, InstanceIdsBySpecifier)> = vec![];
          dependency.for_each_specifier(|a| {
            let (specifier_a, instance_ids_a) = a;
            let range_a = specifier_a.unwrap().parse::<Range>().unwrap();
            dependency.for_each_specifier(|b| {
              let (specifier_b, instance_ids_b) = b;
              if specifier_a == specifier_b {
                return;
              }
              let range_b = specifier_b.unwrap().parse::<Range>().unwrap();
              if range_a.allows_all(&range_b) {
                return;
              }
              mismatches.push((
                InstanceIdsBySpecifier {
                  specifier: specifier_a.clone(),
                  instance_ids: instance_ids_a.clone(),
                },
                InstanceIdsBySpecifier {
                  specifier: specifier_b.clone(),
                  instance_ids: instance_ids_b.clone(),
                },
              ));
            })
          });
          if mismatches.len() == 0 {
            effects.on(Event::DependencyMatchesSameRange(dependency));
          } else {
            effects.on(Event::DependencyMismatchesSameRange(dependency));
            mismatches.into_iter().for_each(
              |(
                InstanceIdsBySpecifier {
                  specifier,
                  instance_ids,
                },
                InstanceIdsBySpecifier {
                  specifier: specifier_outside_range,
                  instance_ids: instance_ids_outside_range,
                },
              )| {
                instance_ids.iter().for_each(|instance_id| {
                  effects.on(Event::InstanceMismatchesSameRange(
                    &mut SameRangeMismatchEvent {
                      dependency,
                      instances_by_id,
                      packages,
                      instance_id: instance_id.clone(),
                      specifier: specifier.clone(),
                      specifier_outside_range: specifier_outside_range.clone(),
                      instance_ids_outside_range: instance_ids_outside_range.clone(),
                    },
                  ));
                });
              },
            );
          }
        });
      }
      Variant::SnappedTo => {
        if let Some(snap_to) = &self.snap_to {
          self.dependencies.values().for_each(|dependency| {
            let mismatches = get_snap_to_mismatches(snap_to, instances_by_id, dependency);
            if mismatches.len() == 0 {
              effects.on(Event::DependencyMatchesSnapTo(dependency));
            } else {
              effects.on(Event::DependencyMismatchesSnapTo(dependency));
              mismatches.into_iter().for_each(|mismatch| {
                mismatch.instance_ids.iter().for_each(|instance_id| {
                  effects.on(Event::InstanceMismatchesSnapTo(&mut SnapToMismatchEvent {
                    instance_id: instance_id.clone(),
                    dependency,
                    expected_specifier: mismatch.expected_specifier.clone(),
                    actual_specifier: mismatch.actual_specifier.clone(),
                    snap_to_instance_id: mismatch.snap_to_instance_id.clone(),
                    instances_by_id,
                    packages,
                  }));
                });
              });
            }
          });
        }
      }
      Variant::Standard => {
        self.dependencies.values().for_each(|dependency| {
          if dependency.all_specifiers_are_identical() {
            effects.on(Event::DependencyMatchesStandard(dependency));
            dependency.for_each_instance_id(|(specifier, instance_id)| {
              effects.on(Event::InstanceMatchesStandard(&MatchEvent {
                instance_id: instance_id.clone(),
                dependency,
                specifier: specifier.clone(),
              }));
            });
          } else {
            effects.on(Event::DependencyMismatchesStandard(dependency));
            dependency.for_each_specifier(|(actual_specifier, instance_ids)| {
              if !dependency.is_version_mismatch(actual_specifier) {
                instance_ids.iter().for_each(|instance_id| {
                  effects.on(Event::InstanceMatchesStandard(&MatchEvent {
                    instance_id: instance_id.clone(),
                    dependency,
                    specifier: actual_specifier.clone(),
                  }));
                });
              } else {
                // local mismatch
                if let Some(local_instance_id) = &dependency.local_instance_id {
                  let local_instance = instances_by_id.get(local_instance_id).unwrap();
                  let expected_specifier = local_instance.specifier.clone();
                  instance_ids.iter().for_each(|instance_id| {
                    effects.on(Event::InstanceMismatchesLocalVersion(&mut MismatchEvent {
                      instance_id: instance_id.clone(),
                      dependency,
                      expected_specifier: expected_specifier.clone(),
                      actual_specifier: actual_specifier.clone(),
                      matching_instance_ids: vec![local_instance_id.clone()],
                      instances_by_id,
                      packages,
                    }));
                  });
                }
                // @FIXME: some non-semver versions ARE supported
                else if !dependency.non_semver.is_empty() {
                  instance_ids.iter().for_each(|instance_id| {
                    effects.on(Event::InstanceUnsupportedMismatch(
                      &mut UnsupportedMismatchEvent {
                        instance_id: instance_id.clone(),
                        dependency,
                        specifier: actual_specifier.clone(),
                        instances_by_id,
                      },
                    ));
                  });
                }
                // higher or lower mismatch
                else if let Some(prefer_version) = &self.prefer_version {
                  let expected_specifier = dependency.expected_version.clone().unwrap();
                  let matching_instance_ids = dependency.get_matching_instance_ids();
                  instance_ids.iter().for_each(|instance_id| {
                    let mut mismatch_event = MismatchEvent {
                      instance_id: instance_id.clone(),
                      dependency,
                      expected_specifier: expected_specifier.clone(),
                      actual_specifier: actual_specifier.clone(),
                      matching_instance_ids: matching_instance_ids.clone(),
                      instances_by_id,
                      packages,
                    };
                    effects.on(if matches!(prefer_version, PreferVersion::LowestSemver) {
                      Event::InstanceMismatchesLowestVersion(&mut mismatch_event)
                    } else {
                      Event::InstanceMismatchesHighestVersion(&mut mismatch_event)
                    });
                  });
                } else {
                  panic!("Unhandled mismatch");
                }
              };
            });
          };
        });
      }
    };
  }
}

/// Return the first instance from the packages which should be snapped to for a
/// given dependency.
fn get_snap_to_instance<'a>(
  snap_to: &Vec<String>,
  dependency_name: &String,
  instances_by_id: &'a mut InstancesById,
) -> Option<&'a Instance> {
  for instance in instances_by_id.values() {
    if instance.name == *dependency_name {
      for snapped_to_package_name in snap_to {
        if instance.package_name == *snapped_to_package_name {
          return Some(instance);
        }
      }
    }
  }
  return None;
}

struct SnapToMismatches {
  pub instance_ids: Vec<String>,
  pub actual_specifier: Specifier,
  pub expected_specifier: Specifier,
  pub snap_to_instance_id: String,
}

/// Find all instances which have and do not match their corresponding snap_to
/// instance
fn get_snap_to_mismatches(
  snap_to: &Vec<String>,
  instances_by_id: &mut InstancesById,
  dependency: &Dependency,
) -> Vec<SnapToMismatches> {
  let mut mismatches: Vec<SnapToMismatches> = vec![];
  let dependency_name = &dependency.name;
  if let Some(snappable_instance) = get_snap_to_instance(snap_to, dependency_name, instances_by_id)
  {
    let expected = &snappable_instance.specifier;
    dependency
      .by_initial_specifier
      .iter()
      .filter(|(actual, _)| *actual != expected)
      .for_each(|(specifier, instance_ids)| {
        let mismatches_with = (expected.clone(), snappable_instance.id.clone());
        let target_instances = (specifier.clone(), instance_ids.clone());
        mismatches.push(SnapToMismatches {
          instance_ids: instance_ids.clone(),
          actual_specifier: specifier.clone(),
          expected_specifier: expected.clone(),
          snap_to_instance_id: snappable_instance.id.clone(),
        });
      });
  }
  mismatches
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
