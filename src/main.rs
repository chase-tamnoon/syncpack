#![allow(dead_code)]
#![allow(unused_variables)]

use colored::*;
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

// - [ ] when fixing, write to fixed_specifier_type/fixed_specifier on instance
// - [ ] don't create version groups etc when running format
fn main() -> io::Result<()> {
  env_logger::init();

  let cwd = std::env::current_dir()?;
  let rcfile = config::get();
  let dependency_types = Rcfile::get_enabled_dependency_types(&rcfile);
  let sources = rcfile.get_sources(&cwd);
  let semver_groups = SemverGroup::from_rcfile(&rcfile);
  let mut packages = get_packages(sources, &cwd);
  let mut version_groups = VersionGroup::from_rcfile(&rcfile);
  let mut instances = get_instances(&packages, &dependency_types);

  instances.iter_mut().for_each(|instance| {
    semver_groups
      .iter()
      .any(|semver_group| semver_group.add_instance(instance));
    version_groups
      .iter_mut()
      .any(|version_group| version_group.add_instance(instance));
  });

  debug!("rcfile: {:#?}", &rcfile);

  let is_valid: bool = match cli::create().get_matches().subcommand() {
    Some(("lint", matches)) => {
      let mut is_valid = true;
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        println!("{}", "Formatting".yellow());
        if !format::lint_all(&cwd, &rcfile, &mut packages) {
          is_valid = false;
        }
        debug!("@TODO: log whether formatting is valid or not");
      }
      if enabled_steps.ranges {
        println!("{}", "Semver Ranges".yellow());
        if !semver_ranges::lint_all(&cwd, &rcfile, &mut packages) {
          is_valid = false;
        }
        debug!("@TODO: log whether semver ranges match or not");
      }
      if enabled_steps.versions {
        println!("{}", "Versions".yellow());
        if !versions::lint_all(&cwd, &rcfile, &mut packages) {
          is_valid = false;
        }
        debug!("@TODO: log whether version mismatches are valid or not");
      }
      is_valid
    }
    Some(("fix", matches)) => {
      let mut is_valid = true;
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        println!("{}", "Formatting".yellow());
        if !format::fix_all(&cwd, &rcfile, &mut packages) {
          is_valid = false;
        }
        debug!("@TODO: log whether formatting was fixed or not");
      }
      if enabled_steps.ranges {
        println!("{}", "Semver Ranges".yellow());
        if !semver_ranges::fix_all(&cwd, &rcfile, &mut packages) {
          is_valid = false;
        }
        debug!("@TODO: log whether semver range mismatches were fixed or not");
      }
      if enabled_steps.versions {
        println!("{}", "Versions".yellow());
        if !versions::fix_all(&cwd, &rcfile, &mut packages) {
          is_valid = false;
        }
        debug!("@TODO: log whether version mismatches were fixed or not");
      }
      is_valid
    }
    _ => {
      debug!("@TODO: output --help when command is not recognised");
      false
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
