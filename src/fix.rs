use itertools::Itertools;
use std::cmp::Ordering;

use crate::{
  config::Config,
  context::Context,
  effects::{Effects, Event},
  format::{self, InMemoryFormattingStatus},
  packages::Packages,
  version_group::Variant,
};

pub fn fix(config: &Config, packages: &mut Packages, effects: &mut impl Effects) {
  effects.on(Event::PackagesLoaded(&packages));

  let cli = &config.cli;
  let Context {
    mut instances_by_id,
    semver_groups,
    version_groups,
  } = Context::create(&config, &packages);

  effects.on(Event::EnterVersionsAndRanges);

  if cli.options.versions {
    version_groups
      .iter()
      // fix snapped to groups last, so that the packages they're snapped to
      // have had any fixes applied to them first.
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
        // group.visit(config, &mut instances_by_id, packages, effects);
      });
  }

  effects.on(Event::EnterFormat);

  if cli.options.format {
    let InMemoryFormattingStatus {
      was_valid: valid,
      was_invalid: invalid,
    } = format::fix(&config, packages);
    if !valid.is_empty() {
      effects.on(Event::PackagesMatchFormatting(&valid));
    }
    if !invalid.is_empty() {
      effects.on(Event::PackagesMismatchFormatting(&invalid));
    }
  }

  // write the changes to the package.json files
  packages.by_name.values_mut().for_each(|package| {
    package.write_to_disk(&config);
  });

  effects.on(Event::ExitCommand);
}
