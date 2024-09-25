use itertools::Itertools;
use std::{cmp::Ordering, collections::HashMap};

use crate::{
  config::Config,
  context::Context,
  effects::{Effects, Event, InstanceEvent, InstanceEventVariant},
  packages::Packages,
  specifier::Specifier,
  version_group::Variant,
};

pub fn visit_packages(config: &Config, packages: Packages, effects: &mut impl Effects) {
  const VALID: u8 = 0;
  const WARNING: u8 = 1;
  const INVALID: u8 = 2;

  let Context {
    mut instances_by_id,
    semver_groups,
    version_groups,
  } = Context::create(config, &packages);

  let local_specifiers_by_name: HashMap<String, Specifier> = packages
    .by_name
    .iter()
    .map(|(name, package_json)| {
      (
        name.clone(),
        package_json
          .get_prop("/version")
          .and_then(|value| value.as_str())
          .map(|str| str.to_string())
          .map(|string| Specifier::new(&string))
          .unwrap_or(Specifier::None),
      )
    })
    .collect();

  effects.set_packages(packages);

  if config.cli.options.versions {
    effects.on(Event::EnterVersionsAndRanges, &mut instances_by_id);

    version_groups
      .iter()
      // fix snapped to groups last, so that the packages they're snapped to
      // have any fixes applied to them first
      .sorted_by(|a, b| {
        if matches!(a.variant, Variant::SnappedTo) {
          Ordering::Greater
        } else if matches!(b.variant, Variant::SnappedTo) {
          Ordering::Less
        } else {
          Ordering::Equal
        }
      })
      .for_each(|group| {
        effects.on(Event::GroupVisited(&group.selector), &mut instances_by_id);

        group.dependencies.values().for_each(|dependency| {
          let mut expected: Option<Specifier> = None;
          let mut queue: Vec<InstanceEvent> = vec![];
          let mut severity = VALID;
          let mut mark_as = |level: u8| {
            if severity < level {
              severity = level;
            }
          };

          match dependency.variant {
            Variant::Banned => {
              dependency.all.iter().for_each(|instance_id| {
                let instance = instances_by_id.get_mut(instance_id).unwrap();
                if instance.is_local {
                  mark_as(WARNING);
                  queue.push(InstanceEvent {
                    dependency,
                    instance_id: instance_id.clone(),
                    variant: InstanceEventVariant::LocalInstanceMistakenlyBanned,
                  });
                } else {
                  mark_as(INVALID);
                  instance.expected = Specifier::None;
                  queue.push(InstanceEvent {
                    dependency,
                    instance_id: instance_id.clone(),
                    variant: InstanceEventVariant::InstanceIsBanned,
                  });
                }
              });
            }
            Variant::HighestSemver | Variant::LowestSemver => {
              let prefer_highest = matches!(dependency.variant, Variant::HighestSemver);
              let preferred_order: Ordering = if prefer_highest { Ordering::Greater } else { Ordering::Less };
              let label: &str = if prefer_highest { "highest" } else { "lowest" };

              match local_specifiers_by_name.get(&dependency.name) {
                Some(local_specifier) => {
                  dependency.all.iter().for_each(|instance_id| {
                    let instance = instances_by_id.get_mut(instance_id).unwrap();
                    if instance.is_local {
                      if instance.has_range_mismatch(local_specifier) {
                        mark_as(WARNING);
                        expected = Some(local_specifier.clone());
                        instance.expected = local_specifier.clone();
                        queue.push(InstanceEvent {
                          dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup,
                        });
                      } else {
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::LocalInstanceIsPreferred,
                        });
                      }
                    } else if matches!(local_specifier, Specifier::None) {
                      mark_as(INVALID);
                      instance.expected = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMismatchesLocalWithMissingVersion,
                      });
                    } else if instance.actual == *local_specifier {
                      if instance.has_range_mismatch(local_specifier) {
                        mark_as(INVALID);
                        instance.expected = instance.get_fixed_range_mismatch();
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup,
                        });
                      } else {
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMatchesLocal,
                        });
                      }
                    } else {
                      mark_as(INVALID);
                      instance.expected = local_specifier.clone();
                      expected = Some(local_specifier.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMismatchesLocal,
                      });
                    }
                  });
                }
                None => {
                  if dependency.all_are_semver(&instances_by_id) {
                    match dependency.get_highest_or_lowest_semver(&instances_by_id, preferred_order) {
                      Some(preferred) => {
                        dependency.all.iter().for_each(|instance_id| {
                          let instance = instances_by_id.get_mut(instance_id).unwrap();
                          if instance.actual == preferred {
                            if instance.has_range_mismatch(&preferred) {
                              mark_as(INVALID);
                              instance.expected = instance.get_fixed_range_mismatch();
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency,
                                instance_id: instance_id.clone(),
                                variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup,
                              });
                            } else {
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency,
                                instance_id: instance_id.clone(),
                                variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
                              });
                            }
                          } else if instance.expected == preferred {
                            if instance.matches_semver_group(&instance.expected) && !instance.matches_semver_group(&instance.actual) {
                              mark_as(INVALID);
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency,
                                instance_id: instance_id.clone(),
                                variant: InstanceEventVariant::InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed,
                              });
                            }
                          } else {
                            // check this
                            mark_as(INVALID);
                            instance.expected = preferred.clone();
                            expected = Some(preferred.clone());
                            queue.push(InstanceEvent {
                              dependency,
                              instance_id: instance_id.clone(),
                              variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
                            });
                          }
                        });
                      }
                      None => {
                        panic!("No {} semver found for dependency {:?}", label, dependency);
                      }
                    }
                  } else if dependency.all_are_identical(&instances_by_id) {
                    mark_as(WARNING);
                    dependency.all.iter().for_each(|instance_id| {
                      let instance = instances_by_id.get(instance_id).unwrap();
                      expected = Some(instance.actual.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMatchesButIsUnsupported,
                      });
                    });
                  } else {
                    mark_as(INVALID);
                    dependency.all.iter().for_each(|instance_id| {
                      let instance = instances_by_id.get_mut(instance_id).unwrap();
                      instance.expected = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMismatchesAndIsUnsupported,
                      });
                    });
                  }
                }
              }
            }
            Variant::Ignored => {
              dependency.all.iter().for_each(|instance_id| {
                let instance = instances_by_id.get(instance_id).unwrap();
                queue.push(InstanceEvent {
                  dependency,
                  instance_id: instance_id.clone(),
                  variant: InstanceEventVariant::InstanceIsIgnored,
                });
              });
            }
            Variant::Pinned => match &dependency.pinned_specifier {
              Some(pinned) => {
                dependency.all.iter().for_each(|instance_id| {
                  let instance = instances_by_id.get_mut(instance_id).unwrap();
                  // CHECK THIS Eq WORKS
                  if instance.actual == *pinned {
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency,
                      instance_id: instance_id.clone(),
                      variant: InstanceEventVariant::InstanceMatchesPinned,
                    });
                  } else if instance.has_range_mismatch(pinned) {
                    if instance.is_local {
                      mark_as(WARNING);
                      expected = Some(pinned.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup,
                      });
                    } else {
                      mark_as(INVALID);
                      instance.expected = instance.get_fixed_range_mismatch();
                      expected = Some(pinned.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup,
                      });
                    }
                  } else if instance.is_local {
                    mark_as(WARNING);
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency,
                      instance_id: instance_id.clone(),
                      variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned,
                    });
                  } else {
                    mark_as(INVALID);
                    instance.expected = pinned.clone();
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency,
                      instance_id: instance_id.clone(),
                      variant: InstanceEventVariant::InstanceMismatchesPinned,
                    });
                  }
                });
              }
              None => {
                panic!("No pinned specifier found for dependency {:?}", dependency);
              }
            },
            Variant::SameRange => {
              if dependency.all_are_semver(&instances_by_id) {
                let mismatches = dependency.get_same_range_mismatches(&instances_by_id);
                dependency.all.iter().for_each(|instance_id| {
                  let instance = instances_by_id.get_mut(instance_id).unwrap();
                  // CHECK THIS OVER
                  if instance.has_range_mismatch(&instance.expected) {
                    if mismatches.contains_key(&instance.actual) {
                      if mismatches.contains_key(&instance.expected) {
                        mark_as(INVALID);
                        instance.expected = Specifier::None;
                        queue.push(InstanceEvent {
                          dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups,
                        });
                      } else {
                        mark_as(INVALID);
                        instance.expected = Specifier::None;
                        queue.push(InstanceEvent {
                          dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups,
                        });
                      }
                    } else if mismatches.contains_key(&instance.expected) {
                      mark_as(INVALID);
                      instance.expected = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup,
                      });
                    } else {
                      mark_as(INVALID);
                      instance.expected = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup,
                      });
                    }
                  } else if mismatches.contains_key(&instance.actual) {
                    mark_as(INVALID);
                    instance.expected = Specifier::None;
                    queue.push(InstanceEvent {
                      dependency,
                      instance_id: instance_id.clone(),
                      variant: InstanceEventVariant::InstanceMismatchesSameRangeGroup,
                    });
                  } else {
                    queue.push(InstanceEvent {
                      dependency,
                      instance_id: instance_id.clone(),
                      variant: InstanceEventVariant::InstanceMatchesSameRangeGroup,
                    });
                  }
                  // /CHECK THIS OVER
                });
              } else if dependency.all_are_identical(&instances_by_id) {
                mark_as(WARNING);
                dependency.all.iter().for_each(|instance_id| {
                  let instance = instances_by_id.get(instance_id).unwrap();
                  queue.push(InstanceEvent {
                    dependency,
                    instance_id: instance_id.clone(),
                    variant: InstanceEventVariant::InstanceMatchesButIsUnsupported,
                  });
                });
              } else {
                mark_as(INVALID);
                dependency.all.iter().for_each(|instance_id| {
                  let instance = instances_by_id.get_mut(instance_id).unwrap();
                  instance.expected = Specifier::None;
                  queue.push(InstanceEvent {
                    dependency,
                    instance_id: instance_id.clone(),
                    variant: InstanceEventVariant::InstanceMismatchesAndIsUnsupported,
                  });
                });
              }
            }
            Variant::SnappedTo => {
              let snapped_to_specifier = dependency.get_snapped_to_specifier(&instances_by_id);
              // @FIXME
              expected = Some(Specifier::new("0.0.0"));
            }
          };

          if severity == VALID {
            effects.on(Event::DependencyValid(dependency, expected), &mut instances_by_id);
          } else if severity == WARNING {
            effects.on(Event::DependencyWarning(dependency, expected), &mut instances_by_id);
          } else {
            effects.on(Event::DependencyInvalid(dependency, expected), &mut instances_by_id);
          }

          // Sort instances by actual specifier and then package name
          queue.sort_by(|a, b| {
            let a = &instances_by_id.get(&a.instance_id).unwrap();
            let b = &instances_by_id.get(&b.instance_id).unwrap();

            if matches!(&a.actual, Specifier::None) {
              return Ordering::Greater;
            }

            if matches!(&b.actual, Specifier::None) {
              return Ordering::Less;
            }

            let specifier_order = a.actual.unwrap().cmp(&b.actual.unwrap());

            if matches!(specifier_order, Ordering::Equal) {
              b.package_name.cmp(&a.package_name)
            } else {
              specifier_order
            }
          });

          while let Some(event) = queue.pop() {
            effects.on_instance(event, &mut instances_by_id);
          }
        });
      });
  }

  // @TODO
  // if config.cli.options.format {
  //   effects.on(Event::EnterFormat, &mut instances_by_id);

  //   let mut packages = effects.get_packages();
  //   let InMemoryFormattingStatus { was_valid, was_invalid } = format::fix(&config, &mut packages);

  //   if !was_valid.is_empty() {
  //     effects.on(Event::PackagesMatchFormatting(was_valid), &mut instances_by_id);
  //   }
  //   if !was_invalid.is_empty() {
  //     effects.on(Event::PackagesMismatchFormatting(was_invalid), &mut instances_by_id);
  //   }
  // }

  effects.on(Event::ExitCommand, &mut instances_by_id);
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    effects::mock::MockEffects,
    expect::{expect, ExpectedMatchEvent, ExpectedMismatchEvent},
  };
  use serde_json::json;

  #[test]
  fn reports_one_highest_version_mismatch_in_one_file() {
    let config = Config::new();
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "1.0.0"
      },
      "devDependencies": {
        "wat": "2.0.0"
      }
    })]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![ExpectedMatchEvent {
        variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
        dependency_name: "wat",
        instance_id: "wat in /devDependencies of package-a",
        actual: "2.0.0",
      }])
      .to_have_mismatches(vec![ExpectedMismatchEvent {
        variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-a",
        actual: "1.0.0",
        expected: "2.0.0",
      }]);
  }

  #[test]
  fn reports_many_highest_version_mismatches_in_one_file() {
    let config = Config::new();
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "0.1.0"
      },
      "devDependencies": {
        "wat": "0.3.0"
      },
      "peerDependencies": {
        "wat": "0.2.0"
      }
    })]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![ExpectedMatchEvent {
        variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
        dependency_name: "wat",
        instance_id: "wat in /devDependencies of package-a",
        actual: "0.3.0",
      }])
      .to_have_mismatches(vec![
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
          dependency_name: "wat",
          instance_id: "wat in /dependencies of package-a",
          actual: "0.1.0",
          expected: "0.3.0",
        },
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
          dependency_name: "wat",
          instance_id: "wat in /peerDependencies of package-a",
          actual: "0.2.0",
          expected: "0.3.0",
        },
      ]);
  }

  #[test]
  fn reports_highest_version_mismatches_in_many_files() {
    let config = Config::new();
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "dependencies": {
          "wat": "1.0.0"
        }
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "wat": "2.0.0"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![ExpectedMatchEvent {
        variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-b",
        actual: "2.0.0",
      }])
      .to_have_mismatches(vec![ExpectedMismatchEvent {
        variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-a",
        actual: "1.0.0",
        expected: "2.0.0",
      }]);
  }

  #[test]
  fn does_not_consider_instances_in_different_version_groups_a_highest_version_mismatch() {
    let config = Config::from_mock(json!({
      "versionGroups": [
        { "packages": ["package-a"] },
        { "packages": ["package-b"] }
      ]
    }));
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "dependencies": {
          "good": "1.0.0"
        }
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "good": "2.0.0"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![
        ExpectedMatchEvent {
          variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
          dependency_name: "good",
          instance_id: "good in /dependencies of package-a",
          actual: "1.0.0",
        },
        ExpectedMatchEvent {
          variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
          dependency_name: "good",
          instance_id: "good in /dependencies of package-b",
          actual: "2.0.0",
        },
      ])
      .to_have_mismatches(vec![]);
  }

  #[test]
  fn rejects_pinned_version_when_it_would_replace_local_version() {
    let config = Config::from_mock(json!({
      "versionGroups": [{
        "dependencies": ["package-a"],
        "pinVersion": "1.2.0"
      }]
    }));
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0"
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "package-a": "1.1.0"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![])
      .to_have_mismatches(vec![
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned,
          dependency_name: "package-a",
          instance_id: "package-a in /version of package-a",
          actual: "1.0.0",
          expected: "1.0.0",
        },
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceMismatchesPinned,
          dependency_name: "package-a",
          instance_id: "package-a in /dependencies of package-b",
          actual: "1.1.0",
          expected: "1.2.0",
        },
      ]);
  }

  #[test]
  fn does_not_confuse_highest_version_matches_and_mismatches() {
    let config = Config::new();
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "dependencies": {
          "mix": "0.3.0"
        },
        "devDependencies": {
          "mix": "0.1.0"
        },
        "peerDependencies": {
          "mix": "0.2.0"
        }
      }),
      json!({
        "name": "package-b",
        "devDependencies": {
          "mix": "0.3.0"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![
        ExpectedMatchEvent {
          variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
          dependency_name: "mix",
          instance_id: "mix in /dependencies of package-a",
          actual: "0.3.0",
        },
        ExpectedMatchEvent {
          variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
          dependency_name: "mix",
          instance_id: "mix in /devDependencies of package-b",
          actual: "0.3.0",
        },
      ])
      .to_have_mismatches(vec![
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
          dependency_name: "mix",
          instance_id: "mix in /devDependencies of package-a",
          actual: "0.1.0",
          expected: "0.3.0",
        },
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
          dependency_name: "mix",
          instance_id: "mix in /peerDependencies of package-a",
          actual: "0.2.0",
          expected: "0.3.0",
        },
      ]);
  }

  #[test]
  fn reports_local_version_mismatch_when_an_instance_uses_a_higher_version() {
    let config = Config::new();
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0"
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "package-a": "1.1.0"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![ExpectedMatchEvent {
        variant: InstanceEventVariant::LocalInstanceIsPreferred,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      }])
      .to_have_mismatches(vec![ExpectedMismatchEvent {
        variant: InstanceEventVariant::InstanceMismatchesLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "1.1.0",
        expected: "1.0.0",
      }]);
  }

  #[test]
  fn instance_has_same_version_as_local_package_but_does_not_match_its_semver_group() {
    let config = Config::from_mock(json!({
      "semverGroups": [{
        "range": "^"
      }]
    }));
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0"
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "package-a": "1.0.0"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![])
      .to_have_mismatches(vec![
        // refuse to break local package's version
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup,
          dependency_name: "package-a",
          instance_id: "package-a in /version of package-a",
          actual: "1.0.0",
          expected: "1.0.0",
        },
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup,
          dependency_name: "package-a",
          instance_id: "package-a in /dependencies of package-b",
          actual: "1.0.0",
          expected: "^1.0.0",
        },
      ]);
  }

  #[test]
  fn instance_is_highest_or_lowest_semver_once_semver_group_is_fixed() {
    let config = Config::from_mock(json!({
      "semverGroups": [{
        "dependencyTypes": ["dev"],
        "range": ">"
      }]
    }));
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "foo": "1.0.0"
      },
      "devDependencies": {
        "foo": "1.0.0"
      }
    })]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![])
      .to_have_mismatches(vec![
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
          dependency_name: "foo",
          instance_id: "foo in /dependencies of package-a",
          actual: "1.0.0",
          expected: ">1.0.0",
        },
        ExpectedMismatchEvent {
          variant: InstanceEventVariant::InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed,
          dependency_name: "foo",
          instance_id: "foo in /devDependencies of package-a",
          actual: "1.0.0",
          expected: ">1.0.0",
        },
      ]);
  }

  #[test]
  fn instance_is_no_longer_highest_or_lowest_semver_once_semver_group_is_fixed() {
    let config = Config::from_mock(json!({
      "semverGroups": [{
        "dependencyTypes": ["dev"],
        "range": "<"
      }]
    }));
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "foo": "1.0.0"
      },
      "devDependencies": {
        "foo": "1.0.0"
      }
    })]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![ExpectedMatchEvent {
        variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: "1.0.0",
      }])
      .to_have_mismatches(vec![ExpectedMismatchEvent {
        variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup,
        dependency_name: "foo",
        instance_id: "foo in /devDependencies of package-a",
        actual: "1.0.0",
        expected: "<1.0.0",
      }]);
  }

  #[test]
  fn reports_local_version_mismatch_when_an_instance_uses_workspace_protocol() {
    let config = Config::new();
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0"
      }),
      json!({
        "name": "package-b",
        "devDependencies": {
          "package-a": "workspace:*"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![ExpectedMatchEvent {
        variant: InstanceEventVariant::LocalInstanceIsPreferred,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      }])
      .to_have_mismatches(vec![ExpectedMismatchEvent {
        variant: InstanceEventVariant::InstanceMismatchesLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /devDependencies of package-b",
        actual: "workspace:*",
        expected: "1.0.0",
      }]);
  }

  #[test]
  fn protects_local_version_when_naively_pinned_to_use_workspace_protocol() {
    let config = Config::from_mock(json!({
      "versionGroups": [{
        "dependencyTypes": ["**"],
        "dependencies": ["**"],
        "packages": ["**"],
        "pinVersion": "workspace:*",
      }]
    }));
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0"
      }),
      json!({
        "name": "package-b",
        "devDependencies": {
          "package-a": "workspace:*"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![ExpectedMatchEvent {
        variant: InstanceEventVariant::InstanceMatchesPinned,
        dependency_name: "package-a",
        instance_id: "package-a in /devDependencies of package-b",
        actual: "workspace:*",
      }])
      .to_have_mismatches(vec![ExpectedMismatchEvent {
        variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
        expected: "1.0.0",
      }]);
  }

  #[test]
  fn reports_unfixable_local_version_mismatch_when_local_version_is_missing() {
    let config = Config::new();
    let mut effects = MockEffects::new(&config);
    let packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a"
      }),
      json!({
        "name": "package-b",
        "devDependencies": {
          "package-a": "0.1.0"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_matches(vec![])
      .to_have_mismatches(vec![ExpectedMismatchEvent {
        variant: InstanceEventVariant::InstanceMismatchesLocalWithMissingVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /devDependencies of package-b",
        actual: "0.1.0",
        expected: "VERSION_IS_MISSING",
      }]);
  }

  #[test]
  #[ignore]
  fn reports_unfixable_local_version_mismatch_when_local_version_is_not_exact_semver() {
    panic!("@TODO");
  }

  #[test]
  #[ignore]
  fn reports_local_version_mismatch_when_an_instance_has_same_version_but_different_range() {
    panic!("@TODO");
  }
}
