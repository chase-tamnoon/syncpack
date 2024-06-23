use itertools::Itertools;
use std::cmp::Ordering;
use version_compare::Cmp;

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
  } = Context::create(&config, &packages);

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
                    dependency: &dependency,
                    instance_id: instance_id.clone(),
                    variant: InstanceEventVariant::LocalInstanceMistakenlyBanned,
                  });
                } else {
                  mark_as(INVALID);
                  instance.expected = Specifier::None;
                  queue.push(InstanceEvent {
                    dependency: &dependency,
                    instance_id: instance_id.clone(),
                    variant: InstanceEventVariant::InstanceIsBanned,
                  });
                }
              });
            }
            Variant::HighestSemver | Variant::LowestSemver => {
              let prefer_highest = matches!(dependency.variant, Variant::HighestSemver);
              let preferred_order: Cmp = if prefer_highest { Cmp::Gt } else { Cmp::Lt };
              let label: &str = if prefer_highest { "highest" } else { "lowest" };
              match dependency.get_local_specifier(&instances_by_id) {
                Some(local_specifier) => {
                  dependency.all.iter().for_each(|instance_id| {
                    let instance = instances_by_id.get_mut(instance_id).unwrap();
                    if instance.is_local {
                      if instance.has_range_mismatch(&local_specifier) {
                        mark_as(WARNING);
                        expected = Some(local_specifier.clone());
                        instance.expected = local_specifier.clone();
                        queue.push(InstanceEvent {
                          dependency: &dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup,
                        });
                      } else {
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency: &dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::LocalInstanceIsPreferred,
                        });
                      }
                    } else {
                        // CHECK THIS Eq WORKS
                      if instance.actual == local_specifier {
                        if instance.has_range_mismatch(&local_specifier) {
                          mark_as(INVALID);
                          instance.expected = instance.get_fixed_range_mismatch();
                          expected = Some(local_specifier.clone());
                          queue.push(InstanceEvent {
                            dependency: &dependency,
                            instance_id: instance_id.clone(),
                            variant: InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup,
                          });
                        } else {
                          expected = Some(local_specifier.clone());
                          queue.push(InstanceEvent {
                            dependency: &dependency,
                            instance_id: instance_id.clone(),
                            variant: InstanceEventVariant::InstanceMatchesLocal,
                          });
                        }
                      } else {
                        mark_as(INVALID);
                        instance.expected = local_specifier.clone();
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency: &dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMismatchesLocal,
                        });
                      }
                    }
                  });
                }
                None => {
                  if dependency.all_are_semver(&instances_by_id) {
                    match dependency.get_highest_or_lowest_semver(&instances_by_id, preferred_order) {
                      Some(preferred) => {
                        dependency.all.iter().for_each(|instance_id| {
                          let instance = instances_by_id.get_mut(instance_id).unwrap();
                          // CHECK THIS Eq WORKS
                          if instance.actual == preferred {
                            if instance.has_range_mismatch(&preferred) {
                              mark_as(INVALID);
                              instance.expected = instance.get_fixed_range_mismatch();
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency: &dependency,
                                instance_id: instance_id.clone(),
                                variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesSemverGroup,
                              });
                            } else {
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency: &dependency,
                                instance_id: instance_id.clone(),
                                variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
                              });
                            }
                          } else {
                            mark_as(INVALID);
                            instance.expected = preferred.clone();
                            expected = Some(preferred.clone());
                            queue.push(InstanceEvent {
                              dependency: &dependency,
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
                        dependency: &dependency,
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
                        dependency: &dependency,
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
                  dependency: &dependency,
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
                      dependency: &dependency,
                      instance_id: instance_id.clone(),
                      variant: InstanceEventVariant::InstanceMatchesPinned,
                    });
                  } else if instance.has_range_mismatch(&pinned) {
                    if instance.is_local {
                      mark_as(WARNING);
                      expected = Some(pinned.clone());
                      queue.push(InstanceEvent {
                        dependency: &dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup,
                      });
                    } else {
                      mark_as(INVALID);
                      instance.expected = instance.get_fixed_range_mismatch();
                      expected = Some(pinned.clone());
                      queue.push(InstanceEvent {
                        dependency: &dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup,
                      });
                    }
                  } else if instance.is_local {
                    mark_as(WARNING);
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency: &dependency,
                      instance_id: instance_id.clone(),
                      variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned,
                    });
                  } else {
                    mark_as(INVALID);
                    instance.expected = pinned.clone();
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency: &dependency,
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
                          dependency: &dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups,
                        });
                      } else {
                        mark_as(INVALID);
                        instance.expected = Specifier::None;
                        queue.push(InstanceEvent {
                          dependency: &dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups,
                        });
                      }
                    } else {
                      if mismatches.contains_key(&instance.expected) {
                        mark_as(INVALID);
                        instance.expected = Specifier::None;
                        queue.push(InstanceEvent {
                          dependency: &dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup,
                        });
                      } else {
                        mark_as(INVALID);
                        instance.expected = Specifier::None;
                        queue.push(InstanceEvent {
                          dependency: &dependency,
                          instance_id: instance_id.clone(),
                          variant: InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup,
                        });
                      }
                    }
                  } else {
                    if mismatches.contains_key(&instance.actual) {
                      mark_as(INVALID);
                      instance.expected = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency: &dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMismatchesSameRangeGroup,
                      });
                    } else {
                      queue.push(InstanceEvent {
                        dependency: &dependency,
                        instance_id: instance_id.clone(),
                        variant: InstanceEventVariant::InstanceMatchesSameRangeGroup,
                      });
                    }
                  }
                  // /CHECK THIS OVER
                });
              } else if dependency.all_are_identical(&instances_by_id) {
                mark_as(WARNING);
                dependency.all.iter().for_each(|instance_id| {
                  let instance = instances_by_id.get(instance_id).unwrap();
                  queue.push(InstanceEvent {
                    dependency: &dependency,
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
                    dependency: &dependency,
                    instance_id: instance_id.clone(),
                    variant: InstanceEventVariant::InstanceMismatchesAndIsUnsupported,
                  });
                });
              }
            }
            Variant::SnappedTo => {
              let snapped_to_specifier = dependency.get_snapped_to_specifier(&instances_by_id);
              // @FIXME
              expected = Some(Specifier::new(&"0.0.0".to_string()));
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

            let specifier_order = a.actual.unwrap().cmp(&&b.actual.unwrap());

            if matches!(specifier_order, Ordering::Equal) {
              return b.package_name.cmp(&a.package_name);
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
    effects_mock::MockEffects,
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
      .to_have_instance_matches_highest_or_lowest_semver(vec![ExpectedMatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /devDependencies of package-a",
        actual: "2.0.0",
      }])
      .to_have_instance_mismatches_highest_or_lowest_semver(vec![ExpectedMismatchEvent {
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
      .to_have_instance_matches_highest_or_lowest_semver(vec![ExpectedMatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /devDependencies of package-a",
        actual: "0.3.0",
      }])
      .to_have_instance_mismatches_highest_or_lowest_semver(vec![
        ExpectedMismatchEvent {
          dependency_name: "wat",
          instance_id: "wat in /dependencies of package-a",
          actual: "0.1.0",
          expected: "0.3.0",
        },
        ExpectedMismatchEvent {
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
      .to_have_instance_matches_highest_or_lowest_semver(vec![ExpectedMatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-b",
        actual: "2.0.0",
      }])
      .to_have_instance_mismatches_highest_or_lowest_semver(vec![ExpectedMismatchEvent {
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
      .to_have_instance_mismatches_highest_or_lowest_semver(vec![])
      .to_have_instance_matches_highest_or_lowest_semver(vec![
        ExpectedMatchEvent {
          dependency_name: "good",
          instance_id: "good in /dependencies of package-a",
          actual: "1.0.0",
        },
        ExpectedMatchEvent {
          dependency_name: "good",
          instance_id: "good in /dependencies of package-b",
          actual: "2.0.0",
        },
      ]);
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
      .to_have_local_instance_mistakenly_mismatches_pinned(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
        expected: "1.0.0",
      }])
      .to_have_instance_mismatches_pinned(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "1.1.0",
        expected: "1.2.0",
      }]);
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
      .to_have_instance_matches_highest_or_lowest_semver(vec![
        ExpectedMatchEvent {
          dependency_name: "mix",
          instance_id: "mix in /dependencies of package-a",
          actual: "0.3.0",
        },
        ExpectedMatchEvent {
          dependency_name: "mix",
          instance_id: "mix in /devDependencies of package-b",
          actual: "0.3.0",
        },
      ])
      .to_have_instance_mismatches_highest_or_lowest_semver(vec![
        ExpectedMismatchEvent {
          dependency_name: "mix",
          instance_id: "mix in /devDependencies of package-a",
          actual: "0.1.0",
          expected: "0.3.0",
        },
        ExpectedMismatchEvent {
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
        },
        "devDependencies": {
          "package-a": "workspace:*"
        }
      }),
    ]);

    visit_packages(&config, packages, &mut effects);

    expect(&effects)
      .to_have_local_instance_is_preferred(vec![ExpectedMatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      }])
      .to_have_instance_mismatches_local(vec![
        ExpectedMismatchEvent {
          dependency_name: "package-a",
          instance_id: "package-a in /dependencies of package-b",
          actual: "1.1.0",
          expected: "1.0.0",
        },
        ExpectedMismatchEvent {
          dependency_name: "package-a",
          instance_id: "package-a in /devDependencies of package-b",
          actual: "workspace:*",
          expected: "1.0.0",
        },
      ]);
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

    // refuse to break local package's version
    expect(&effects)
      .to_have_local_instance_mistakenly_mismatches_semver_group(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
        expected: "1.0.0",
      }])
      .to_have_instance_matches_local_but_mismatches_semver_group(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "1.0.0",
        expected: "^1.0.0",
      }]);
  }

  #[test]
  #[ignore]
  fn reports_local_version_mismatch_when_an_instance_uses_workspace_protocol() {
    panic!("@TODO");
  }

  #[test]
  #[ignore]
  fn reports_unfixable_local_version_mismatch_when_local_version_is_missing() {
    panic!("@TODO");
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

  #[test]
  fn highest_version_match_becomes_mismatch_after_semver_range_has_been_fixed() {
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
      .to_have_instance_mismatches_highest_or_lowest_semver(vec![ExpectedMismatchEvent {
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: "1.0.0",
        expected: ">1.0.0",
      }])
      .to_have_instance_matches_highest_or_lowest_semver_but_mismatches_semver_group(vec![
        ExpectedMismatchEvent {
          dependency_name: "foo",
          instance_id: "foo in /devDependencies of package-a",
          actual: "1.0.0",
          expected: ">1.0.0",
        },
      ]);
  }
}
