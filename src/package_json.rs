use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Packages {
  pub all_names: Vec<String>,
  pub by_name: HashMap<String, PackageJson>,
}

#[derive(Clone, Debug)]
pub struct PackageJson {
  /// The path to the package.json file
  pub file_path: PathBuf,
  /// The original, unedited raw JSON string
  pub json: String,
  /// The parsed JSON object
  pub contents: serde_json::Value,
}

impl PackageJson {
  /// Convenience method to get the name of the package
  pub fn get_name(&self) -> String {
    self
      .get_prop("/name")
      .and_then(|name| name.as_str())
      .unwrap_or("")
      .to_string()
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

  /// Return a short path for logging to the terminal
  pub fn get_relative_file_path(&self, cwd: &PathBuf) -> String {
    self
      .file_path
      .strip_prefix(&cwd)
      .ok()
      .map(|path| path.to_str().map(|path_str| path_str.to_string()))
      .flatten()
      .expect("Failed to create relative file path")
  }
}
