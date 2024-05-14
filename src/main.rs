#![allow(dead_code)]
#![allow(unused_variables)]

use cli::CliOptions;
use colored::*;
use dependency_type::{DependencyType, Strategy};
use effects_fix::FixEffects;
use glob::glob;
use instance::Instance;
use json_file::read_json_file;
use package_json::Packages;
use regex::Regex;
use serde_json::Value;
use std::{collections::HashMap, io, path::PathBuf};

use crate::{
  config::Rcfile, effects::Effects, effects_lint::LintEffects, format::LintResult,
  semver_group::SemverGroup, version_group::VersionGroup,
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
  let filter = rcfile.get_filter();
  let dependency_types = Rcfile::get_enabled_dependency_types(&rcfile);
  let source_patterns = get_enabled_source_patterns(&cli_options, &rcfile);
  let absolute_file_paths = get_file_paths(&cwd, &source_patterns);
  let semver_groups = SemverGroup::from_rcfile(&rcfile);

  // all dependent on `packages`
  let packages = get_packages(&absolute_file_paths);
  let mut version_groups = VersionGroup::from_rcfile(&rcfile, &packages.all_names);
  let instances = get_instances(&packages, &dependency_types, &filter);

  // assign every instance to the first group it matches
  instances.iter().for_each(|instance| {
    let semver_group = semver_groups
      .iter()
      .find(|semver_group| semver_group.selector.can_add(instance));
    version_groups
      .iter_mut()
      .find(|version_group| version_group.selector.can_add(instance))
      .unwrap()
      .add_instance(instance, semver_group);
  });

  // Switch version groups back to immutable
  let version_groups = version_groups;

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
        version_groups.iter().for_each(|group| {
          // @TODO: update effects to return a bool
          let group_is_valid = group.visit(&instances, &effects, &mut packages);
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
          let group_is_valid = group.visit(&instances, &effects, &mut packages);
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

/// Based on the user's config file and command line `--source` options, return
/// the source glob patterns which should be used to resolve package.json files
fn get_enabled_source_patterns(cli_options: &CliOptions, rcfile: &Rcfile) -> Vec<String> {
  Some(cli_options.source.clone())
    .filter(|list| !list.is_empty())
    .or_else(|| Some(rcfile.source.clone()))
    .filter(|list| !list.is_empty())
    .or_else(|| {
      Some(vec![
        String::from("package.json"),
        String::from("packages/*/package.json"),
      ])
    })
    .unwrap_or(vec![])
}

/// Resolve every source glob pattern into their absolute file paths of
/// package.json files
fn get_file_paths(cwd: &PathBuf, source_patterns: &Vec<String>) -> Vec<PathBuf> {
  source_patterns
    .iter()
    .map(|pattern| {
      if PathBuf::from(pattern).is_absolute() {
        pattern.clone()
      } else {
        cwd.join(pattern).to_str().unwrap().to_string()
      }
    })
    .flat_map(|pattern| glob(&pattern).ok())
    .flat_map(|paths| {
      paths
        .filter_map(Result::ok)
        .fold(vec![], |mut paths, path| {
          paths.push(path.clone());
          paths
        })
    })
    .collect()
}

/// Get every package.json file matched by the user's source patterns
fn get_packages(file_paths: &Vec<PathBuf>) -> Packages {
  let mut packages = Packages {
    all_names: vec![],
    by_name: HashMap::new(),
  };
  for file_path in file_paths {
    if let Ok(file) = read_json_file(&file_path) {
      let name = file.get_name();
      packages.all_names.push(name.clone());
      packages.by_name.insert(name.clone(), file);
    }
  }
  packages
}

/// Get every instance of a dependency from every package.json file
fn get_instances<'a>(
  packages: &'a Packages,
  dependency_types: &Vec<DependencyType>,
  filter: &Regex,
) -> Vec<Instance> {
  let mut instances: Vec<Instance> = vec![];
  for package in packages.by_name.values() {
    for dependency_type in dependency_types {
      match dependency_type.strategy {
        Strategy::NameAndVersionProps => {
          if let (Some(Value::String(name)), Some(Value::String(version))) = (
            package.get_prop(&dependency_type.name_path.as_ref().unwrap()),
            package.get_prop(&dependency_type.path),
          ) {
            if filter.is_match(name) {
              instances.push(Instance::new(
                name.to_string(),
                version.to_string(),
                dependency_type.clone(),
                &package,
              ));
            }
          }
        }
        Strategy::NamedVersionString => {
          if let Some(Value::String(specifier)) = package.get_prop(&dependency_type.path) {
            if let Some((name, version)) = specifier.split_once('@') {
              if filter.is_match(name) {
                instances.push(Instance::new(
                  name.to_string(),
                  version.to_string(),
                  dependency_type.clone(),
                  &package,
                ));
              }
            }
          }
        }
        Strategy::UnnamedVersionString => {
          if let Some(Value::String(version)) = package.get_prop(&dependency_type.path) {
            if filter.is_match(&dependency_type.name) {
              instances.push(Instance::new(
                dependency_type.name.clone(),
                version.to_string(),
                dependency_type.clone(),
                &package,
              ));
            }
          }
        }
        Strategy::VersionsByName => {
          if let Some(Value::Object(versions_by_name)) = package.get_prop(&dependency_type.path) {
            for (name, version) in versions_by_name {
              if filter.is_match(name) {
                if let Value::String(version) = version {
                  instances.push(Instance::new(
                    name.to_string(),
                    version.to_string(),
                    dependency_type.clone(),
                    &package,
                  ));
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
  instances
}
