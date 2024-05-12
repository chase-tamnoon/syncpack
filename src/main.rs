#![allow(dead_code)]
#![allow(unused_variables)]

use cli::CliOptions;
use colored::*;
use log::debug;
use path_buf::path_buf_to_str;
use regex::Regex;
use std::{collections::HashMap, io, path};

use crate::{
  config::Rcfile, effects::Effects, format::LintResult, semver_group::SemverGroup,
  version_group::VersionGroup,
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
mod package_json;
mod path_buf;
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

  debug!("cwd: {:?}", &cwd);
  debug!("command_name: {:?}", &command_name);
  debug!("cli_options: {:?}", &cli_options);
  debug!("rcfile: {:?}", &rcfile);

  let dependency_types = Rcfile::get_enabled_dependency_types(&rcfile);
  debug!("dependency_types: {:?}", dependency_types);
  let file_paths = get_sources(&cwd, &cli_options, &rcfile).unwrap();
  debug!("file_paths: {:?}", file_paths);
  let mut packages = get_packages(&file_paths).unwrap();
  debug!("packages: {:?}", packages);
  let local_package_names = get_local_package_names(&packages);
  debug!("local_package_names: {:?}", local_package_names);
  let semver_groups = SemverGroup::from_rcfile(&rcfile);
  debug!("semver_groups: {:?}", semver_groups);
  let mut version_groups = VersionGroup::from_rcfile(&rcfile, &local_package_names);
  debug!("version_groups: {:?}", version_groups);

  let instances = get_instances(&packages, &dependency_types, &rcfile.get_filter());

  // assign every instance to the first group it matches
  instances.iter().for_each(|instance| {
    let semver_group = semver_groups
      .iter()
      .find(|semver_group| semver_group.selector.can_add(instance));
    version_groups
      .iter_mut()
      .find(|version_group| version_group.selector.can_add(instance))
      .expect("instance did not match a version group")
      .add_instance(instance, semver_group);
  });

  let is_valid: bool = match command_name {
    Subcommand::Lint => {
      let effects = Effects {};
      let mut lint_is_valid = lint_formatting(&cwd, &rcfile, &packages, &cli_options);

      let header = match (cli_options.ranges, cli_options.versions) {
        (true, true) => "= SEMVER RANGES AND VERSION MISMATCHES",
        (true, false) => "= SEMVER RANGES",
        (false, true) => "= VERSION MISMATCHES",
        (false, false) => "",
      };

      if header != "" {
        println!("{}", header.yellow());
      }

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

/// Return a right aligned column of a count of instances
/// Example "    38x"
fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

/// Check formatting of package.json files and return whether all are valid
fn lint_formatting(
  cwd: &path::PathBuf,
  rcfile: &Rcfile,
  packages: &Vec<package_json::PackageJson>,
  enabled: &cli::CliOptions,
) -> bool {
  if !enabled.format {
    return true;
  }
  println!("{}", "= FORMATTING".yellow());
  let LintResult { valid, invalid } = format::lint(rcfile, packages);
  println!("{} {} valid", render_count_column(valid.len()), "✓".green());
  println!(
    "{} {} invalid",
    render_count_column(invalid.len()),
    "✘".red()
  );
  invalid.iter().for_each(|package| {
    println!(
      "      {} {}",
      "✘".red(),
      package.get_relative_file_path(cwd).red()
    );
  });
  invalid.len() == 0
}

fn get_sources(
  cwd: &path::PathBuf,
  cli_options: &CliOptions,
  rcfile: &Rcfile,
) -> io::Result<Vec<path::PathBuf>> {
  let sources: Vec<path::PathBuf> = if cli_options.source.len() > 0 {
    cli_options.source.clone()
  } else {
    rcfile.get_sources(&cwd)
  };

  let mut file_paths: Vec<path::PathBuf> = vec![];

  for source in sources.iter() {
    let absolute_source = cwd.join(source);
    match glob::glob(path_buf_to_str(&absolute_source)) {
      Ok(glob_paths) => {
        for glob_path in glob_paths {
          match glob_path {
            Ok(file_path) => {
              file_paths.push(file_path);
            }
            Err(_) => {
              panic!("Failed to read source {:?}", source);
            }
          };
        }
      }
      Err(_) => {
        panic!("Failed to read source {:?}", source);
      }
    };
  }

  Ok(file_paths)
}

fn get_packages(file_paths: &Vec<path::PathBuf>) -> io::Result<Vec<package_json::PackageJson>> {
  Ok(
    file_paths
      .iter()
      .map(|file_path| match json_file::read_json_file(&file_path) {
        Ok(package_json) => package_json,
        Err(err) => {
          panic!("Failed to read {:?} {}", &file_path.to_str(), err);
        }
      })
      .collect(),
  )
}

/// Get all package names, to be used by the `$LOCAL` alias
fn get_local_package_names(packages: &Vec<package_json::PackageJson>) -> Vec<String> {
  packages.iter().map(|package| package.get_name()).collect()
}

fn get_instances<'a>(
  packages: &'a Vec<package_json::PackageJson>,
  dependency_types: &'a HashMap<String, dependency_type::DependencyType>,
  filter: &Regex,
) -> Vec<instance::Instance<'a>> {
  packages
    .iter()
    .flat_map(|package| package.get_instances(&dependency_types, &filter))
    .collect()
}
