use colored::Colorize;
use serde_json;
use std::collections;
use std::fs;
use std::io;
use std::path;
use std::vec;

use crate::instance;
use crate::strategy;
use crate::strategy::StrategyTrait;

#[derive(Debug)]
pub struct PackageJson {
  /// The path to the package.json file
  pub file_path: path::PathBuf,
  /// The original, unedited raw JSON string
  pub json: String,
  /// The parsed JSON object
  pub contents: serde_json::Value,
}

impl PackageJson {
  /// Create an instance for every enabled dependency type
  pub fn get_instances(
    &self,
    enabled_dependency_types: &collections::HashMap<String, strategy::Strategy>,
  ) -> Vec<instance::Instance> {
    let mut instances: Vec<instance::Instance> = vec![];

    for (type_name, type_strategy) in enabled_dependency_types {
      match type_strategy {
        strategy::Strategy::NameAndVersionProps(ref strategy) => {
          let instances_for_dependency_type = strategy.read(&self);
          instances.extend(instances_for_dependency_type);
        }
        strategy::Strategy::NamedVersionString(ref strategy) => {
          let instances_for_dependency_type = strategy.read(&self);
          instances.extend(instances_for_dependency_type);
        }
        strategy::Strategy::UnnamedVersionString(ref strategy) => {
          let instances_for_dependency_type = strategy.read(&self);
          instances.extend(instances_for_dependency_type);
        }
        strategy::Strategy::VersionsByName(ref strategy) => {
          let instances_for_dependency_type = strategy.read(&self);
          instances.extend(instances_for_dependency_type);
        }
      };
    }

    instances
  }

  /// Deeply get a property in the parsed package.json
  pub fn get_prop(&self, pointer: &str) -> Option<&serde_json::Value> {
    self.contents.pointer(pointer)
  }

  /// Deeply get a property in the parsed package.json as mutable
  pub fn get_prop_mut(&mut self, pointer: &str) -> Option<&mut serde_json::Value> {
    self.contents.pointer_mut(pointer)
  }

  /// Deeply set a property in the parsed package.json
  pub fn set_prop(&mut self, pointer: &str, next_value: serde_json::Value) {
    if let Some(value) = self.contents.pointer_mut(pointer) {
      *value = next_value;
    }
  }

  /// Report whether the package in memory has changed from what's on disk
  pub fn has_changed(&self) -> bool {
    self.json != self.contents.to_string()
  }

  /// Write the parsed package.json to disk
  pub fn write_to_disk(&self) -> io::Result<()> {
    fs::write(&self.file_path, self.contents.to_string())
  }

  pub fn log_as_valid(&self, cwd: &std::path::PathBuf) -> () {
    println!("{} {}", "✓".green(), self.get_short_path(cwd));
  }

  pub fn log_as_invalid(&self, cwd: &std::path::PathBuf) -> () {
    println!("{} {}", "✘".red(), self.get_short_path(cwd));
  }

  fn get_short_path(&self, cwd: &std::path::PathBuf) -> String {
    self
      .file_path
      .strip_prefix(&cwd)
      .unwrap()
      .to_str()
      .unwrap()
      .to_string()
  }
}
