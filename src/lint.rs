use std::path::PathBuf;

use crate::{
  cli::Cli,
  config::Rcfile,
  context::{get_context, Context},
  effects::Effects,
  format::{self, LintResult},
  packages::Packages,
};

pub fn lint<T: Effects>(
  cwd: &PathBuf,
  cli: &Cli,
  rcfile: &Rcfile,
  packages: &mut Packages,
  effects: &T,
) -> () {
  let mut is_valid = true;
  let Context {
    version_groups,
    mut instances_by_id,
  } = get_context(&rcfile, &packages);

  match (cli.options.ranges, cli.options.versions) {
    (true, true) => effects.on_begin_ranges_and_versions(),
    (true, false) => effects.on_begin_ranges_only(),
    (false, true) => effects.on_begin_versions_only(),
    (false, false) => effects.on_skip_ranges_and_versions(),
  };

  if cli.options.ranges || cli.options.versions {
    version_groups.iter().for_each(|group| {
      let group_is_valid = group.visit(&mut instances_by_id, effects, packages);
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
    if !invalid.is_empty() {
      is_valid = false;
    }
  }

  effects.on_complete(is_valid);
}
