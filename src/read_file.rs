use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn read_file(file_path: &str) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn read_package_json(file_path: &str) -> std::io::Result<Value> {
    let json = read_file(file_path)?;
    let json_str: &str = &json[..];
    let res: Value = serde_json::from_str(json_str)?;
    Ok(res)
}
