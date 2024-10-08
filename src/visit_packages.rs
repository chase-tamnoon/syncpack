#[cfg(test)]
#[path = "visit_packages_test.rs"]
mod visit_packages_test;

use itertools::Itertools;
use log::debug;
use std::{cmp::Ordering, rc::Rc};

use crate::{
  config::Config,
  context::Context,
  dependency::DependencyState,
  format,
  instance::{FixableInstance::*, SemverGroupAndVersionConflict::*, SuspectInstance::*, UnfixableInstance::*, ValidInstance::*},
  package_json::{FormatMismatch, FormatMismatchVariant::*},
  packages::Packages,
  specifier::{semver_range::SemverRange, Specifier},
  version_group::VersionGroupVariant,
};

pub fn visit_packages(config: Config, packages: Packages) -> Context {
  let ctx = Context::create(config, packages);

  if ctx.config.cli.options.versions {
    debug!("visit versions");

    ctx
      .version_groups
      .iter()
      // @TODO: can moving snapped to groups last be done when reading config at
      // the start?
      //
      // fix snapped to groups last, so that the packages they're snapped to
      // have any fixes applied to them first
      .sorted_by(|a, b| {
        if matches!(a.variant, VersionGroupVariant::SnappedTo) {
          Ordering::Greater
        } else if matches!(b.variant, VersionGroupVariant::SnappedTo) {
          Ordering::Less
        } else {
          Ordering::Equal
        }
      })
      .for_each(|group| {
        group.dependencies.borrow().values().for_each(|dependency| {
          match dependency.variant {
            VersionGroupVariant::Banned => {
              debug!("visit banned version group");
              debug!("  visit dependency '{}'", dependency.name);
              dependency.instances.borrow().iter().for_each(|instance| {
                let actual_specifier = &instance.actual_specifier;
                debug!("    visit instance '{}' ({actual_specifier:?})", instance.id);
                if instance.is_local {
                  debug!("      it is the local instance of a package developed locally in this monorepo");
                  debug!("        refuse to change it");
                  debug!("          mark as suspect, user should change their config");
                  dependency.set_state(DependencyState::Suspect);
                  instance.mark_suspect(RefuseToBanLocal, &instance.actual_specifier);
                } else {
                  debug!("      it should be removed");
                  debug!("        mark as error");
                  dependency.set_state(DependencyState::Invalid);
                  instance.mark_fixable(Banned, &Specifier::None);
                }
              });
            }
            VersionGroupVariant::HighestSemver | VersionGroupVariant::LowestSemver => {
              debug!("visit standard version group");
              debug!("  visit dependency '{}'", dependency.name);
              if dependency.has_local_instance_with_invalid_specifier() {
                debug!("    it has an invalid local instance");
                dependency.instances.borrow().iter().for_each(|instance| {
                  let actual_specifier = &instance.actual_specifier;
                  debug!("      visit instance '{}' ({actual_specifier:?})", instance.id);
                  if instance.is_local {
                    debug!("        it is the invalid local instance");
                    debug!("          mark as suspect");
                    dependency.set_state(DependencyState::Suspect);
                    instance.mark_suspect(InvalidLocalVersion, &instance.actual_specifier);
                  } else {
                    debug!("        it depends on an unknowable version of an invalid local instance");
                    debug!("          mark as error");
                    dependency.set_state(DependencyState::Invalid);
                    instance.mark_unfixable(MismatchesInvalidLocalVersion, &instance.actual_specifier);
                  }
                });
              } else if dependency.has_local_instance() {
                debug!("    it is a package developed locally in this monorepo");
                let local_specifier = dependency.get_local_specifier().unwrap();
                dependency.set_expected_specifier(&local_specifier);
                dependency.instances.borrow().iter().for_each(|instance| {
                  let actual_specifier = &instance.actual_specifier;
                  debug!("      visit instance '{}' ({actual_specifier:?})", instance.id);
                  if instance.is_local {
                    debug!("        it is the valid local instance");
                    dependency.set_state(DependencyState::Valid);
                    instance.mark_valid(ValidLocal, &local_specifier);
                    return;
                  }
                  debug!("        it depends on the local instance");
                  debug!("          its version number (without a range):");
                  if !instance.actual_specifier.has_same_version_number_as(&local_specifier) {
                    debug!("            differs to the local instance");
                    debug!("              mark as error");
                    dependency.set_state(DependencyState::Invalid);
                    instance.mark_fixable(MismatchesLocal, &local_specifier);
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
                        debug!("                    mark as suspect (the config is asking for an inexact match)");
                        dependency.set_state(DependencyState::Suspect);
                        instance.mark_valid(MatchesLocal, &instance.get_specifier_with_preferred_semver_range().unwrap());
                      } else {
                        debug!("                  the preferred semver range will not satisfy the local version");
                        debug!("                    mark as unfixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_conflict(MatchConflictsWithLocal, &instance.actual_specifier);
                      }
                    } else {
                      debug!("                its semver range does not match its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&local_specifier) {
                        debug!("                  the preferred semver range will satisfy the local version");
                        debug!("                    mark as fixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_fixable(SemverRangeMismatch, &instance.get_specifier_with_preferred_semver_range().unwrap());
                      } else {
                        debug!("                  the preferred semver range will not satisfy the local version");
                        debug!("                    mark as unfixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_conflict(MismatchConflictsWithLocal, &instance.actual_specifier);
                      }
                    }
                    return;
                  }
                  debug!("              it is not in a semver group which prefers a different semver range to the local instance");
                  if instance.already_equals(&local_specifier) {
                    debug!("                its semver range matches the local instance");
                    debug!("                  mark as valid");
                    dependency.set_state(DependencyState::Valid);
                    instance.mark_valid(EqualsLocal, &local_specifier);
                  } else {
                    debug!("                its semver range differs to the local instance");
                    debug!("                  mark as error");
                    dependency.set_state(DependencyState::Invalid);
                    instance.mark_fixable(MismatchesLocal, &local_specifier);
                  }
                });
              } else if let Some(highest_specifier) = dependency.get_highest_or_lowest_specifier() {
                debug!("    a highest semver version was found ({highest_specifier:?})");
                dependency.set_expected_specifier(&highest_specifier);
                dependency.instances.borrow().iter().for_each(|instance| {
                  let actual_specifier = &instance.actual_specifier;
                  debug!("      visit instance '{}' ({actual_specifier:?})", instance.id);
                  debug!("        its version number (without a range):");
                  if !instance.actual_specifier.has_same_version_number_as(&highest_specifier) {
                    debug!("          differs to the highest semver version");
                    debug!("            mark as error");
                    dependency.set_state(DependencyState::Invalid);
                    instance.mark_fixable(MismatchesPreferVersion, &highest_specifier);
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
                        debug!("                  mark as suspect (the config is asking for an inexact match)");
                        dependency.set_state(DependencyState::Suspect);
                        instance.mark_valid(MatchesPreferVersion, &instance.actual_specifier);
                      } else {
                        debug!("                the preferred semver range will not satisfy the highest semver version");
                        debug!("                  mark as unfixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_conflict(MatchConflictsWithPrefer, &instance.actual_specifier);
                      }
                    } else {
                      debug!("              its semver range does not match its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&highest_specifier) {
                        debug!("                the preferred semver range will satisfy the highest semver version");
                        debug!("                  mark as fixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_fixable(SemverRangeMismatch, &instance.get_specifier_with_preferred_semver_range().unwrap());
                      } else {
                        debug!("                the preferred semver range will not satisfy the highest semver version");
                        debug!("                  mark as unfixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_conflict(MismatchConflictsWithPrefer, &instance.actual_specifier);
                      }
                    }
                  } else {
                    debug!("        it is not in a semver group which prefers a different semver range to the highest semver version");
                    if instance.already_equals(&highest_specifier) {
                      debug!("          it is identical to the highest semver version");
                      debug!("            mark as valid");
                      dependency.set_state(DependencyState::Valid);
                      instance.mark_valid(EqualsPreferVersion, &highest_specifier);
                    } else {
                      debug!("          it is different to the highest semver version");
                      debug!("            mark as error");
                      dependency.set_state(DependencyState::Invalid);
                      instance.mark_fixable(MismatchesPreferVersion, &highest_specifier);
                    }
                  }
                });
              } else {
                debug!("    no instances have a semver version");
                if dependency.every_specifier_is_already_identical() {
                  debug!("      but all are identical");
                  dependency.set_state(DependencyState::Valid);
                  dependency.instances.borrow().iter().for_each(|instance| {
                    let actual_specifier = &instance.actual_specifier;
                    debug!("        visit instance '{}' ({actual_specifier:?})", instance.id);
                    debug!("          it is identical to every other instance");
                    debug!("            mark as valid");
                    instance.mark_valid(EqualsNonSemverPreferVersion, &instance.actual_specifier);
                  });
                } else {
                  debug!("      and they differ");
                  dependency.set_state(DependencyState::Invalid);
                  dependency.instances.borrow().iter().for_each(|instance| {
                    let actual_specifier = &instance.actual_specifier;
                    debug!("        visit instance '{}' ({actual_specifier:?})", instance.id);
                    debug!("          it depends on a currently unknowable correct version from a set of unsupported version specifiers");
                    debug!("            mark as error");
                    instance.mark_unfixable(MismatchesNonSemverPreferVersion, &instance.actual_specifier);
                  });
                }
              }
            }
            VersionGroupVariant::Ignored => {
              debug!("visit ignored version group");
              debug!("  visit dependency '{}'", dependency.name);
              dependency.set_state(DependencyState::Valid);
              dependency.instances.borrow().iter().for_each(|instance| {
                let actual_specifier = &instance.actual_specifier;
                debug!("    visit instance '{}' ({actual_specifier:?})", instance.id);
                instance.mark_valid(Ignored, &instance.actual_specifier);
              });
            }
            VersionGroupVariant::Pinned => {
              debug!("visit pinned version group");
              debug!("  visit dependency '{}'", dependency.name);
              let pinned_specifier = dependency.pinned_specifier.clone().unwrap();
              dependency.set_expected_specifier(&pinned_specifier);
              dependency.instances.borrow().iter().for_each(|instance| {
                let actual_specifier = &instance.actual_specifier;
                debug!("    visit instance '{}' ({actual_specifier:?})", instance.id);
                if instance.is_local {
                  debug!("      it is the local instance of a package developed locally in this monorepo");
                  debug!("        refuse to change it");
                  debug!("          mark as error, user should change their config");
                  dependency.set_state(DependencyState::Suspect);
                  instance.mark_suspect(RefuseToPinLocal, &instance.actual_specifier);
                  return;
                }
                debug!("      it depends on the local instance");
                debug!("        its version number (without a range):");
                if !instance.actual_specifier.has_same_version_number_as(&pinned_specifier) {
                  debug!("          differs to the pinned version");
                  debug!("            mark as error");
                  dependency.set_state(DependencyState::Invalid);
                  instance.mark_fixable(MismatchesPin, &pinned_specifier);
                  return;
                }
                debug!("          is the same as the pinned version");
                if instance.must_match_preferred_semver_range_which_differs_to(&pinned_specifier) {
                  let preferred_semver_range = &instance.preferred_semver_range.borrow().clone().unwrap();
                  debug!("            it is in a semver group which prefers a different semver range to the pinned version ({preferred_semver_range:?})");
                  if instance.matches_preferred_semver_range() {
                    debug!("              its semver range matches its semver group");
                    debug!("                1. pin it and ignore the semver group");
                    debug!("                2. mark as suspect (the config is asking for a different range AND they want to pin it)");
                    dependency.set_state(DependencyState::Suspect);
                    instance.mark_fixable(PinMatchOverridesSemverRangeMatch, &pinned_specifier);
                  } else {
                    debug!("              its semver range does not match its semver group or the pinned version's");
                    debug!("                1. pin it and ignore the semver group");
                    debug!("                2. mark as suspect (the config is asking for a different range AND they want to pin it)");
                    dependency.set_state(DependencyState::Suspect);
                    instance.mark_fixable(PinMatchOverridesSemverRangeMismatch, &pinned_specifier);
                  }
                  return;
                }
                debug!("            it is not in a semver group which prefers a different semver range to the pinned version");
                if instance.already_equals(&pinned_specifier) {
                  debug!("              it is identical to the pinned version");
                  debug!("                mark as valid");
                  dependency.set_state(DependencyState::Valid);
                  instance.mark_valid(EqualsPin, &pinned_specifier);
                } else {
                  debug!("              it differs to the pinned version");
                  debug!("                mark as error");
                  dependency.set_state(DependencyState::Invalid);
                  instance.mark_fixable(MismatchesPin, &pinned_specifier);
                }
              });
            }
            VersionGroupVariant::SameRange => {
              debug!("visit same range version group");
              debug!("  visit dependency '{}'", dependency.name);
              dependency.instances.borrow().iter().for_each(|instance| {
                let actual_specifier = &instance.actual_specifier;
                debug!("    visit instance '{}' ({actual_specifier:?})", instance.id);
                if instance.already_satisfies_all(&dependency.instances.borrow()) {
                  debug!("      its specifier satisfies all other instances in the group");
                  if instance.must_match_preferred_semver_range() {
                    debug!("        it belongs to a semver group");
                    if instance.matches_preferred_semver_range() {
                      debug!("          its specifier matches its semver group");
                      dependency.set_state(DependencyState::Valid);
                      instance.mark_valid(MatchesSameRangeGroup, actual_specifier);
                    } else {
                      debug!("          its specifier mismatches its semver group");
                      dependency.set_state(DependencyState::Invalid);
                      instance.mark_fixable(SemverRangeMismatch, &instance.get_specifier_with_preferred_semver_range().unwrap());
                    }
                  } else {
                    debug!("        it does not belong to a semver group");
                    dependency.set_state(DependencyState::Valid);
                    instance.mark_valid(MatchesSameRangeGroup, actual_specifier);
                  }
                } else {
                  debug!("      its specifier does not satisfy all other instances in the group");
                  dependency.set_state(DependencyState::Invalid);
                  instance.mark_unfixable(MismatchesSameRangeGroup, actual_specifier);
                }
              });
            }
            VersionGroupVariant::SnappedTo => {
              debug!("visit snapped to version group");
              debug!("  visit dependency '{}'", dependency.name);
              if let Some(snapped_to_specifier) = dependency.get_snapped_to_specifier(&ctx.instances) {
                debug!("    a target version was found ({snapped_to_specifier:?})");
                dependency.set_expected_specifier(&snapped_to_specifier);
                dependency.instances.borrow().iter().for_each(|instance| {
                  let actual_specifier = &instance.actual_specifier;
                  debug!("      visit instance '{}' ({actual_specifier:?})", instance.id);
                  if instance.is_local && !instance.already_equals(&snapped_to_specifier) {
                    debug!("        it is the local instance of a package developed locally in this monorepo");
                    debug!("          refuse to change it");
                    debug!("            mark as error, user should change their config");
                    dependency.set_state(DependencyState::Suspect);
                    instance.mark_suspect(RefuseToSnapLocal, &instance.actual_specifier);
                    return;
                  }
                  debug!("        it is not a local instance of a package developed locally in this monorepo");
                  debug!("          its version number (without a range):");
                  if !instance.actual_specifier.has_same_version_number_as(&snapped_to_specifier) {
                    debug!("          differs to the target version");
                    debug!("            mark as error");
                    dependency.set_state(DependencyState::Invalid);
                    instance.mark_fixable(MismatchesSnapToVersion, &snapped_to_specifier);
                    return;
                  }
                  debug!("          is the same as the target version");
                  let range_of_snapped_to_specifier = snapped_to_specifier.get_simple_semver().unwrap().get_range();
                  if instance.must_match_preferred_semver_range_which_is_not(&range_of_snapped_to_specifier) {
                    let preferred_semver_range = &instance.preferred_semver_range.borrow().clone().unwrap();
                    debug!("            it is in a semver group which prefers a different semver range to the target version ({preferred_semver_range:?})");
                    if instance.matches_preferred_semver_range() {
                      debug!("              its semver range matches its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&snapped_to_specifier) {
                        debug!("                the semver range satisfies the target version");
                        debug!("                  mark as suspect (the config is asking for an inexact match)");
                        dependency.set_state(DependencyState::Suspect);
                        instance.mark_valid(MatchesSnapToVersion, &instance.actual_specifier);
                      } else {
                        debug!("                the preferred semver range will not satisfy the target version");
                        debug!("                  mark as unfixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_conflict(MatchConflictsWithSnapTo, &instance.actual_specifier);
                      }
                    } else {
                      debug!("              its semver range does not match its semver group");
                      if instance.specifier_with_preferred_semver_range_will_satisfy(&snapped_to_specifier) {
                        debug!("                the preferred semver range will satisfy the target version");
                        debug!("                  mark as fixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_fixable(SemverRangeMismatch, &instance.get_specifier_with_preferred_semver_range().unwrap());
                      } else {
                        debug!("                the preferred semver range will not satisfy the target version");
                        debug!("                  mark as unfixable error");
                        dependency.set_state(DependencyState::Invalid);
                        instance.mark_conflict(MismatchConflictsWithSnapTo, &instance.actual_specifier);
                      }
                    }
                  } else {
                    debug!("        it is not in a semver group which prefers a different semver range to the target version");
                    if instance.already_equals(&snapped_to_specifier) {
                      debug!("          it is identical to the target version");
                      debug!("            mark as valid");
                      dependency.set_state(DependencyState::Valid);
                      instance.mark_valid(EqualsSnapToVersion, &snapped_to_specifier);
                    } else {
                      debug!("          it is different to the target version");
                      debug!("            mark as error");
                      dependency.set_state(DependencyState::Invalid);
                      instance.mark_fixable(MismatchesSnapToVersion, &snapped_to_specifier);
                    }
                  }
                });
              } else {
                debug!("    no target version was found");
                dependency.set_state(DependencyState::Suspect);
                dependency.instances.borrow().iter().for_each(|instance| {
                  instance.mark_unfixable(SnapToVersionNotFound, &instance.actual_specifier);
                });
              }
            }
          }
        });
      });
  }

  if ctx.config.cli.options.format {
    ctx.packages.by_name.values().for_each(|package| {
      let mut formatting_mismatches: Vec<FormatMismatch> = Vec::new();
      if ctx.config.rcfile.format_bugs {
        if let Some(expected) = format::get_formatted_bugs(&package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/bugs".to_string(),
            variant: BugsPropertyIsNotFormatted,
          });
        }
      }
      if ctx.config.rcfile.format_repository {
        if let Some(expected) = format::get_formatted_repository(&package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/repository".to_string(),
            variant: RepositoryPropertyIsNotFormatted,
          });
        }
      }
      if !ctx.config.rcfile.sort_exports.is_empty() {
        if let Some(expected) = format::get_sorted_exports(&ctx.config.rcfile, &package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/exports".to_string(),
            variant: ExportsPropertyIsNotSorted,
          });
        }
      }
      if !ctx.config.rcfile.sort_az.is_empty() {
        for key in ctx.config.rcfile.sort_az.iter() {
          if let Some(expected) = format::get_sorted_az(key, &package.borrow()) {
            formatting_mismatches.push(FormatMismatch {
              expected,
              package: Rc::clone(package),
              property_path: format!("/{}", key),
              variant: PropertyIsNotSortedAz,
            });
          }
        }
      }
      if ctx.config.rcfile.sort_packages || !ctx.config.rcfile.sort_first.is_empty() {
        if let Some(expected) = format::get_sorted_first(&ctx.config.rcfile, &package.borrow()) {
          formatting_mismatches.push(FormatMismatch {
            expected,
            package: Rc::clone(package),
            property_path: "/".to_string(),
            variant: PackagePropertiesAreNotSorted,
          });
        }
      }
    });
  }

  ctx
}
