#![allow(dead_code)]
#![allow(unused_variables)]

use colored::*;
use itertools::Itertools;
use log::debug;
use std::{collections::HashMap, fs, io, path};

use crate::{config::Rcfile, semver_group::SemverGroup, version_group::VersionGroup};

mod cli;
mod config;
mod dependency_type;
mod format;
mod group_selector;
mod instance;
mod package_json;
mod semver_group;
mod semver_ranges;
mod specifier;
mod version_group;
mod versions;

enum Subcommand {
  List,
  Lint,
  Fix,
}

// - [ ] when fixing, write to fixed_specifier_type/fixed_specifier on instance
fn main() -> io::Result<()> {
  env_logger::init();

  let subcommand = match cli::create().get_matches().subcommand() {
    Some(("list", matches)) => Some((Subcommand::List, None)),
    Some(("lint", matches)) => Some((Subcommand::Lint, Some(cli::get_enabled_steps(matches)))),
    Some(("fix", matches)) => Some((Subcommand::Fix, Some(cli::get_enabled_steps(matches)))),
    _ => None,
  };

  if subcommand.is_none() {
    debug!("@TODO: output --help when command is not recognised");
    std::process::exit(1);
  }

  let subcommand = subcommand.unwrap();
  let cwd = std::env::current_dir()?;
  let rcfile = config::get();

  debug!("rcfile: {:#?}", &rcfile);

  let dependency_types = Rcfile::get_enabled_dependency_types(&rcfile);
  let sources = rcfile.get_sources(&cwd);
  let semver_groups = SemverGroup::from_rcfile(&rcfile);
  let mut packages = get_packages(sources, &cwd);
  let mut version_groups = VersionGroup::from_rcfile(&rcfile);
  let mut instances = get_instances(&packages, &dependency_types);

  // assign every instance to the first group it matches
  instances.iter_mut().for_each(|instance| {
    semver_groups
      .iter()
      .any(|group| group.add_instance_if_eligible(instance));
    version_groups
      .iter_mut()
      .any(|group| group.add_instance_if_eligible(instance));
  });

  let is_valid: bool = match subcommand {
    (Subcommand::List, _) => {
      version_groups.iter().for_each(|group| {
        match group {
          VersionGroup::Banned(group) => {}
          VersionGroup::Ignored(group) => {}
          VersionGroup::Pinned(group) => {}
          VersionGroup::SameRange(group) => {}
          VersionGroup::SnappedTo(group) => {}
          VersionGroup::Standard(group) => {
            println!("{}", group.selector.label.blue());
            group
              .instances_by_name
              .iter()
              .for_each(|(name, instance_group)| {
                let count = format!("{: >4}x", instance_group.all.len()).dimmed();
                let size = &instance_group.unique_specifiers.len();
                if *size > (1 as usize) {
                  println!("  {} {}", count, name.red());
                  let preferred = instance_group.preferred_version.as_ref().unwrap();
                  instance_group
                    .unique_specifiers
                    .iter()
                    .for_each(|specifier| {
                      if specifier != preferred {
                        let icon = "✘".red();
                        let arrow = "→".dimmed();
                        println!(
                          "        {} {} {} {}",
                          icon,
                          specifier.red(),
                          arrow,
                          preferred.green()
                        );
                      }
                    });
                } else {
                  let versions = &instance_group.unique_specifiers.iter().join(" ");
                  println!("  {} {} {}", count, name, &versions.dimmed());
                };
              })
          }
        };
      });
      true
    }
    (Subcommand::Lint, some_enabled) => {
      let enabled = some_enabled.unwrap();
      let format_valid = !enabled.format || format::lint_all(&cwd, &rcfile, &mut packages);
      println!("format: {}", format_valid);
      let ranges_valid = !enabled.ranges || semver_ranges::lint_all(&cwd, &rcfile, &mut packages);
      println!("semver ranges: {}", ranges_valid);
      let versions_valid = !enabled.versions || versions::lint_all(&cwd, &rcfile, &mut packages);
      println!("versions: {}", versions_valid);
      format_valid && ranges_valid && versions_valid
    }
    (Subcommand::Fix, some_enabled) => {
      let enabled = some_enabled.unwrap();
      let format_valid = !enabled.format || format::fix_all(&cwd, &rcfile, &mut packages);
      println!("format: {}", format_valid);
      let ranges_valid = !enabled.ranges || semver_ranges::fix_all(&cwd, &rcfile, &mut packages);
      println!("semver ranges: {}", ranges_valid);
      let versions_valid = !enabled.versions || versions::fix_all(&cwd, &rcfile, &mut packages);
      println!("versions: {}", versions_valid);
      format_valid && ranges_valid && versions_valid
    }
  };

  if is_valid {
    std::process::exit(0);
  } else {
    std::process::exit(1);
  }
}

fn get_packages(
  mut sources: Vec<path::PathBuf>,
  cwd: &path::PathBuf,
) -> Vec<package_json::PackageJson> {
  sources
    .iter_mut()
    .filter_map(|file_path| read_file(&cwd, &file_path).ok())
    .collect()
}

fn get_instances<'a>(
  packages: &'a Vec<package_json::PackageJson>,
  dependency_types: &'a HashMap<String, dependency_type::DependencyType>,
) -> Vec<instance::Instance<'a>> {
  packages
    .iter()
    .flat_map(|package| package.get_instances(&dependency_types))
    .collect()
}

/// Read and parse a package.json file
fn read_file<P: AsRef<path::Path>>(
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

fn create_error(message: &str) -> io::Error {
  io::Error::new(io::ErrorKind::Other, message)
}
