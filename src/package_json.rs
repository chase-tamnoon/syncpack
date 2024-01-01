use serde_json;
use std::fs;
use std::io;
use std::path;

#[derive(Debug)]
pub struct Package {
  /// The path to the package.json file
  pub file_path: path::PathBuf,
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

  /// Log the file path and parsed package.json
  pub fn pretty_print(&self) -> () {
    println!("{}: {:#?}", &self.file_path().display(), &self.contents);
  }
}

/// Read and parse a package.json file
pub fn read_file<P: AsRef<path::Path>>(file_path: &P) -> io::Result<Package> {
  let file_contents = fs::read_to_string(file_path)?;
  let parsed_json: serde_json::Value = serde_json::from_str(&file_contents)?;
  Ok(Package {
    contents: parsed_json,
    json: file_contents,
    file_path: file_path.as_ref().to_path_buf(),
  })
}
