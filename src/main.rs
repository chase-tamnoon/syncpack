#![allow(dead_code)]
#![allow(unused_variables)]

use cli::CliOptions;
use colored::*;
use glob::glob;
use json_file::read_json_file;
use package_json::PackageJson;
use regex::Regex;
use std::{io, path::PathBuf};

use crate::{
  config::Rcfile, effects::Effects, format::LintResult, lint_effects::LintEffects,
  semver_group::SemverGroup, version_group::VersionGroup,
};

mod cli;
mod config;
mod dependency_type;
mod effects;
mod format;
mod group_selector;
mod instance;
mod instance_group;
mod json_file;
mod lint_effects;
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
  let enabled_source_patterns = get_enabled_source_patterns(&cli_options, &rcfile);
  let absolute_file_paths = get_file_paths(&cwd, &enabled_source_patterns);
  let semver_groups = SemverGroup::from_rcfile(&rcfile);

  // all dependent on `packages`
  let packages = get_packages(&absolute_file_paths);
  let mut version_groups = VersionGroup::from_rcfile(&rcfile, &packages.all_names);
  let instances = get_instances(&packages.all, &dependency_types, &filter);

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

  // @FIXME: how can this expensive clone be avoided?
  //
  // Above this line `packages` only needs to be read, but below it needs to be
  // mutated. I *think* via lifetimes I've tied the lifetime of `packages` to
  // the lifetime of `all_instances`(?) which could be why I can't borrow it
  // mutably here(?)
  let mut packages = packages.all.clone();

  let is_valid: bool = match command_name {
    Subcommand::Lint => {
      let effects = LintEffects {};
      let mut lint_is_valid = true;

      if cli_options.format {
        effects.on_begin_format();
        let LintResult { valid, invalid } = format::lint(&rcfile, &mut packages);
        effects.on_formatted_packages(&valid, &cwd);
        effects.on_unformatted_packages(&invalid, &cwd);
        if !invalid.is_empty() {
          lint_is_valid = false;
        }
      }

      match (cli_options.ranges, cli_options.versions) {
        (true, true) => effects.on_begin_ranges_and_versions(),
        (true, false) => effects.on_begin_ranges_only(),
        (false, true) => effects.on_begin_versions_only(),
        (false, false) => effects.on_skip_ranges_and_versions(),
      };

      version_groups.iter().for_each(|group| {
        let group_is_valid = group.visit(&instances, &effects);
        if !group_is_valid {
          lint_is_valid = false;
        }
      });

      lint_is_valid
    }
    Subcommand::Fix => {
      println!("fix enabled {:?}", cli_options);
      if cli_options.format {
        println!("format packages");
        format::fix(&rcfile, &mut packages);
      }
      if cli_options.versions {
        println!("@TOD: fix versions");
      }
      true
    }
  };

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

struct Packages {
  all: Vec<PackageJson>,
  all_names: Vec<String>,
}

/// Get every package.json file matched by the user's source patterns
fn get_packages(file_paths: &Vec<PathBuf>) -> Packages {
  let mut packages = Packages {
    all: vec![],
    all_names: vec![],
  };
  for file_path in file_paths {
    if let Ok(file) = read_json_file(&file_path) {
      packages.all_names.push(file.get_name());
      packages.all.push(file);
    }
  }
  packages
}

/// Get every instance of a dependency from every package.json file
fn get_instances<'a>(
  packages: &'a Vec<PackageJson>,
  dependency_types: &'a Vec<dependency_type::DependencyType>,
  filter: &Regex,
) -> Vec<instance::Instance<'a>> {
  packages
    .iter()
    .flat_map(|package| package.get_instances(&dependency_types, &filter))
    .collect()
}
