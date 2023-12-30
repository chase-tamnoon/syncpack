use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

pub fn read_and_parse_json<P: AsRef<Path>>(path: P) -> io::Result<Value> {
  let file_contents = fs::read_to_string(path)?;
  let parsed_json: Value = serde_json::from_str(&file_contents)?;
  Ok(parsed_json)
}

#[derive(Debug)]
pub struct Package {
  pub data: Value,
  pub json: String,
  pub path: String,
}

pub fn get_package(file_path: String) -> io::Result<Package> {
  fs::read_to_string(&file_path).and_then(|json| {
    serde_json::from_str(&json)
      .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
      .map(|data| Package {
        data,
        json,
        path: file_path,
      })
  })
}
