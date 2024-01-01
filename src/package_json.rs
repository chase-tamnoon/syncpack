use serde_json;
use std::fs;
use std::io;
use std::path;

#[derive(Debug)]
pub struct Package {
  /// The path to the package.json file
  pub file_path: path::PathBuf,
  /// Relative path to the package.json file
  pub short_path: String,
  /// The original, unedited raw JSON string
  pub json: String,
  /// The parsed JSON object
  pub contents: serde_json::Value,
}

impl Package {
  /// Get the absolute path to the package.json file
  pub fn file_path(&self) -> &path::Path {
    &self.file_path
  }

  /// Deeply get a property in the parsed package.json
  pub fn get_prop(&self, pointer: &str) -> Option<&serde_json::Value> {
    self.contents.pointer(pointer)
  }

  /// Deeply get a property in the parsed package.json as mutable
  pub fn get_prop_mut(
    &mut self,
    pointer: &str,
  ) -> Option<&mut serde_json::Value> {
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

  /// Log the file path and parsed package.json
  pub fn pretty_print(&self) -> () {
    println!("{}: {:#?}", &self.file_path().display(), &self.contents);
  }
}
