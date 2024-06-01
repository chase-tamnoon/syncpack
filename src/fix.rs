use itertools::Itertools;
use std::cmp::Ordering;

use crate::{
  config::Config,
  context::{get_context, Context},
  effects::Effects,
  format::{self, LintResult},
  packages::Packages,
  version_group::VersionGroupVariant,
};

pub fn fix<T: Effects>(config: &Config, packages: &mut Packages, effects: &T) -> () {
  let mut is_valid = true;
  let cli_options = &config.cli.options;
  let Context {
    version_groups,
    mut instances_by_id,
  } = get_context(&config, &packages);

  match (cli_options.ranges, cli_options.versions) {
    (true, true) => effects.on_begin_ranges_and_versions(),
    (true, false) => effects.on_begin_ranges_only(),
    (false, true) => effects.on_begin_versions_only(),
    (false, false) => effects.on_skip_ranges_and_versions(),
  };

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
        // @TODO: update effects to return a bool
        let group_is_valid = group.visit(&mut instances_by_id, effects, packages);
        if !group_is_valid {
          is_valid = false;
        }
      });
  }

  if cli_options.format {
    effects.on_begin_format();
    let LintResult { valid, invalid } = format::lint(&config, packages);
    effects.on_formatted_packages(&valid, &config);
    effects.on_unformatted_packages(&invalid, &config);
  }

  // write the changes to the package.json files
  packages.by_name.values_mut().for_each(|package| {
    package.write_to_disk(&config);
  });

  effects.on_complete(is_valid);
}
