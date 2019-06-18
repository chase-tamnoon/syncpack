use serde_json::Value as JsonValue;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn read_file(file_path: &String) -> std::io::Result<String> {
  let file = File::open(file_path)?;
  let mut buf_reader = BufReader::new(file);
  let mut contents = String::new();
  buf_reader.read_to_string(&mut contents)?;
  Ok(contents)
}

fn read_package_json(file_path: &String) -> std::io::Result<JsonValue> {
  let json = read_file(&file_path)?;
  let json_str: &str = &json[..];
  let res: JsonValue = serde_json::from_str(json_str)?;
  Ok(res)
}

pub struct Package {
  pub data: JsonValue,
  pub path: String,
}

pub fn get_package(file_path: String) -> std::io::Result<Package> {
  let data: JsonValue = read_package_json(&file_path)?;
  Ok(Package {
    data: data,
    path: file_path,
  })
}
