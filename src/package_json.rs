use serde_json::Value as JsonValue;
use std::fs;
use std::io;

#[derive(Debug)]
pub struct Package {
  pub data: JsonValue,
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
