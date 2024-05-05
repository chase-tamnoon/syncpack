use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path;

use crate::dependency_type::DependencyType;
use crate::instance;

#[derive(Clone, Debug)]
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
  pub fn get_instances<'a>(
    &'a self,
    enabled_dependency_types: &'a HashMap<String, DependencyType>,
    filter: &Regex,
  ) -> Vec<instance::Instance> {
    enabled_dependency_types
      .iter()
      .flat_map(|(name, dependency_type)| dependency_type.get_instances(&self, filter))
      .collect()
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

  /// Return a short path for logging to the terminal
  pub fn get_relative_file_path(&self, cwd: &std::path::PathBuf) -> String {
    self
      .file_path
      .strip_prefix(&cwd)
      .unwrap()
      .to_str()
      .unwrap()
      .to_string()
  }
}
