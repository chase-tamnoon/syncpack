use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Package {
  /// The parsed JSON object
  pub contents: Value,
  /// The original, unedited raw JSON string
  pub json: String,
  /// The path to the package.json file
  pub file_path: PathBuf,
}

impl Package {
  /// Get the absolute path to the package.json file
  pub fn file_path(&self) -> &Path {
    &self.file_path
  }

  /// Deeply set a property in the parsed package.json
  pub fn set_prop(&mut self, pointer: &str, next_value: serde_json::Value) {
    if let Some(value) = self.contents.pointer_mut(pointer) {
      *value = next_value;
    }
  }

  /// Log the file path and parsed package.json
  pub fn pretty_print(&self) -> () {
    println!("{}: {:#?}", &self.file_path.display(), &self.contents);
  }
}

/// Read and parse a package.json file, returning a Package which can be freely mutated
pub fn read_file<P: AsRef<Path>>(file_path: &P) -> io::Result<Package> {
  let file_contents = fs::read_to_string(file_path)?;
  let parsed_json: Value = serde_json::from_str(&file_contents)?;
  Ok(Package {
    contents: parsed_json,
    json: file_contents,
    file_path: file_path.as_ref().to_path_buf(),
  })
}
