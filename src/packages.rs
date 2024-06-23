use glob::glob;
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

use crate::{
  cli::CliOptions,
  config::{Config, Rcfile},
  dependency_type::Strategy,
  instance::Instance,
  package_json::PackageJson,
};

#[derive(Debug)]
pub struct Packages {
  pub all_names: Vec<String>,
  pub by_name: HashMap<String, PackageJson>,
}

impl Packages {
  /// Create an empty collection of package.json files
  pub fn new() -> Self {
    Self { all_names: vec![], by_name: HashMap::new() }
  }

  #[cfg(test)]
  pub fn from_mocks(values: Vec<Value>) -> Self {
    let mut packages = Self::new();
    for value in values {
      packages.add_package(PackageJson::from_value(value));
    }
    packages
  }

  /// Get every package.json file matched by the user's source patterns
  pub fn from_config(config: &Config) -> Self {
    let file_paths = get_file_paths(config);
    let mut packages = Self::new();
    file_paths.iter().for_each(|file_path| {
      PackageJson::from_file(&file_path).map(|package_json| {
        packages.add_package(package_json);
      });
    });
    packages
  }

  /// Add a package.json file to this collection
  pub fn add_package(&mut self, package_json: PackageJson) -> &mut Self {
    let name = package_json.get_name();
    self.all_names.push(name.clone());
    self.by_name.insert(name, package_json);
    self
  }

  /// Get every instance of a dependency from every package.json file
  pub fn get_all_instances<F>(&self, config: &Config, mut on_instance: F)
  where
    F: FnMut(Instance),
  {
    let dependency_types = &config.rcfile.get_enabled_dependency_types();
    let filter = &config.cli.options.filter;
    let matches_filter = |name: &str| -> bool {
      if let Some(filter) = filter {
        filter.is_match(name)
      } else {
        true
      }
    };

    for package in self.by_name.values() {
      for dependency_type in dependency_types {
        match dependency_type.strategy {
          Strategy::NameAndVersionProps => {
            if let (Some(Value::String(name)), Some(Value::String(raw_specifier))) = (package.get_prop(&dependency_type.name_path.as_ref().unwrap()), package.get_prop(&dependency_type.path)) {
              if matches_filter(name) {
                on_instance(Instance::new(name.to_string(), raw_specifier.to_string(), &dependency_type, &package));
              }
            }
          }
          Strategy::NamedVersionString => {
            if let Some(Value::String(specifier)) = package.get_prop(&dependency_type.path) {
              if let Some((name, raw_specifier)) = specifier.split_once('@') {
                if matches_filter(name) {
                  on_instance(Instance::new(name.to_string(), raw_specifier.to_string(), &dependency_type, &package));
                }
              }
            }
          }
          Strategy::UnnamedVersionString => {
            if let Some(Value::String(raw_specifier)) = package.get_prop(&dependency_type.path) {
              if matches_filter(&dependency_type.name) {
                on_instance(Instance::new(dependency_type.name.clone(), raw_specifier.to_string(), &dependency_type, &package));
              }
            }
          }
          Strategy::VersionsByName => {
            if let Some(Value::Object(versions_by_name)) = package.get_prop(&dependency_type.path) {
              for (name, raw_specifier) in versions_by_name {
                if matches_filter(name) {
                  if let Value::String(version) = raw_specifier {
                    on_instance(Instance::new(name.to_string(), version.to_string(), &dependency_type, &package));
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
  }
}

/// Resolve every source glob pattern into their absolute file paths of
/// package.json files
fn get_file_paths(config: &Config) -> Vec<PathBuf> {
  get_source_patterns(config)
    .iter()
    .map(|pattern| {
      if PathBuf::from(pattern).is_absolute() {
        pattern.clone()
      } else {
        config.cwd.join(pattern).to_str().unwrap().to_string()
      }
    })
    .flat_map(|pattern| glob(&pattern).ok())
    .flat_map(|paths| {
      paths.filter_map(Result::ok).fold(vec![], |mut paths, path| {
        paths.push(path.clone());
        paths
      })
    })
    .collect()
}

/// Based on the user's config file and command line `--source` options, return
/// the source glob patterns which should be used to resolve package.json files
fn get_source_patterns(config: &Config) -> Vec<String> {
  get_cli_patterns(&config.cli.options)
    .or_else(|| get_rcfile_patterns(&config.rcfile))
    .or_else(get_npm_patterns)
    .or_else(get_pnpm_patterns)
    .or_else(get_yarn_patterns)
    .or_else(get_lerna_patterns)
    .or_else(get_default_patterns)
    .unwrap()
}

fn get_cli_patterns(cli_options: &CliOptions) -> Option<Vec<String>> {
  if cli_options.source.is_empty() {
    return None;
  } else {
    return Some(cli_options.source.clone());
  }
}

fn get_rcfile_patterns(rcfile: &Rcfile) -> Option<Vec<String>> {
  if rcfile.source.is_empty() {
    return None;
  } else {
    return Some(rcfile.source.clone());
  }
}

fn get_npm_patterns() -> Option<Vec<String>> {
  None
}

fn get_pnpm_patterns() -> Option<Vec<String>> {
  None
}

fn get_yarn_patterns() -> Option<Vec<String>> {
  None
}

fn get_lerna_patterns() -> Option<Vec<String>> {
  None
}

fn get_default_patterns() -> Option<Vec<String>> {
  Some(vec![String::from("package.json"), String::from("packages/*/package.json")])
}
