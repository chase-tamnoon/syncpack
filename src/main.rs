#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use cli::CliOptions;
use colored::*;
use itertools::Itertools;
use log::debug;
use node_semver::Range;
use path_buf::path_buf_to_str;
use regex::Regex;
use std::{
  collections::HashMap,
  io::{self, Error},
  path,
};

use crate::{
  config::Rcfile,
  format::LintResult,
  semver_group::SemverGroup,
  version_group::{PreferVersion, VersionGroup, VersionGroupVariant},
};

mod cli;
mod config;
mod dependency_type;
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
  let rcfile = config::get(&cwd).expect("missing config file");

  debug!("cwd: {:?}", &cwd);
  debug!("command_name: {:?}", &command_name);
  debug!("cli_options: {:?}", &cli_options);
  debug!("rcfile: {:?}", &rcfile);

  let dependency_types = Rcfile::get_enabled_dependency_types(&rcfile);
  debug!("dependency_types: {:?}", dependency_types);
  let semver_groups = SemverGroup::from_rcfile(&rcfile);
  debug!("semver_groups: {:?}", semver_groups);
  let mut version_groups = VersionGroup::from_rcfile(&rcfile);
  debug!("version_groups: {:?}", version_groups);
  let file_paths = get_sources(&cwd, &cli_options, &rcfile).expect("Failed to get globs");
  debug!("file_paths: {:?}", file_paths);
  let mut packages = get_packages(&file_paths).expect("Failed to read packages");
  debug!("packages: {:?}", packages);

  let instances = get_instances(&packages, &dependency_types, &rcfile.get_filter());

  // assign every instance to the first group it matches
  instances.iter().for_each(|instance| {
    let semver_group = semver_groups
      .iter()
      .find(|semver_group| semver_group.selector.can_add(instance))
      .expect("instance did not match a semver group");
    version_groups
      .iter_mut()
      .find(|version_group| version_group.selector.can_add(instance))
      .expect("instance did not match a version group")
      .add_instance(instance, semver_group);
  });

  let is_valid: bool = match command_name {
    Subcommand::Lint => {
      lint_formatting(&cwd, &rcfile, &packages, &cli_options);

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
        match group.variant {
          VersionGroupVariant::Ignored => {
            print_group_header(&group.selector.label);
            group
              .instance_groups_by_name
              .iter()
              .for_each(|(name, instance_group)| {
                print_ignored(instance_group, name);
              });
          }
          VersionGroupVariant::Banned => {
            print_group_header(&group.selector.label);
            group
              .instance_groups_by_name
              .iter()
              .for_each(|(name, instance_group)| {
                let count = render_count_column(instance_group.all.len());
                println!("{} {}", count, name.red());
                instance_group.unique_specifiers.iter().for_each(|actual| {
                  print_banned(instance_group, actual);
                });
              });
          }
          VersionGroupVariant::Pinned => {
            print_group_header(&group.selector.label);
            group
              .instance_groups_by_name
              .iter()
              .for_each(|(name, instance_group)| {
                if has_mismatches(instance_group) {
                  let count = render_count_column(instance_group.all.len());
                  println!("{} {}", count, name.red());
                  instance_group.unique_specifiers.iter().for_each(|actual| {
                    if instance_group.is_mismatch(actual) {
                      print_pinned_version_mismatch(instance_group, actual);
                    }
                  });
                } else {
                  print_version_match(instance_group, name);
                };
              });
          }
          VersionGroupVariant::SameRange => {
            print_group_header(&group.selector.label);
            group
              .instance_groups_by_name
              .iter()
              .for_each(|(name, instance_group)| {
                println!("@TODO SameRange: {}", name);

                instance_group.unique_specifiers.iter().for_each(|a| {
                  let range_a = a.parse::<Range>().unwrap();
                  instance_group.unique_specifiers.iter().for_each(|b| {
                    let range_b = b.parse::<Range>().unwrap();
                    println!("{:?} {:?} {}", a, b, range_a.allows_all(&range_b));
                  });
                });
              });
          }
          VersionGroupVariant::SnappedTo => {
            print_group_header(&group.selector.label);
            group
              .instance_groups_by_name
              .iter()
              .for_each(|(name, _instance_group)| {
                println!("@TODO SnappedTo: {}", name);
              });
          }
          VersionGroupVariant::Standard => {
            print_group_header(&group.selector.label);
            group
              .instance_groups_by_name
              .iter()
              .for_each(|(name, instance_group)| {
                if has_mismatches(instance_group) {
                  let count = render_count_column(instance_group.all.len());
                  println!("{} {}", count, name.red());
                  instance_group.unique_specifiers.iter().for_each(|actual| {
                    if instance_group.is_mismatch(actual) {
                      if let Some(PreferVersion::LowestSemver) = group.prefer_version {
                        print_lowest_version_mismatch(instance_group, actual);
                      } else {
                        print_highest_version_mismatch(instance_group, actual);
                      }
                    }
                  });
                } else {
                  print_version_match(instance_group, name);
                };
              });
          }
        };
      });
      true
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
    std::process::exit(0);
  } else {
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

fn print_group_header(label: &String) {
  let print_width = 80;
  let header = format!("= {} ", label);
  let divider = if header.len() < print_width {
    "=".repeat(print_width - header.len())
  } else {
    "".to_string()
  };
  let full_header = format!("{}{}", header, divider);
  println!("{}", full_header.blue());
}

fn print_banned(instance_group: &instance_group::InstanceGroup<'_>, actual: &String) {
  let icon = "✘".red();
  println!("      {} {} {}", icon, actual.red(), "[Banned]".dimmed());
}

fn print_ignored(instance_group: &instance_group::InstanceGroup<'_>, name: &String) {
  let count = render_count_column(instance_group.all.len());
  println!("{} {} {}", count, name.dimmed(), "[Ignored]".dimmed());
}

fn print_version_match(instance_group: &instance_group::InstanceGroup<'_>, name: &String) {
  let count = render_count_column(instance_group.all.len());
  let version = &instance_group.unique_specifiers.iter().join(" ");
  println!("{} {} {}", count, name, &version.dimmed());
}

fn print_pinned_version_mismatch(
  instance_group: &instance_group::InstanceGroup<'_>,
  actual: &String,
) {
  let icon = "✘".red();
  let arrow = "→".dimmed();
  let expected = instance_group.expected_version.as_ref().unwrap();
  println!(
    "      {} {} {} {} {}",
    icon,
    actual.red(),
    arrow,
    expected.green(),
    "[PinnedMismatch]".dimmed()
  );
}

fn print_lowest_version_mismatch(
  instance_group: &instance_group::InstanceGroup<'_>,
  actual: &String,
) {
  let icon = "✘".red();
  let arrow = "→".dimmed();
  let expected = instance_group.expected_version.as_ref().unwrap();
  println!(
    "      {} {} {} {} {}",
    icon,
    actual.red(),
    arrow,
    expected.green(),
    "[LowestSemverMismatch]".dimmed()
  );
}

fn print_highest_version_mismatch(
  instance_group: &instance_group::InstanceGroup<'_>,
  actual: &String,
) {
  let icon = "✘".red();
  let arrow = "→".dimmed();
  let expected = instance_group.expected_version.as_ref().unwrap();
  println!(
    "      {} {} {} {} {}",
    icon,
    actual.red(),
    arrow,
    expected.green(),
    "[HighestSemverMismatch]".dimmed()
  );
}

fn has_mismatches(instance_group: &instance_group::InstanceGroup<'_>) -> bool {
  instance_group.unique_specifiers.len() > (1 as usize)
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
    let outcome = match glob::glob(path_buf_to_str(&absolute_source)) {
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
        Err(x) => {
          panic!("Failed to read {:?}", &file_path.to_str());
        }
      })
      .collect(),
  )
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
