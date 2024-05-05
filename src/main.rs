#![allow(dead_code)]
#![allow(unused_variables)]

use colored::*;
use itertools::Itertools;
use log::debug;
use regex::Regex;
use std::{collections::HashMap, fs, io, path};

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
mod semver_group;
mod specifier;
mod version_group;
mod versions;

enum Subcommand {
  Lint,
  Fix,
}

fn main() -> io::Result<()> {
  env_logger::init();

  let subcommand = match cli::create().get_matches().subcommand() {
    Some(("lint", matches)) => (Subcommand::Lint, cli::get_enabled_steps(matches)),
    Some(("fix", matches)) => (Subcommand::Fix, cli::get_enabled_steps(matches)),
    _ => {
      debug!("@TODO: output --help when command is not recognised");
      std::process::exit(1);
    }
  };

  let cwd = std::env::current_dir()?.join("fixtures/fluid-framework");
  let rcfile = config::get(&cwd).expect("missing config file");

  debug!("rcfile: {:?}", &rcfile);

  let dependency_types = Rcfile::get_enabled_dependency_types(&rcfile);
  let semver_groups = SemverGroup::from_rcfile(&rcfile);
  let mut version_groups = VersionGroup::from_rcfile(&rcfile);
  let mut packages = get_packages(&cwd, &rcfile);
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

  let is_valid: bool = match subcommand {
    (Subcommand::Lint, enabled) => {
      lint_formatting(&cwd, &rcfile, &packages, &enabled);

      let header = match (enabled.ranges, enabled.versions) {
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
              })
          }
          VersionGroupVariant::Banned
          | VersionGroupVariant::Pinned
          | VersionGroupVariant::SameRange
          | VersionGroupVariant::SnappedTo => {
            print_group_header(&group.selector.label);
            group
              .instance_groups_by_name
              .iter()
              .for_each(|(name, instance_group)| {
                print_version_match(instance_group, name);
              })
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
              })
          }
        };
      });
      true
    }
    (Subcommand::Fix, enabled) => {
      println!("fix enabled {:?}", enabled);
      if enabled.format {
        println!("format packages");
        format::fix(&rcfile, &mut packages);
      }
      if enabled.versions {
        println!("fix versions");
        versions::fix(&cwd, &rcfile, &mut packages);
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
  enabled: &cli::EnabledSteps,
) -> bool {
  if !enabled.format {
    return true;
  }
  println!("{}", "= FORMATTING".yellow());
  let LintResult { valid, invalid } = format::lint(rcfile, packages);
  println!("{} valid", render_count_column(valid.len()));
  println!("{} invalid", render_count_column(invalid.len()));
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

fn print_ignored(instance_group: &instance_group::InstanceGroup<'_>, name: &String) {
  let count = render_count_column(instance_group.all.len());
  println!("{} {} {}", count, name.dimmed(), "[Ignored]".dimmed());
}

fn print_version_match(instance_group: &instance_group::InstanceGroup<'_>, name: &String) {
  let count = render_count_column(instance_group.all.len());
  let version = &instance_group.unique_specifiers.iter().join(" ");
  println!("{} {} {}", count, name, &version.dimmed());
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

fn get_packages(cwd: &path::PathBuf, rcfile: &Rcfile) -> Vec<package_json::PackageJson> {
  rcfile
    .get_sources(&cwd)
    .iter_mut()
    .filter_map(|file_path| json_file::read_json_file(&cwd, &file_path).ok())
    .collect()
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

/// Read and parse a package.json file
fn read_json_file<P: AsRef<path::Path>>(
  cwd: &std::path::PathBuf,
  file_path: &P,
) -> io::Result<package_json::PackageJson> {
  let json = fs::read_to_string(file_path)?;
  let contents: serde_json::Value = serde_json::from_str(&json)?;

  Ok(package_json::PackageJson {
    file_path: file_path.as_ref().to_path_buf(),
    json,
    contents,
  })
}
