use itertools::Itertools;
use std::{cmp::Ordering, path::PathBuf};

use crate::{
  cli::Cli,
  config::Rcfile,
  dependency::InstancesById,
  effects::Effects,
  format::{self, LintResult},
  packages::Packages,
  version_group::{VersionGroup, VersionGroupVariant},
};

pub fn fix<T: Effects>(
  cwd: &PathBuf,
  cli: &Cli,
  rcfile: &Rcfile,
  packages: &mut Packages,
  instances_by_id: &mut InstancesById,
  version_groups: &mut Vec<VersionGroup>,
  effects: &T,
) -> () {
  let mut is_valid = true;

  match (cli.options.ranges, cli.options.versions) {
    (true, true) => effects.on_begin_ranges_and_versions(),
    (true, false) => effects.on_begin_ranges_only(),
    (false, true) => effects.on_begin_versions_only(),
    (false, false) => effects.on_skip_ranges_and_versions(),
  };

  if cli.options.ranges || cli.options.versions {
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
        let group_is_valid = group.visit(instances_by_id, effects, packages);
        if !group_is_valid {
          is_valid = false;
        }
      });
  }

  if cli.options.format {
    effects.on_begin_format();
    let LintResult { valid, invalid } = format::lint(&rcfile, packages);
    effects.on_formatted_packages(&valid, &cwd);
    effects.on_unformatted_packages(&invalid, &cwd);
  }

  // write the changes to the package.json files
  packages.by_name.values_mut().for_each(|package| {
    package.write_to_disk(&rcfile.indent);
  });

  effects.on_complete(is_valid);
}
