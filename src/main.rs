#![allow(dead_code)]
#![allow(unused_variables)]

use colored::*;
use itertools::Itertools;
use std::{cmp::Ordering, collections::BTreeMap, io};

use crate::{
  cli::Subcommand, effects::Effects, effects_fix::FixEffects, effects_lint::LintEffects,
  format::LintResult, instance_group::InstancesById, packages::get_packages,
  version_group::VersionGroupVariant,
};

mod cli;
mod config;
mod dependency_type;
mod effects;
mod effects_fix;
mod effects_lint;
mod format;
mod group_selector;
mod instance;
mod instance_group;
mod json_file;
mod package_json;
mod packages;
mod semver_group;
mod specifier;
mod version_group;

fn main() -> io::Result<()> {
  env_logger::init();

  let cli = cli::parse_input();
  let cwd = std::env::current_dir()?;
  let rcfile = config::get(&cwd);
  let semver_groups = rcfile.get_semver_groups();
  let packages = get_packages(&cwd, &cli.options, &rcfile);

  let mut version_groups = rcfile.get_version_groups(&packages.all_names);
  let mut instances_by_id: InstancesById = BTreeMap::new();

  packages.get_all_instances(&rcfile, |instance| {
    // assign every instance to the first group it matches
    let semver_group = semver_groups
      .iter()
      .find(|semver_group| semver_group.selector.can_add(&instance));
    version_groups
      .iter_mut()
      .find(|version_group| version_group.selector.can_add(&instance))
      .unwrap()
      .add_instance(&instance, semver_group);
    // move instance to the lookup
    instances_by_id.insert(instance.id.clone(), instance);
  });

  // packages are mutated when linting formatting, but not written to disk
  // everything is mutated and written when fixing
  let mut packages = packages;

  let is_valid: bool = match cli.command_name {
    Subcommand::Fix => {
      let effects = FixEffects {};
      let mut fix_is_valid = true;

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
            let group_is_valid = group.visit(&mut instances_by_id, &effects, &mut packages);
            if !group_is_valid {
              fix_is_valid = false;
            }
          });
      }

      if cli.options.format {
        effects.on_begin_format();
        let LintResult { valid, invalid } = format::lint(&rcfile, &mut packages);
        effects.on_formatted_packages(&valid, &cwd);
        effects.on_unformatted_packages(&invalid, &cwd);
      }

      // write the changes to the package.json files
      packages.by_name.values_mut().for_each(|package| {
        package.write_to_disk(&rcfile.indent);
      });

      fix_is_valid
    }
    Subcommand::Lint => {
      let effects = LintEffects {};
      let mut lint_is_valid = true;

      match (cli.options.ranges, cli.options.versions) {
        (true, true) => effects.on_begin_ranges_and_versions(),
        (true, false) => effects.on_begin_ranges_only(),
        (false, true) => effects.on_begin_versions_only(),
        (false, false) => effects.on_skip_ranges_and_versions(),
      };

      if cli.options.ranges || cli.options.versions {
        version_groups.iter().for_each(|group| {
          let group_is_valid = group.visit(&mut instances_by_id, &effects, &mut packages);
          if !group_is_valid {
            lint_is_valid = false;
          }
        });
      }

      if cli.options.format {
        effects.on_begin_format();
        let LintResult { valid, invalid } = format::lint(&rcfile, &mut packages);
        effects.on_formatted_packages(&valid, &cwd);
        effects.on_unformatted_packages(&invalid, &cwd);
        if !invalid.is_empty() {
          lint_is_valid = false;
        }
      }

      lint_is_valid
    }
  };

  // @TODO: when fixing and unfixable errors happen, explain them to the user

  if is_valid {
    println!("{} {}", "\n✓".green(), "syncpack found no errors");
    std::process::exit(0);
  } else {
    println!("{} {}", "\n✘".red(), "syncpack found errors");
    std::process::exit(1);
  }
}
