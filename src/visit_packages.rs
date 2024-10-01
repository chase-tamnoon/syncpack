#[cfg(test)]
#[path = "visit_packages_test.rs"]
mod visit_packages_test;

use itertools::Itertools;
use std::{cmp::Ordering, rc::Rc};

use crate::{
  config::Config,
  context::Context,
  dependency::DependencyState,
  effects::{Effects, Event, FormatMismatch, FormatMismatchEvent, FormatMismatchVariant, InstanceEvent, InstanceState},
  format,
  packages::Packages,
  specifier::Specifier,
  version_group::Variant,
};

pub fn visit_packages(config: &Config, packages: &Packages, effects: &mut impl Effects) {
  let ctx = Context::create(config, packages);

  if config.cli.options.versions {
    effects.on(Event::EnterVersionsAndRanges);

    ctx
      .version_groups
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
          match dependency.variant {
            Variant::Banned => {
              dependency.all_instances.borrow().iter().for_each(|instance| {
                if instance.is_local {
                  dependency.set_state(DependencyState::Warning);
                  instance.set_state(InstanceState::RefuseToBanLocal);
                } else {
                  dependency.set_state(DependencyState::Invalid);
                  instance.set_expected_specifier(&Specifier::None);
                  instance.set_state(InstanceState::Banned);
                }
              });
            }
            Variant::HighestSemver | Variant::LowestSemver => {
              let prefer_highest = matches!(dependency.variant, Variant::HighestSemver);
              let preferred_order: Ordering = if prefer_highest { Ordering::Greater } else { Ordering::Less };
              let label: &str = if prefer_highest { "highest" } else { "lowest" };

              if dependency.has_local_instance() {
                let local_specifier = dependency.get_local_specifier().unwrap();
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  if instance.is_local {
                    if matches!(local_specifier, Specifier::None) {
                      dependency.set_state(DependencyState::Warning);
                      instance.set_state(InstanceState::MissingLocalVersion);
                    } else if instance.has_range_mismatch(&local_specifier) {
                      dependency.set_state(DependencyState::Warning);
                      dependency.set_expected_specifier(&local_specifier);
                      instance.set_expected_specifier(&local_specifier);
                      instance.set_state(InstanceState::RefuseToChangeLocalSemverRange);
                    } else {
                      dependency.set_expected_specifier(&local_specifier);
                      instance.set_state(InstanceState::LocalWithValidVersion);
                    }
                  } else if matches!(local_specifier, Specifier::None) {
                    dependency.set_state(DependencyState::Invalid);
                    instance.set_expected_specifier(&Specifier::None);
                    instance.set_state(InstanceState::MismatchesMissingLocalVersion);
                  } else if instance.already_matches(&local_specifier) {
                    if instance.has_range_mismatch(&local_specifier) {
                      dependency.set_state(DependencyState::Invalid);
                      instance.set_expected_specifier(&instance.get_fixed_range_mismatch());
                      dependency.set_expected_specifier(&local_specifier);
                      instance.set_state(InstanceState::LocalMatchConflictsWithSemverGroup);
                    } else {
                      dependency.set_expected_specifier(&local_specifier);
                      instance.set_state(InstanceState::MatchesLocal);
                    }
                  } else {
                    dependency.set_state(DependencyState::Invalid);
                    instance.set_expected_specifier(&local_specifier);
                    dependency.set_expected_specifier(&local_specifier);
                    instance.set_state(InstanceState::MismatchesLocal);
                  }
                });
              } else if dependency.all_are_semver() {
                match dependency.get_preferred_specifier(preferred_order) {
                  Some(preferred) => {
                    dependency.all_instances.borrow().iter().for_each(|instance| {
                      if instance.already_matches(&preferred) {
                        if instance.has_range_mismatch(&preferred) {
                          dependency.set_state(DependencyState::Invalid);
                          instance.set_expected_specifier(&instance.get_fixed_range_mismatch());
                          dependency.set_expected_specifier(&preferred);
                          instance.set_state(InstanceState::PreferVersionMatchConflictsWithSemverGroup);
                        } else {
                          dependency.set_expected_specifier(&preferred);
                          instance.set_state(InstanceState::MatchesPreferVersion);
                        }
                      } else if *instance.expected.borrow() == preferred {
                        if instance.matches_semver_group(&instance.expected.borrow()) && !instance.matches_semver_group(&instance.actual) {
                          dependency.set_state(DependencyState::Invalid);
                          dependency.set_expected_specifier(&preferred);
                          instance.set_state(InstanceState::SemverRangeMismatchWillFixPreferVersion);
                        }
                      } else {
                        // check this
                        dependency.set_state(DependencyState::Invalid);
                        instance.set_expected_specifier(&preferred);
                        dependency.set_expected_specifier(&preferred);
                        instance.set_state(InstanceState::MismatchesPreferVersion);
                      }
                    });
                  }
                  None => {
                    panic!("No {} semver found for dependency {:?}", label, dependency);
                  }
                }
              } else if dependency.all_are_identical() {
                dependency.set_state(DependencyState::Warning);
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  dependency.set_expected_specifier(&instance.actual);
                  instance.set_state(InstanceState::MatchesButUnsupported);
                });
              } else {
                dependency.set_state(DependencyState::Invalid);
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  instance.set_expected_specifier(&Specifier::None);
                  instance.set_state(InstanceState::MismatchesUnsupported);
                });
              }
            }
            Variant::Ignored => {
              dependency.all_instances.borrow().iter().for_each(|instance| {
                instance.set_state(InstanceState::MatchesIgnored);
              });
            }
            Variant::Pinned => match &dependency.pinned_specifier {
              Some(pinned) => {
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  // CHECK THIS Eq WORKS
                  if instance.already_matches(pinned) {
                    dependency.set_expected_specifier(pinned);
                    instance.set_state(InstanceState::MatchesPin);
                  } else if instance.has_range_mismatch(pinned) {
                    if instance.is_local {
                      dependency.set_state(DependencyState::Warning);
                      dependency.set_expected_specifier(pinned);
                      instance.set_state(InstanceState::RefuseToChangeLocalSemverRange);
                    } else {
                      dependency.set_state(DependencyState::Invalid);
                      instance.set_expected_specifier(&instance.get_fixed_range_mismatch());
                      dependency.set_expected_specifier(pinned);
                      instance.set_state(InstanceState::PinMatchConflictsWithSemverGroup);
                    }
                  } else if instance.is_local {
                    dependency.set_state(DependencyState::Warning);
                    dependency.set_expected_specifier(pinned);
                    instance.set_state(InstanceState::RefuseToPinLocal);
                  } else {
                    dependency.set_state(DependencyState::Invalid);
                    instance.set_expected_specifier(pinned);
                    dependency.set_expected_specifier(pinned);
                    instance.set_state(InstanceState::MismatchesPin);
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
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  // CHECK THIS OVER
                  if instance.has_range_mismatch(&instance.expected.borrow()) {
                    if mismatches.contains_key(&instance.actual) {
                      if mismatches.contains_key(&*instance.expected.borrow()) {
                        dependency.set_state(DependencyState::Invalid);
                        instance.set_expected_specifier(&Specifier::None);
                        instance.set_state(InstanceState::SemverRangeMismatchWontFixSameRangeGroup);
                      } else {
                        dependency.set_state(DependencyState::Invalid);
                        instance.set_expected_specifier(&Specifier::None);
                        instance.set_state(InstanceState::SemverRangeMismatchWillFixSameRangeGroup);
                      }
                    } else if mismatches.contains_key(&*instance.expected.borrow()) {
                      dependency.set_state(DependencyState::Invalid);
                      instance.set_expected_specifier(&Specifier::None);
                      instance.set_state(InstanceState::SameRangeMatchConflictsWithSemverGroup);
                    } else {
                      dependency.set_state(DependencyState::Invalid);
                      instance.set_expected_specifier(&Specifier::None);
                      instance.set_state(InstanceState::SemverRangeMismatchWillMatchSameRangeGroup);
                    }
                  } else if mismatches.contains_key(&instance.actual) {
                    dependency.set_state(DependencyState::Invalid);
                    instance.set_expected_specifier(&Specifier::None);
                    instance.set_state(InstanceState::MismatchesSameRangeGroup);
                  } else {
                    instance.set_state(InstanceState::MatchesSameRangeGroup);
                  }
                  // /CHECK THIS OVER
                });
              } else if dependency.all_are_identical() {
                dependency.set_state(DependencyState::Warning);
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  instance.set_state(InstanceState::MatchesButUnsupported);
                });
              } else {
                dependency.set_state(DependencyState::Invalid);
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  instance.set_expected_specifier(&Specifier::None);
                  instance.set_state(InstanceState::MismatchesUnsupported);
                });
              }
            }
            Variant::SnappedTo => {
              let snapped_to_specifier = dependency.get_snapped_to_specifier();
              // @FIXME
              dependency.set_expected_specifier(&Specifier::new("0.0.0"));
            }
          };

          // @TODO: this can be one event
          if dependency.has_state(DependencyState::Valid) {
            effects.on(Event::DependencyValid(dependency));
          } else if dependency.has_state(DependencyState::Warning) {
            effects.on(Event::DependencyWarning(dependency));
          } else {
            effects.on(Event::DependencyInvalid(dependency));
          }

          dependency.sort_instances();

          dependency.all_instances.borrow().iter().for_each(|instance| {
            effects.on_instance(InstanceEvent {
              dependency,
              instance: Rc::clone(instance),
              variant: instance.state.borrow().clone(),
            });
          });
        });
      });
  }

  if config.cli.options.format {
    effects.on(Event::EnterFormat);

    packages.sorted_by_path().for_each(|package| {
      let mut formatting_mismatches: Vec<FormatMismatch> = Vec::new();
      if config.rcfile.format_bugs {
        if let Some(expected) = format::get_formatted_bugs(&package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/bugs".to_string(),
            variant: FormatMismatchVariant::BugsPropertyIsNotFormatted,
          });
        }
      }
      if config.rcfile.format_repository {
        if let Some(expected) = format::get_formatted_repository(&package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/repository".to_string(),
            variant: FormatMismatchVariant::RepositoryPropertyIsNotFormatted,
          });
        }
      }
      if !config.rcfile.sort_exports.is_empty() {
        if let Some(expected) = format::get_sorted_exports(&config.rcfile, &package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/exports".to_string(),
            variant: FormatMismatchVariant::ExportsPropertyIsNotSorted,
          });
        }
      }
      if !config.rcfile.sort_az.is_empty() {
        for key in config.rcfile.sort_az.iter() {
          if let Some(expected) = format::get_sorted_az(key, &package.borrow()) {
            formatting_mismatches.push(FormatMismatch {
              expected,
              package: Rc::clone(package),
              property_path: format!("/{}", key),
              variant: FormatMismatchVariant::PropertyIsNotSortedAz,
            });
          }
        }
      }
      if config.rcfile.sort_packages || !config.rcfile.sort_first.is_empty() {
        if let Some(expected) = format::get_sorted_first(&config.rcfile, &package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/".to_string(),
            variant: FormatMismatchVariant::PackagePropertiesAreNotSorted,
          });
        }
      }
      effects.on(if formatting_mismatches.is_empty() {
        Event::PackageFormatMatch(Rc::clone(package))
      } else {
        Event::PackageFormatMismatch(FormatMismatchEvent {
          package: Rc::clone(package),
          formatting_mismatches,
        })
      });
    });
  }

  effects.on(Event::ExitCommand);
}
