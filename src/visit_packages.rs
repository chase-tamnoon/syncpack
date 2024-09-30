#[cfg(test)]
#[path = "visit_packages_test.rs"]
mod visit_packages_test;

use itertools::Itertools;
use std::{cmp::Ordering, collections::HashMap, rc::Rc};

use crate::{
  config::Config,
  context::Context,
  effects::{Effects, Event, FormatEvent, FormatEventVariant, InstanceEvent, InstanceEventVariant, PackageFormatEvent},
  format,
  packages::Packages,
  specifier::Specifier,
  version_group::Variant,
};

pub fn visit_packages(config: &Config, packages: &Packages, effects: &mut impl Effects) {
  const VALID: u8 = 0;
  const WARNING: u8 = 1;
  const INVALID: u8 = 2;

  let Context {
    instances_by_id,
    semver_groups,
    version_groups,
  } = Context::create(config, packages);

  let local_specifiers_by_name: HashMap<String, Specifier> = packages
    .by_name
    .iter()
    .map(|(name, package_json)| {
      (
        name.clone(),
        package_json
          .get_string("/version")
          .map(|string| Specifier::new(&string))
          .unwrap_or(Specifier::None),
      )
    })
    .collect();

  if config.cli.options.versions {
    effects.on(Event::EnterVersionsAndRanges);

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
        effects.on(Event::GroupVisited(&group.selector));

        group.dependencies.borrow().values().for_each(|dependency| {
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
              dependency.all.borrow().iter().for_each(|instance| {
                if instance.is_local {
                  mark_as(WARNING);
                  queue.push(InstanceEvent {
                    dependency,
                    instance: Rc::clone(instance),
                    variant: InstanceEventVariant::LocalInstanceMistakenlyBanned,
                  });
                } else {
                  mark_as(INVALID);
                  *instance.expected.borrow_mut() = Specifier::None;
                  queue.push(InstanceEvent {
                    dependency,
                    instance: Rc::clone(instance),
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
                  dependency.all.borrow().iter().for_each(|instance| {
                    if instance.is_local {
                      if instance.has_range_mismatch(local_specifier) {
                        mark_as(WARNING);
                        expected = Some(local_specifier.clone());
                       *instance.expected.borrow_mut() = local_specifier.clone();
                        queue.push(InstanceEvent {
                          dependency,
                          instance: Rc::clone(instance),
                          variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup,
                        });
                      } else {
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency,
                          instance: Rc::clone(instance),
                          variant: InstanceEventVariant::LocalInstanceIsPreferred,
                        });
                      }
                    } else if matches!(local_specifier, Specifier::None) {
                      mark_as(INVALID);
                     *instance.expected.borrow_mut() = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::InstanceMismatchesLocalWithMissingVersion,
                      });
                    } else if instance.actual == *local_specifier {
                      if instance.has_range_mismatch(local_specifier) {
                        mark_as(INVALID);
                       *instance.expected.borrow_mut() = instance.get_fixed_range_mismatch();
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency,
                          instance: Rc::clone(instance),
                          variant: InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup,
                        });
                      } else {
                        expected = Some(local_specifier.clone());
                        queue.push(InstanceEvent {
                          dependency,
                          instance: Rc::clone(instance),
                          variant: InstanceEventVariant::InstanceMatchesLocal,
                        });
                      }
                    } else {
                      mark_as(INVALID);
                     *instance.expected.borrow_mut() = local_specifier.clone();
                      expected = Some(local_specifier.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::InstanceMismatchesLocal,
                      });
                    }
                  });
                }
                None => {
                  if dependency.all_are_semver() {
                    match dependency.get_highest_or_lowest_semver( preferred_order) {
                      Some(preferred) => {
                        dependency.all.borrow().iter().for_each(|instance| {
                          if instance.actual == preferred {
                            if instance.has_range_mismatch(&preferred) {
                              mark_as(INVALID);
                             *instance.expected.borrow_mut() = instance.get_fixed_range_mismatch();
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency,
                                instance: Rc::clone(instance),
                                variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup,
                              });
                            } else {
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency,
                                instance: Rc::clone(instance),
                                variant: InstanceEventVariant::InstanceMatchesHighestOrLowestSemver,
                              });
                            }
                          } else if *instance.expected.borrow() == preferred {
                            if instance.matches_semver_group(&instance.expected.borrow()) && !instance.matches_semver_group(&instance.actual) {
                              mark_as(INVALID);
                              expected = Some(preferred.clone());
                              queue.push(InstanceEvent {
                                dependency,
                                instance: Rc::clone(instance),
                                variant: InstanceEventVariant::InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed,
                              });
                            }
                          } else {
                            // check this
                            mark_as(INVALID);
                           *instance.expected.borrow_mut() = preferred.clone();
                            expected = Some(preferred.clone());
                            queue.push(InstanceEvent {
                              dependency,
                              instance: Rc::clone(instance),
                              variant: InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver,
                            });
                          }
                        });
                      }
                      None => {
                        panic!("No {} semver found for dependency {:?}", label, dependency);
                      }
                    }
                  } else if dependency.all_are_identical() {
                    mark_as(WARNING);
                    dependency.all.borrow().iter().for_each(|instance| {
                      expected = Some(instance.actual.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::InstanceMatchesButIsUnsupported,
                      });
                    });
                  } else {
                    mark_as(INVALID);
                    dependency.all.borrow().iter().for_each(|instance| {
                     *instance.expected.borrow_mut() = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::InstanceMismatchesAndIsUnsupported,
                      });
                    });
                  }
                }
              }
            }
            Variant::Ignored => {
              dependency.all.borrow().iter().for_each(|instance| {
                queue.push(InstanceEvent {
                  dependency,
                  instance: Rc::clone(instance),
                  variant: InstanceEventVariant::InstanceIsIgnored,
                });
              });
            }
            Variant::Pinned => match &dependency.pinned_specifier {
              Some(pinned) => {
                dependency.all.borrow().iter().for_each(|instance| {
                  // CHECK THIS Eq WORKS
                  if instance.actual == *pinned {
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency,
                      instance: Rc::clone(instance),
                      variant: InstanceEventVariant::InstanceMatchesPinned,
                    });
                  } else if instance.has_range_mismatch(pinned) {
                    if instance.is_local {
                      mark_as(WARNING);
                      expected = Some(pinned.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup,
                      });
                    } else {
                      mark_as(INVALID);
                     *instance.expected.borrow_mut() = instance.get_fixed_range_mismatch();
                      expected = Some(pinned.clone());
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup,
                      });
                    }
                  } else if instance.is_local {
                    mark_as(WARNING);
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency,
                      instance: Rc::clone(instance),
                      variant: InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned,
                    });
                  } else {
                    mark_as(INVALID);
                   *instance.expected.borrow_mut() = pinned.clone();
                    expected = Some(pinned.clone());
                    queue.push(InstanceEvent {
                      dependency,
                      instance: Rc::clone(instance),
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
              if dependency.all_are_semver() {
                let mismatches = dependency.get_same_range_mismatches();
                dependency.all.borrow().iter().for_each(|instance| {
                  // CHECK THIS OVER
                  if instance.has_range_mismatch(&instance.expected.borrow()) {
                    if mismatches.contains_key(&instance.actual) {
                      if mismatches.contains_key(&*instance.expected.borrow()) {
                        mark_as(INVALID);
                       *instance.expected.borrow_mut() = Specifier::None;
                        queue.push(InstanceEvent {
                          dependency,
                          instance: Rc::clone(instance),
                          variant: InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups,
                        });
                      } else {
                        mark_as(INVALID);
                       *instance.expected.borrow_mut() = Specifier::None;
                        queue.push(InstanceEvent {
                          dependency,
                          instance: Rc::clone(instance),
                          variant: InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups,
                        });
                      }
                    } else if mismatches.contains_key(&*instance.expected.borrow()) {
                      mark_as(INVALID);
                     *instance.expected.borrow_mut() = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup,
                      });
                    } else {
                      mark_as(INVALID);
                     *instance.expected.borrow_mut() = Specifier::None;
                      queue.push(InstanceEvent {
                        dependency,
                        instance: Rc::clone(instance),
                        variant: InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup,
                      });
                    }
                  } else if mismatches.contains_key(&instance.actual) {
                    mark_as(INVALID);
                   *instance.expected.borrow_mut() = Specifier::None;
                    queue.push(InstanceEvent {
                      dependency,
                      instance: Rc::clone(instance),
                      variant: InstanceEventVariant::InstanceMismatchesSameRangeGroup,
                    });
                  } else {
                    queue.push(InstanceEvent {
                      dependency,
                      instance: Rc::clone(instance),
                      variant: InstanceEventVariant::InstanceMatchesSameRangeGroup,
                    });
                  }
                  // /CHECK THIS OVER
                });
              } else if dependency.all_are_identical() {
                mark_as(WARNING);
                dependency.all.borrow().iter().for_each(|instance| {
                  queue.push(InstanceEvent {
                    dependency,
                    instance: Rc::clone(instance),
                    variant: InstanceEventVariant::InstanceMatchesButIsUnsupported,
                  });
                });
              } else {
                mark_as(INVALID);
                dependency.all.borrow().iter().for_each(|instance| {
                 *instance.expected.borrow_mut() = Specifier::None;
                  queue.push(InstanceEvent {
                    dependency,
                    instance: Rc::clone(instance),
                    variant: InstanceEventVariant::InstanceMismatchesAndIsUnsupported,
                  });
                });
              }
            }
            Variant::SnappedTo => {
              let snapped_to_specifier = dependency.get_snapped_to_specifier();
              // @FIXME
              expected = Some(Specifier::new("0.0.0"));
            }
          };

          if severity == VALID {
            effects.on(Event::DependencyValid(dependency, expected));
          } else if severity == WARNING {
            effects.on(Event::DependencyWarning(dependency, expected));
          } else {
            effects.on(Event::DependencyInvalid(dependency, expected));
          }

          // Sort instances by actual specifier and then package name
          queue.sort_by(|a, b| {
            let a = &a.instance;
            let b = &b.instance;

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
            effects.on_instance(event);
          }
        });
      });
  }

  if config.cli.options.format {
    effects.on(Event::EnterFormat);

    packages.by_name.values().for_each(|package| {
      let mut formatting_mismatches: Vec<FormatEvent> = Vec::new();
      if config.rcfile.format_bugs {
        if let Some(expected) = format::get_formatted_bugs(package) {
          formatting_mismatches.push(FormatEvent {
            expected,
            package_name: package.get_name_unsafe(),
            property_path: "/bugs".to_string(),
            variant: FormatEventVariant::BugsPropertyIsNotFormatted,
          });
        }
      }
      if config.rcfile.format_repository {
        if let Some(expected) = format::get_formatted_repository(package) {
          formatting_mismatches.push(FormatEvent {
            expected,
            package_name: package.get_name_unsafe(),
            property_path: "/repository".to_string(),
            variant: FormatEventVariant::RepositoryPropertyIsNotFormatted,
          });
        }
      }
      if !config.rcfile.sort_exports.is_empty() {
        if let Some(expected) = format::get_sorted_exports(&config.rcfile, package) {
          formatting_mismatches.push(FormatEvent {
            expected,
            package_name: package.get_name_unsafe(),
            property_path: "/exports".to_string(),
            variant: FormatEventVariant::ExportsPropertyIsNotSorted,
          });
        }
      }
      if !config.rcfile.sort_az.is_empty() {
        for key in config.rcfile.sort_az.iter() {
          if let Some(expected) = format::get_sorted_az(key, package) {
            formatting_mismatches.push(FormatEvent {
              expected,
              package_name: package.get_name_unsafe(),
              property_path: format!("/{}", key),
              variant: FormatEventVariant::PropertyIsNotSortedAz,
            });
          }
        }
      }
      if config.rcfile.sort_packages || !config.rcfile.sort_first.is_empty() {
        if let Some(expected) = format::get_sorted_first(&config.rcfile, package) {
          formatting_mismatches.push(FormatEvent {
            expected,
            package_name: package.get_name_unsafe(),
            property_path: "/".to_string(),
            variant: FormatEventVariant::PackagePropertiesAreNotSorted,
          });
        }
      }
      effects.on(if formatting_mismatches.is_empty() {
        Event::PackageFormatMatch(package.get_name_unsafe())
      } else {
        Event::PackageFormatMismatch(PackageFormatEvent {
          package_name: package.get_name_unsafe(),
          formatting_mismatches,
        })
      });
    });
  }

  effects.on(Event::ExitCommand);
}
