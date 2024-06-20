use itertools::Itertools;
use std::cmp::Ordering;
use version_compare::Cmp;

use crate::{
  config::Config,
  context::Context,
  effects::{Effects, Event},
  format::{self, InMemoryFormattingStatus},
  instance::{Instance, InstanceId},
  packages::Packages,
  version_group::Variant,
};

pub fn lint(config: &Config, packages: Packages, effects: impl Effects) {
  effects.on(Event::PackagesLoaded(&packages));

  let cli = &config.cli;
  let ctx = Context::create(&config, &packages);
  let mut instances_by_id = ctx.instances_by_id;
  let mut packages = packages;

  effects.on(Event::EnterVersionsAndRanges);

  /*
  fn local_instance_is_preferred(effects: &mut impl Effects, instance: Instance) {
    effects.on(Event::LocalInstanceIsPreferred(instance));
  }

  fn instance_matches_local(effects: &mut impl Effects, instance: Instance) {
    effects.on(Event::InstanceMatchesLocal(instance));
  }

  fn instance_matches_highest_or_lowest_semver(effects: &mut impl Effects, instance: Instance) {
    effects.on(Event::InstanceMatchesHighestOrLowestSemver(instance));
  }

  fn instance_matches_but_is_unsupported(effects: &mut impl Effects, instance: Instance) {
    effects.on(Event::InstanceMatchesButIsUnsupported(instance));
  }

  fn instance_is_ignored(effects: &mut impl Effects, instance: Instance) {
    effects.on(Event::InstanceIsIgnored(instance));
  }

  fn instance_matches_pinned(effects: &mut impl Effects, instance: Instance) {
    effects.on(Event::InstanceMatchesPinned(instance));
  }

  /// ✓ Instance matches its same range group
  /// ✓ Instance matches its semver group
  fn instance_matches_same_range_group(effects: &mut impl Effects, instance: Instance) {
    effects.on(Event::InstanceMatchesSameRangeGroup(instance));
  }

  // ===========================================================================

  /// Refuse to change local dependency specifiers
  fn local_instance_mistakenly_banned(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::LocalInstanceMistakenlyBanned(instance, packages));
  }

  fn instance_is_banned(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceIsBanned(instance, packages));
  }

  fn instance_matches_local_but_mismatches_semver_group(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceMatchesLocalButMismatchesSemverGroup(instance, packages));
  }

  fn instance_mismatches_local(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceMismatchesLocal(instance, packages));
  }

  fn instance_matches_highest_or_lowest_semver_but_mismatches_semver_group(
    effects: &mut impl Effects,
    instance: Instance,
    packages: &mut Packages,
  ) {
    effects.on(Event::InstanceMatchesHighestOrLowestSemverButMismatchesSemverGroup(
      instance, packages,
    ));
  }

  fn instance_mismatches_highest_or_lowest_semver(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceMismatchesHighestOrLowestSemver(instance, packages));
  }

  fn instance_mismatches_and_is_unsupported(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceMismatchesAndIsUnsupported(instance, packages));
  }

  /// Refuse to change local dependency specifiers
  fn local_instance_mistakenly_mismatches_semver_group(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::LocalInstanceMistakenlyMismatchesSemverGroup(instance, packages));
  }

  fn instance_matches_pinned_but_mismatches_semver_group(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceMatchesPinnedButMismatchesSemverGroup(instance, packages));
  }

  /// Refuse to change local dependency specifiers
  fn local_instance_mistakenly_mismatches_pinned(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::LocalInstanceMistakenlyMismatchesPinned(instance, packages));
  }

  fn instance_mismatches_pinned(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceMismatchesPinned(instance, packages));
  }

  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✘ If semver group is fixed, instance would still mismatch its same range group
  fn instance_mismatches_both_same_range_and_conflicting_semver_groups(
    effects: &mut impl Effects,
    instance: Instance,
    packages: &mut Packages,
  ) {
    effects.on(Event::InstanceMismatchesBothSameRangeAndConflictingSemverGroups(instance, packages));
  }

  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would match its same range group
  fn instance_mismatches_both_same_range_and_compatible_semver_groups(
    effects: &mut impl Effects,
    instance: Instance,
    packages: &mut Packages,
  ) {
    effects.on(Event::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups(instance, packages));
  }

  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✘ If semver group is fixed, instance would then mismatch its same range group
  fn instance_matches_same_range_group_but_mismatches_conflicting_semver_group(
    effects: &mut impl Effects,
    instance: Instance,
    packages: &mut Packages,
  ) {
    effects.on(Event::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup(
      instance, packages,
    ));
  }

  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would still match its same range group
  fn instance_matches_same_range_group_but_mismatches_compatible_semver_group(
    effects: &mut impl Effects,
    instance: Instance,
    packages: &mut Packages,
  ) {
    effects.on(Event::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup(
      instance, packages,
    ));
  }

  /// ✘ Instance mismatches its same range group
  /// ✓ Instance matches its semver group
  /// ✘ We can't know what range the user wants and have to ask them
  fn instance_mismatches_same_range_group(effects: &mut impl Effects, instance: Instance, packages: &mut Packages) {
    effects.on(Event::InstanceMismatchesSameRangeGroup(instance, packages));
  }
  */

  if cli.options.versions {
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
        group.dependencies.values().for_each(|dependency| {
          match dependency.variant {
            Variant::Banned => {
              effects.on(Event::DependencyBanned(dependency));
              dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                if instance.is_local {
                  effects.on(Event::LocalInstanceMistakenlyBanned(instance, &mut packages));
                } else {
                  effects.on(Event::InstanceIsBanned(instance, &mut packages));
                }
              });
            }
            Variant::HighestSemver | Variant::LowestSemver => {
              let prefer_highest = matches!(dependency.variant, Variant::HighestSemver);
              let preferred_order: Cmp = if prefer_highest { Cmp::Gt } else { Cmp::Lt };
              let label: &str = if prefer_highest { "highest" } else { "lowest" };
              match dependency.get_local_specifier(&instances_by_id) {
                Some(local_specifier) => {
                  dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                    if instance.is_local {
                      effects.on(Event::LocalInstanceIsPreferred(instance));
                    } else if instance.actual.matches(&local_specifier) {
                      if instance.has_range_mismatch() {
                        effects.on(Event::InstanceMatchesLocalButMismatchesSemverGroup(instance, &mut packages));
                      } else {
                        effects.on(Event::InstanceMatchesLocal(instance));
                      }
                    } else {
                      effects.on(Event::InstanceMismatchesLocal(instance, &mut packages));
                    }
                  });
                }
                None => {
                  if dependency.all_are_semver(&instances_by_id) {
                    match dependency.get_highest_or_lowest_semver(&instances_by_id, preferred_order) {
                      Some(preferred) => {
                        dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                          if instance.actual.matches(&preferred) {
                            if instance.has_range_mismatch() {
                              effects.on(Event::InstanceMatchesHighestOrLowestSemverButMismatchesSemverGroup(instance, &mut packages));
                            } else {
                              effects.on(Event::InstanceMatchesHighestOrLowestSemver(instance));
                            }
                          } else {
                            effects.on(Event::InstanceMismatchesHighestOrLowestSemver(instance, &mut packages));
                          }
                        });
                      }
                      None => {
                        panic!("No {} semver found for dependency {:?}", label, dependency);
                      }
                    }
                  } else if dependency.all_are_identical(&instances_by_id) {
                    dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                      effects.on(Event::InstanceMatchesButIsUnsupported(instance));
                    });
                  } else {
                    dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                      effects.on(Event::InstanceMismatchesAndIsUnsupported(instance, &mut packages));
                    });
                  }
                }
              }
            }
            Variant::Ignored => {
              dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                effects.on(Event::InstanceIsIgnored(instance));
              });
            }
            Variant::Pinned => match &dependency.pinned_specifier {
              Some(pinned) => {
                dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                  if instance.actual.matches(&pinned) {
                    if instance.has_range_mismatch() {
                      if instance.is_local {
                        effects.on(Event::LocalInstanceMistakenlyMismatchesSemverGroup(instance, &mut packages));
                      } else {
                        effects.on(Event::InstanceMatchesPinnedButMismatchesSemverGroup(instance, &mut packages));
                      }
                    } else {
                      effects.on(Event::InstanceMatchesPinned(instance));
                    }
                  } else if instance.is_local {
                    effects.on(Event::LocalInstanceMistakenlyMismatchesPinned(instance, &mut packages));
                  } else {
                    effects.on(Event::InstanceMismatchesPinned(instance, &mut packages));
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
                dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                  if instance.has_range_mismatch() {
                    if mismatches.contains_key(&instance.actual) {
                      if mismatches.contains_key(&instance.expected) {
                        effects.on(Event::InstanceMismatchesBothSameRangeAndConflictingSemverGroups(instance, &mut packages));
                      } else {
                        effects.on(Event::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups(instance, &mut packages));
                      }
                    } else {
                      if mismatches.contains_key(&instance.expected) {
                        effects.on(Event::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup(instance, &mut packages));
                      } else {
                        effects.on(Event::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup(instance, &mut packages));
                      }
                    }
                  } else {
                    if mismatches.contains_key(&instance.actual) {
                      effects.on(Event::InstanceMismatchesSameRangeGroup(instance, &mut packages));
                    } else {
                      effects.on(Event::InstanceMatchesSameRangeGroup(instance));
                    }
                  }
                });
              } else if dependency.all_are_identical(&instances_by_id) {
                dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                  effects.on(Event::InstanceMatchesButIsUnsupported(instance));
                });
              } else {
                dependency.get_instances(&instances_by_id).iter_mut().for_each(|instance| {
                  effects.on(Event::InstanceMismatchesAndIsUnsupported(instance, &mut packages));
                });
              }
            }
            Variant::SnappedTo => {
              let snapped_to_specifier = dependency.get_snapped_to_specifier(&instances_by_id);
            }
          };
        });
      });
  }

  effects.on(Event::EnterFormat);

  if cli.options.format {
    let InMemoryFormattingStatus { was_valid, was_invalid } = format::fix(&config, packages);
    if !was_valid.is_empty() {
      effects.on(Event::PackagesMatchFormatting(&was_valid));
    }
    if !was_invalid.is_empty() {
      effects.on(Event::PackagesMismatchFormatting(&was_invalid));
    }
  }

  effects.on(Event::ExitCommand);
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
  fn runs_effect_when_packages_loaded() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::new();

    lint(&config, &mut packages, &mut effects);
    assert_eq!(effects.events.packages_loaded.len(), 1);
  }

  #[test]
  fn reports_one_highest_version_mismatch_in_one_file() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "1.0.0"
      },
      "devDependencies": {
        "wat": "2.0.0"
      }
    })]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects)
      .to_have_standard_version_group_matches(vec![ExpectedMatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /devDependencies of package-a",
        specifier: "2.0.0",
      }])
      .to_have_highest_version_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-a",
        actual_specifier: "1.0.0",
        expected_specifier: "2.0.0",
      }]);
  }

  #[test]
  fn reports_many_highest_version_mismatches_in_one_file() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![json!({
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

    lint(&config, &mut packages, &mut effects);

    expect(&effects)
      .to_have_standard_version_group_matches(vec![ExpectedMatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /devDependencies of package-a",
        specifier: "0.3.0",
      }])
      .to_have_highest_version_mismatches(vec![
        ExpectedMismatchEvent {
          dependency_name: "wat",
          instance_id: "wat in /dependencies of package-a",
          actual_specifier: "0.1.0",
          expected_specifier: "0.3.0",
        },
        ExpectedMismatchEvent {
          dependency_name: "wat",
          instance_id: "wat in /peerDependencies of package-a",
          actual_specifier: "0.2.0",
          expected_specifier: "0.3.0",
        },
      ]);
  }

  #[test]
  fn reports_highest_version_mismatches_in_many_files() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![
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

    lint(&config, &mut packages, &mut effects);

    expect(&effects)
      .to_have_standard_version_group_matches(vec![ExpectedMatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-b",
        specifier: "2.0.0",
      }])
      .to_have_highest_version_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-a",
        actual_specifier: "1.0.0",
        expected_specifier: "2.0.0",
      }]);
  }

  #[test]
  fn does_not_consider_instances_in_different_version_groups_a_highest_version_mismatch() {
    let mut effects = MockEffects::new();
    let config = Config::from_mock(json!({
      "versionGroups": [
        { "packages": ["package-a"] },
        { "packages": ["package-b"] }
      ]
    }));
    let mut packages = Packages::from_mocks(vec![
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

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_highest_version_mismatches(vec![]).to_have_standard_version_group_matches(vec![
      ExpectedMatchEvent {
        dependency_name: "good",
        instance_id: "good in /dependencies of package-a",
        specifier: "1.0.0",
      },
      ExpectedMatchEvent {
        dependency_name: "good",
        instance_id: "good in /dependencies of package-b",
        specifier: "2.0.0",
      },
    ]);
  }

  #[test]
  fn rejects_pinned_version_when_it_would_replace_local_version() {
    let mut effects = MockEffects::new();
    let config = Config::from_mock(json!({
      "versionGroups": [{
        "dependencies": ["package-a"],
        "pinVersion": "1.2.0"
      }]
    }));
    let mut packages = Packages::from_mocks(vec![
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

    lint(&config, &mut packages, &mut effects);

    expect(&effects)
      .to_have_rejected_local_version_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual_specifier: "1.0.0",
        expected_specifier: "1.2.0",
      }])
      .to_have_pinned_version_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual_specifier: "1.1.0",
        expected_specifier: "1.2.0",
      }]);
  }

  #[test]
  fn does_not_confuse_highest_version_matches_and_mismatches() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![
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

    lint(&config, &mut packages, &mut effects);

    expect(&effects)
      .to_have_standard_version_group_matches(vec![
        ExpectedMatchEvent {
          dependency_name: "mix",
          instance_id: "mix in /dependencies of package-a",
          specifier: "0.3.0",
        },
        ExpectedMatchEvent {
          dependency_name: "mix",
          instance_id: "mix in /devDependencies of package-b",
          specifier: "0.3.0",
        },
      ])
      .to_have_highest_version_mismatches(vec![
        ExpectedMismatchEvent {
          dependency_name: "mix",
          instance_id: "mix in /devDependencies of package-a",
          actual_specifier: "0.1.0",
          expected_specifier: "0.3.0",
        },
        ExpectedMismatchEvent {
          dependency_name: "mix",
          instance_id: "mix in /peerDependencies of package-a",
          actual_specifier: "0.2.0",
          expected_specifier: "0.3.0",
        },
      ]);
  }

  #[test]
  fn reports_local_version_mismatch_when_an_instance_uses_a_higher_version() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![
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

    lint(&config, &mut packages, &mut effects);

    expect(&effects)
      .to_have_standard_version_group_matches(vec![ExpectedMatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        specifier: "1.0.0",
      }])
      .to_have_local_version_mismatches(vec![
        ExpectedMismatchEvent {
          dependency_name: "package-a",
          instance_id: "package-a in /dependencies of package-b",
          actual_specifier: "1.1.0",
          expected_specifier: "1.0.0",
        },
        ExpectedMismatchEvent {
          dependency_name: "package-a",
          instance_id: "package-a in /devDependencies of package-b",
          actual_specifier: "workspace:*",
          expected_specifier: "1.0.0",
        },
      ]);
  }

  #[test]
  fn instance_has_same_version_as_local_package_but_does_not_match_its_semver_group() {
    let mut effects = MockEffects::new();
    let config = Config::from_mock(json!({
      "semverGroups": [{
        "range": "^"
      }]
    }));
    let mut packages = Packages::from_mocks(vec![
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

    lint(&config, &mut packages, &mut effects);

    // refuse to break local package's version
    expect(&effects)
      .debug()
      .to_have_rejected_local_version_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual_specifier: "1.0.0",
        expected_specifier: "^1.0.0",
      }])
      .to_have_semver_range_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual_specifier: "1.0.0",
        expected_specifier: "^1.0.0",
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
  #[ignore]
  fn highest_version_match_becomes_mismatch_after_semver_range_has_been_fixed() {
    let mut effects = MockEffects::new();
    let config = Config::from_mock(json!({
      "semverGroups": [{
        "dependencyTypes": ["dev"],
        "range": ">"
      }]
    }));
    let mut packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "foo": "1.0.0"
      },
      "devDependencies": {
        "foo": "1.0.0"
      }
    })]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects)
      .to_have_highest_version_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual_specifier: "1.0.0",
        expected_specifier: ">1.0.0",
      }])
      .to_have_semver_range_mismatches(vec![ExpectedMismatchEvent {
        dependency_name: "foo",
        instance_id: "foo in /devDependencies of package-a",
        actual_specifier: "1.0.0",
        expected_specifier: ">1.0.0",
      }]);
  }
}
