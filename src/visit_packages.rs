#[cfg(test)]
#[path = "visit_packages_test.rs"]
mod visit_packages_test;

use itertools::Itertools;
use log::debug;
use std::{cmp::Ordering, rc::Rc};

use crate::{
  config::Config,
  context::Context,
  dependency::DependencyState::*,
  effects::{Effects, Event, FormatMismatch, FormatMismatchEvent, FormatMismatchVariant, InstanceEvent, InstanceState::*},
  format,
  packages::Packages,
  specifier::{semver_range::SemverRange, Specifier},
  version_group::Variant,
};

pub fn visit_packages(config: &Config, packages: &Packages, effects: &mut impl Effects) {
  let ctx = Context::create(config, packages);

  if config.cli.options.versions {
    debug!("visit versions");
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
              debug!("visit banned version group");
              debug!("  visit dependency '{}'", dependency.name);
              dependency.all_instances.borrow().iter().for_each(|instance| {
                let actual_specifier = &instance.actual_specifier;
                debug!("    visit instance '{}' ({actual_specifier:?})", instance.id);
                if instance.is_local {
                  debug!("      it is the local instance of a package developed locally in this monorepo");
                  debug!("        refuse to change it");
                  debug!("          mark as warning, user should change their config");
                  dependency.set_state(Warning);
                  instance.set_state(RefuseToBanLocal, &instance.actual_specifier);
                } else {
                  debug!("      it should be removed");
                  debug!("        mark as error");
                  dependency.set_state(Invalid);
                  instance.set_state(Banned, &Specifier::None);
                }
              });
            }
            Variant::HighestSemver | Variant::LowestSemver => {
              debug!("visit standard version group");
              debug!("  visit dependency '{}'", dependency.name);
              if dependency.has_local_instance_with_invalid_specifier() {
                debug!("    it has an invalid local instance");
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  let actual_specifier = &instance.actual_specifier;
                  debug!("      visit instance '{}' ({actual_specifier:?})", instance.id);
                  if instance.is_local {
                    debug!("        it is the invalid local instance");
                    debug!("          mark as warning");
                    dependency.set_state(Warning);
                    instance.set_state(InvalidLocalVersion, &instance.actual_specifier);
                  } else {
                    debug!("        it depends on an unknowable version of an invalid local instance");
                    debug!("          mark as error");
                    dependency.set_state(Invalid);
                    instance.set_state(MismatchesInvalidLocalVersion, &instance.actual_specifier);
                  }
                });
              } else if dependency.has_local_instance() {
                debug!("    it is a package developed locally in this monorepo");
                let local_specifier = dependency.get_local_specifier().unwrap();
                dependency.set_expected_specifier(&local_specifier);
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  let actual_specifier = &instance.actual_specifier;
                  debug!("      visit instance '{}' ({actual_specifier:?})", instance.id);
                  if instance.is_local {
                    debug!("        it is the valid local instance");
                    dependency.set_state(Valid);
                    instance.set_state(ValidLocal, &local_specifier);
                    return;
                  }
                  debug!("        it depends on the local instance");
                  debug!("          its version number (without a range):");
                  if !instance.actual_specifier.has_same_version_number_as(&local_specifier) {
                    debug!("            differs to the local instance");
                    debug!("              mark as error");
                    dependency.set_state(Invalid);
                    instance.set_state(MismatchesLocal, &local_specifier);
                    return;
                  }
                  debug!("            is the same as the local instance");
                  if instance.must_match_preferred_semver_range_which_is_not(&SemverRange::Exact) {
                    let preferred_semver_range = &instance.preferred_semver_range.borrow().clone().unwrap();
                    debug!("              it is in a semver group which prefers a different semver range to the local instance ({preferred_semver_range:?})");
                    if instance.matches_preferred_semver_range() {
                      debug!("                its semver range matches its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&local_specifier) {
                        debug!("                  the semver range satisfies the local version");
                        debug!("                    mark as warning (the config is asking for an inexact match)");
                        dependency.set_state(Warning);
                        instance.set_state(MatchesLocal, &instance.get_specifier_with_preferred_semver_range().unwrap());
                      } else {
                        debug!("                  the preferred semver range will not satisfy the local version");
                        debug!("                    mark as unfixable error");
                        dependency.set_state(Invalid);
                        instance.set_state(SemverRangeMatchConflictsWithLocalVersion, &instance.actual_specifier);
                      }
                    } else {
                      debug!("                its semver range does not match its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&local_specifier) {
                        debug!("                  the preferred semver range will satisfy the local version");
                        debug!("                    mark as fixable error");
                        dependency.set_state(Invalid);
                        instance.set_state(SemverRangeMismatch, &instance.get_specifier_with_preferred_semver_range().unwrap());
                      } else {
                        debug!("                  the preferred semver range will not satisfy the local version");
                        debug!("                    mark as unfixable error");
                        dependency.set_state(Invalid);
                        instance.set_state(SemverRangeMismatchConflictsWithLocalVersion, &instance.actual_specifier);
                      }
                    }
                    return;
                  }
                  debug!("              it is not in a semver group which prefers a different semver range to the local instance");
                  if instance.already_equals(&local_specifier) {
                    debug!("                its semver range matches the local instance");
                    debug!("                  mark as valid");
                    dependency.set_state(Valid);
                    instance.set_state(EqualsLocal, &local_specifier);
                  } else {
                    debug!("                its semver range differs to the local instance");
                    debug!("                  mark as error");
                    dependency.set_state(Invalid);
                    instance.set_state(MismatchesLocal, &local_specifier);
                  }
                });
              } else if let Some(highest_specifier) = dependency.get_highest_or_lowest_specifier() {
                debug!("    a highest semver version was found ({highest_specifier:?})");
                dependency.set_expected_specifier(&highest_specifier);
                dependency.all_instances.borrow().iter().for_each(|instance| {
                  let actual_specifier = &instance.actual_specifier;
                  debug!("      visit instance '{}' ({actual_specifier:?})", instance.id);
                  debug!("        its version number (without a range):");
                  if !instance.actual_specifier.has_same_version_number_as(&highest_specifier) {
                    debug!("          differs to the highest semver version");
                    debug!("            mark as error");
                    dependency.set_state(Invalid);
                    instance.set_state(MismatchesPreferVersion, &highest_specifier);
                    return;
                  }
                  debug!("          is the same as the highest semver version");
                  let range_of_highest_specifier = highest_specifier.get_simple_semver().unwrap().get_range();
                  if instance.must_match_preferred_semver_range_which_is_not(&range_of_highest_specifier) {
                    let preferred_semver_range = &instance.preferred_semver_range.borrow().clone().unwrap();
                    debug!("            it is in a semver group which prefers a different semver range to the highest semver version ({preferred_semver_range:?})");
                    if instance.matches_preferred_semver_range() {
                      debug!("              its semver range matches its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&highest_specifier) {
                        debug!("                the semver range satisfies the highest semver version");
                        debug!("                  mark as warning (the config is asking for an inexact match)");
                        dependency.set_state(Warning);
                        instance.set_state(MatchesPreferVersion, &instance.actual_specifier);
                      } else {
                        debug!("                the preferred semver range will not satisfy the highest semver version");
                        debug!("                  mark as unfixable error");
                        dependency.set_state(Invalid);
                        instance.set_state(SemverRangeMatchConflictsWithPreferVersion, &instance.actual_specifier);
                      }
                    } else {
                      debug!("              its semver range does not match its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&highest_specifier) {
                        debug!("                the preferred semver range will satisfy the highest semver version");
                        debug!("                  mark as fixable error");
                        dependency.set_state(Invalid);
                        instance.set_state(SemverRangeMismatch, &instance.get_specifier_with_preferred_semver_range().unwrap());
                      } else {
                        debug!("                the preferred semver range will not satisfy the highest semver version");
                        debug!("                  mark as unfixable error");
                        dependency.set_state(Invalid);
                        instance.set_state(SemverRangeMismatchConflictsWithPreferVersion, &instance.actual_specifier);
                      }
                    }
                  } else {
                    debug!("        it is not in a semver group which prefers a different semver range to the highest semver version");
                    if instance.already_equals(&highest_specifier) {
                      debug!("          it is identical to the highest semver version");
                      debug!("            mark as valid");
                      dependency.set_state(Valid);
                      instance.set_state(EqualsPreferVersion, &highest_specifier);
                    } else {
                      debug!("          it is different to the highest semver version");
                      debug!("            mark as error");
                      dependency.set_state(Invalid);
                      instance.set_state(MismatchesPreferVersion, &highest_specifier);
                    }
                  }
                });
              } else {
                debug!("    no instances have a semver version");
                if dependency.every_specifier_is_already_identical() {
                  debug!("      but all are identical");
                  dependency.set_state(Valid);
                  dependency.all_instances.borrow().iter().for_each(|instance| {
                    let actual_specifier = &instance.actual_specifier;
                    debug!("        visit instance '{}' ({actual_specifier:?})", instance.id);
                    debug!("          it is identical to every other instance");
                    debug!("            mark as valid");
                    instance.set_state(EqualsNonSemverPreferVersion, &instance.actual_specifier);
                  });
                } else {
                  debug!("      and they differ");
                  dependency.set_state(Invalid);
                  dependency.all_instances.borrow().iter().for_each(|instance| {
                    let actual_specifier = &instance.actual_specifier;
                    debug!("        visit instance '{}' ({actual_specifier:?})", instance.id);
                    debug!("          it depends on a currently unknowable correct version from a set of unsupported version specifiers");
                    debug!("            mark as error");
                    instance.set_state(MismatchesNonSemverPreferVersion, &instance.actual_specifier);
                  });
                }
              }
            }
            Variant::Ignored => {
              debug!("visit ignored version group");
              debug!("  visit dependency '{}'", dependency.name);
              dependency.set_state(Valid);
              dependency.all_instances.borrow().iter().for_each(|instance| {
                let actual_specifier = &instance.actual_specifier;
                debug!("    visit instance '{}' ({actual_specifier:?})", instance.id);
                instance.set_state(Ignored, &instance.actual_specifier);
              });
            }
            Variant::Pinned => {
              debug!("visit pinned version group");
              debug!("  visit dependency '{}'", dependency.name);
              let pinned_specifier = dependency.pinned_specifier.clone().unwrap();
              dependency.set_expected_specifier(&pinned_specifier);
              dependency.all_instances.borrow().iter().for_each(|instance| {
                let actual_specifier = &instance.actual_specifier;
                debug!("    visit instance '{}' ({actual_specifier:?})", instance.id);
                if instance.is_local {
                  debug!("      it is the local instance of a package developed locally in this monorepo");
                  debug!("        refuse to change it");
                  debug!("          mark as error, user should change their config");
                  dependency.set_state(Warning);
                  instance.set_state(RefuseToPinLocal, &instance.actual_specifier);
                  return;
                }
                debug!("      it depends on the local instance");
                debug!("        its version number (without a range):");
                if !instance.actual_specifier.has_same_version_number_as(&pinned_specifier) {
                  debug!("          differs to the pinned version");
                  debug!("            mark as error");
                  dependency.set_state(Invalid);
                  instance.set_state(MismatchesPin, &pinned_specifier);
                  return;
                }
                debug!("          is the same as the pinned version");
                if instance.must_match_preferred_semver_range_which_differs_to(&pinned_specifier) {
                  let preferred_semver_range = &instance.preferred_semver_range.borrow().clone().unwrap();
                  debug!("            it is in a semver group which prefers a different semver range to the pinned version ({preferred_semver_range:?})");
                  if instance.matches_preferred_semver_range() {
                    debug!("              its semver range matches its semver group");
                    debug!("                1. pin it and ignore the semver group");
                    debug!("                2. mark as warning (the config is asking for a different range AND they want to pin it)");
                    dependency.set_state(Warning);
                    instance.set_state(PinMatchOverridesSemverRangeMatch, &pinned_specifier);
                  } else {
                    debug!("              its semver range does not match its semver group or the pinned version's");
                    debug!("                1. pin it and ignore the semver group");
                    debug!("                2. mark as warning (the config is asking for a different range AND they want to pin it)");
                    dependency.set_state(Warning);
                    instance.set_state(PinMatchOverridesSemverRangeMismatch, &pinned_specifier);
                  }
                  return;
                }
                debug!("            it is not in a semver group which prefers a different semver range to the pinned version");
                if instance.already_equals(&pinned_specifier) {
                  debug!("              it is identical to the pinned version");
                  debug!("                mark as valid");
                  dependency.set_state(Valid);
                  instance.set_state(EqualsPin, &pinned_specifier);
                } else {
                  debug!("              it differs to the pinned version");
                  debug!("                mark as error");
                  dependency.set_state(Invalid);
                  instance.set_state(MismatchesPin, &pinned_specifier);
                }
              });
            }
            Variant::SameRange => {
              debug!("visit same range version group");
              debug!("  visit dependency '{}'", dependency.name);
              dependency.all_instances.borrow().iter().for_each(|instance| {
                // @TODO: instance.
                // gets a node_semver::Range and calls allows_all against another node_semver::Range
              });

              // if dependency.all_are_simple_semver() {
              //   let mismatches = dependency.get_same_range_mismatches();
              //   dependency.all_instances.borrow().iter().for_each(|instance| {
              //     // CHECK THIS OVER
              //     if instance.has_range_mismatch(&instance.expected_specifier.borrow()) {
              //       if mismatches.contains_key(&instance.actual_specifier) {
              //         if mismatches.contains_key(&*instance.expected_specifier.borrow()) {
              //           dependency.set_state(Invalid);
              //           instance
              //             .set_state(SemverRangeMismatchWontFixSameRangeGroup)
              //             .set_expected_specifier(&Specifier::None);
              //         } else {
              //           dependency.set_state(Invalid);
              //           instance
              //             .set_state(SemverRangeMismatchWillFixSameRangeGroup)
              //             .set_expected_specifier(&Specifier::None);
              //         }
              //       } else if mismatches.contains_key(&*instance.expected_specifier.borrow()) {
              //         dependency.set_state(Invalid);
              //         instance
              //           .set_state(SameRangeMatchConflictsWithSemverGroup)
              //           .set_expected_specifier(&Specifier::None);
              //       } else {
              //         dependency.set_state(Invalid);
              //         instance
              //           .set_state(SemverRangeMismatchWillMatchSameRangeGroup)
              //           .set_expected_specifier(&Specifier::None);
              //       }
              //     } else if mismatches.contains_key(&instance.actual_specifier) {
              //       dependency.set_state(Invalid);
              //       instance
              //         .set_state(MismatchesSameRangeGroup)
              //         .set_expected_specifier(&Specifier::None);
              //     } else {
              //       instance.set_state(MatchesSameRangeGroup);
              //     }
              //     // /CHECK THIS OVER
              //   });
              // } else if dependency.every_specifier_is_already_identical() {
              //   dependency.set_state(Warning);
              //   dependency.all_instances.borrow().iter().for_each(|instance| {
              //     instance.set_state(EqualsNonSemverPreferVersion);
              //   });
              // } else {
              //   dependency.set_state(Invalid);
              //   dependency.all_instances.borrow().iter().for_each(|instance| {
              //     instance
              //       .set_state(MismatchesNonSemverPreferVersion)
              //       .set_expected_specifier(&Specifier::None);
              //   });
              // }
            }
            Variant::SnappedTo => {
              debug!("visit snapped to version group");
              debug!("  visit dependency '{}'", dependency.name);
              let snapped_to_specifier = dependency.get_snapped_to_specifier();
              // @FIXME
              dependency.set_expected_specifier(&Specifier::new("0.0.0"));
            }
          };

          // @TODO: this can be one event
          if dependency.has_state(Valid) {
            effects.on(Event::DependencyValid(dependency));
          } else if dependency.has_state(Warning) {
            effects.on(Event::DependencyWarning(dependency));
          } else {
            effects.on(Event::DependencyInvalid(dependency));
          }

          dependency.sort_instances();

          dependency.all_instances.borrow().iter().for_each(|instance| {
            // @TODO: remove InstanceEvent and just emit the instance
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
