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

/// Read and parse a package.json file
pub fn read_file<P: AsRef<Path>>(file_path: P) -> io::Result<Package> {
  let file_contents = fs::read_to_string(file_path)?;
  let parsed_json: Value = serde_json::from_str(&file_contents)?;
  Ok(Package {
    contents: parsed_json,
    json: file_contents,
    file_path: file_path.as_ref().to_path_buf(),
  })
}
