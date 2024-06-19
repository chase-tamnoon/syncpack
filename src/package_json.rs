use log::error;
use serde::Serialize;
use serde_json::{ser::PrettyFormatter, Serializer};
use std::{fs, path::PathBuf};

use crate::config::Config;

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
  /// Parse a package.json string
  pub fn from_value(contents: serde_json::Value) -> Self {
    Self {
      file_path: PathBuf::new(),
      json: contents.to_string(),
      contents,
    }
  }

  /// Read a package.json file from the given location
  pub fn from_file(file_path: &PathBuf) -> Option<Self> {
    fs::read_to_string(&file_path)
      .inspect_err(|_| {
        error!("package.json not readable at {}", &file_path.to_str().unwrap());
      })
      .ok()
      .and_then(|json| {
        serde_json::from_str(&json)
          .inspect_err(|_| {
            error!("file is not valid JSON at {}", &file_path.to_str().unwrap());
          })
          .map(|contents| Self {
            file_path: file_path.clone(),
            json,
            contents,
          })
          .ok()
      })
  }

  /// Convenience method to get the name of the package
  pub fn get_name(&self) -> String {
    self
      .get_prop("/name")
      .and_then(|name| name.as_str())
      .expect("package.json file has no .name property")
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
  pub fn has_changed(&self, indent: &String) -> bool {
    self.json != self.to_pretty_json(self.serialize(indent))
  }

  /// Serialize the parsed JSON object back into pretty JSON as bytes
  pub fn serialize(&self, indent: &String) -> Vec<u8> {
    // Create a pretty JSON formatter
    let indent_with_fixed_tabs = &indent.clone().replace("\\t", "\t");
    let formatter = PrettyFormatter::with_indent(indent_with_fixed_tabs.as_bytes());
    let buffer = Vec::new();
    let mut serializer = Serializer::with_formatter(buffer, formatter);
    // Write pretty JSON to the buffer
    self.contents.serialize(&mut serializer).expect("Failed to serialize package.json");
    // Append a new line to the buffer
    let mut writer = serializer.into_inner();
    writer.extend(b"\n");
    writer
  }

  /// Convert a buffer of pretty JSON as bytes to a pretty JSON string
  pub fn to_pretty_json(&self, vec: Vec<u8>) -> String {
    let from_utf8 = String::from_utf8(vec);
    from_utf8.expect("Failed to convert JSON buffer to string")
  }

  /// Write the package.json to disk
  pub fn write_to_disk(&mut self, config: &Config) {
    let vec = self.serialize(&config.rcfile.indent);
    std::fs::write(&self.file_path, &vec).expect("Failed to write package.json to disk");
    self.json = self.to_pretty_json(vec);
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
