use std::{cell::RefCell, collections::HashMap, path::PathBuf};

use serde_json::Value;

use crate::{
  cli::{Cli, CliOptions, Subcommand},
  config::{Config, Rcfile},
  package_json::PackageJson,
  packages::Packages,
  test::mock_effects::{EventsByType, MockEffects},
};

pub fn cli() -> Cli {
  Cli {
    command_name: Subcommand::Lint,
    options: CliOptions {
      filter: None,
      format: false,
      versions: true,
      source: vec![],
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

pub fn effects(config: &Config) -> MockEffects {
  MockEffects {
    config,
    events: EventsByType::new(),
    fixable_mismatches: HashMap::new(),
    is_valid: true,
    matches: HashMap::new(),
    overrides: HashMap::new(),
    packages: None,
    unfixable_mismatches: HashMap::new(),
    warnings: HashMap::new(),
    warnings_of_instance_changes: HashMap::new(),
  }
}
