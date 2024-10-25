use std::{cell::RefCell, path::PathBuf};

use serde_json::Value;

use crate::{
  cli::{Cli, CliOptions, Subcommand},
  config::{Config, Rcfile},
  package_json::PackageJson,
  packages::Packages,
};

pub fn cli() -> Cli {
  Cli {
    command_name: Subcommand::Lint,
    options: CliOptions {
      filter: None,
      format: false,
      log_levels: vec![],
      source: vec![],
      versions: true,
    },
  }
}

/// Create an empty Config struct
pub fn config() -> Config {
  Config {
    cli: cli(),
    cwd: std::env::current_dir().unwrap(),
    rcfile: rcfile(),
  }
}

/// Create a Config struct from a mocked .syncpackrc
pub fn config_from_mock(value: serde_json::Value) -> Config {
  Config {
    cli: cli(),
    cwd: std::env::current_dir().unwrap(),
    rcfile: rcfile_from_mock(value),
  }
}

/// Create an empty Rcfile struct
pub fn rcfile() -> Rcfile {
  let empty_json = "{}".to_string();
  serde_json::from_str::<Rcfile>(&empty_json).unwrap()
}

/// Create an Rcfile struct from a mocked .syncpackrc
pub fn rcfile_from_mock(value: serde_json::Value) -> Rcfile {
  serde_json::from_value::<Rcfile>(value).unwrap()
}

/// Parse a package.json string
pub fn package_json_from_value(contents: Value) -> PackageJson {
  PackageJson {
    file_path: PathBuf::new(),
    formatting_mismatches: RefCell::new(vec![]),
    json: RefCell::new(contents.to_string()),
    contents: RefCell::new(contents),
  }
}

/// Create an collection of package.json files from mocked values
pub fn packages_from_mocks(values: Vec<serde_json::Value>) -> Packages {
  let mut packages = Packages::new();
  for value in values {
    packages.add_package(package_json_from_value(value));
  }
  packages
}
