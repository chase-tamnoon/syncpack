#![allow(dead_code)]
#![allow(unused_variables)]

use colored::*;
use config::Rcfile;
use itertools::Itertools;
use serde_json::Value;
use std::{cmp::Ordering, collections::BTreeMap, io};

use crate::{
  dependency_type::Strategy, effects::Effects, effects_fix::FixEffects, effects_lint::LintEffects,
  format::LintResult, instance::Instance, instance_group::InstancesById, package_json::Packages,
  packages::get_packages, version_group::VersionGroupVariant,
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

#[derive(Debug)]
enum Subcommand {
  Lint,
  Fix,
}

fn main() -> io::Result<()> {
  env_logger::init();

  let subcommand = match cli::create().get_matches().subcommand() {
    Some(("lint", matches)) => (Subcommand::Lint, cli::get_cli_options(matches)),
    Some(("fix", matches)) => (Subcommand::Fix, cli::get_cli_options(matches)),
    _ => {
      std::process::exit(1);
    }
  };

  let (command_name, cli_options) = &subcommand;
  let cwd = std::env::current_dir()?;
  let rcfile = config::get(&cwd);
  let semver_groups = rcfile.get_semver_groups();

  // all dependent on `packages`
  let packages = get_packages(&cwd, &cli_options, &rcfile);
  let mut version_groups = rcfile.get_version_groups(&packages.all_names);
  let instances_by_id = get_all_instances(&packages, &rcfile);

  // assign every instance to the first group it matches
  instances_by_id.iter().for_each(|(_, instance)| {
    let semver_group = semver_groups
      .iter()
      .find(|semver_group| semver_group.selector.can_add(instance));
    version_groups
      .iter_mut()
      .find(|version_group| version_group.selector.can_add(instance))
      .unwrap()
      .add_instance(instance, semver_group);
  });

  let mut instances_by_id = instances_by_id;

  // packages are mutated when linting formatting, but not written to disk
  // everything is mutated and written when fixing
  let mut packages = packages;

  let is_valid: bool = match command_name {
    Subcommand::Fix => {
      let effects = FixEffects {};
      let mut fix_is_valid = true;

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
            let group_is_valid = group.visit(&mut instances_by_id, &effects, &mut packages);
            if !group_is_valid {
              fix_is_valid = false;
            }
          });
      }

      if cli_options.format {
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

      match (cli_options.ranges, cli_options.versions) {
        (true, true) => effects.on_begin_ranges_and_versions(),
        (true, false) => effects.on_begin_ranges_only(),
        (false, true) => effects.on_begin_versions_only(),
        (false, false) => effects.on_skip_ranges_and_versions(),
      };

      if cli_options.ranges || cli_options.versions {
        version_groups.iter().for_each(|group| {
          let group_is_valid = group.visit(&mut instances_by_id, &effects, &mut packages);
          if !group_is_valid {
            lint_is_valid = false;
          }
        });
      }

      if cli_options.format {
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

/// Get every instance of a dependency from every package.json file
fn get_all_instances(packages: &Packages, rcfile: &Rcfile) -> InstancesById {
  let filter = &rcfile.get_filter();
  let dependency_types = &rcfile.get_enabled_dependency_types();
  let mut instances_by_id: InstancesById = BTreeMap::new();
  for package in packages.by_name.values() {
    for dependency_type in dependency_types {
      match dependency_type.strategy {
        Strategy::NameAndVersionProps => {
          if let (Some(Value::String(name)), Some(Value::String(version))) = (
            package.get_prop(&dependency_type.name_path.as_ref().unwrap()),
            package.get_prop(&dependency_type.path),
          ) {
            if filter.is_match(name) {
              let instance = Instance::new(
                name.to_string(),
                version.to_string(),
                dependency_type.clone(),
                &package,
              );
              instances_by_id.insert(instance.id.clone(), instance);
            }
          }
        }
        Strategy::NamedVersionString => {
          if let Some(Value::String(specifier)) = package.get_prop(&dependency_type.path) {
            if let Some((name, version)) = specifier.split_once('@') {
              if filter.is_match(name) {
                let instance = Instance::new(
                  name.to_string(),
                  version.to_string(),
                  dependency_type.clone(),
                  &package,
                );
                instances_by_id.insert(instance.id.clone(), instance);
              }
            }
          }
        }
        Strategy::UnnamedVersionString => {
          if let Some(Value::String(version)) = package.get_prop(&dependency_type.path) {
            if filter.is_match(&dependency_type.name) {
              let instance = Instance::new(
                dependency_type.name.clone(),
                version.to_string(),
                dependency_type.clone(),
                &package,
              );
              instances_by_id.insert(instance.id.clone(), instance);
            }
          }
        }
        Strategy::VersionsByName => {
          if let Some(Value::Object(versions_by_name)) = package.get_prop(&dependency_type.path) {
            for (name, version) in versions_by_name {
              if filter.is_match(name) {
                if let Value::String(version) = version {
                  let instance = Instance::new(
                    name.to_string(),
                    version.to_string(),
                    dependency_type.clone(),
                    &package,
                  );
                  instances_by_id.insert(instance.id.clone(), instance);
                }
              }
            }
          }
        }
        Strategy::InvalidConfig => {
          panic!("unrecognised strategy");
        }
      };
    }
  }
  instances_by_id
}
