use itertools::Itertools;
use std::cmp::Ordering;

use crate::{
  config::Config,
  context::{self, Context},
  effects::Effects,
  format::{self, InMemoryFormattingStatus},
  packages::Packages,
  version_group::VersionGroupVariant,
};

pub fn fix(config: &Config, packages: &mut Packages, run_effect: fn(Effects) -> ()) {
  let cli_options = &config.cli.options;
  let Context {
    mut instances_by_id,
    mut state,
    version_groups,
  } = context::get(&config, &packages);

  run_effect(Effects::EnterVersionsAndRanges(&config));

  if cli_options.ranges || cli_options.versions {
    version_groups
      .iter()
      // fix snapped to groups last, so that the packages they're snapped to
      // have had any fixes applied to them first.
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
        group.visit(&mut instances_by_id, packages, run_effect, &mut state);
      });
  }

  run_effect(Effects::EnterFormat(&config));

  if cli_options.format {
    let InMemoryFormattingStatus {
      was_valid: valid,
      was_invalid: invalid,
    } = format::fix(&config, packages);
    if !valid.is_empty() {
      run_effect(Effects::PackagesMatchFormatting(&valid, &config));
    }
    if !invalid.is_empty() {
      run_effect(Effects::PackagesMismatchFormatting(
        &invalid, &config, &mut state,
      ));
    }
  }

  // write the changes to the package.json files
  packages.by_name.values_mut().for_each(|package| {
    package.write_to_disk(&config);
  });

  run_effect(Effects::ExitCommand(&mut state));
}
