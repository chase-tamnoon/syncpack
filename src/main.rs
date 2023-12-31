// Standard library imports
use std::io;

// External crates
extern crate glob;
extern crate serde;
extern crate serde_json;

// Local modules
mod dependencies;
mod file_paths;
mod package_json;

// Imports from external crates
use serde_json::json;
use std::collections::BTreeMap;

fn main() -> io::Result<()> {
  let pattern = "/Users/foldleft/Dev/FluidFramework/package.json";
  let paths = file_paths::get_file_paths(pattern);

  paths
    .into_iter()
    .map(|file_path| package_json::read_file(&file_path))
    .filter_map(Result::ok)
    .for_each(|mut package| {
      package.set_prop("/name", json!("new name"));
      package.set_prop("/c8/cache-dir", json!("new cache-dir"));
      sort_json(&mut package.contents);
      package.pretty_print();
    });

  Ok(())
}

fn sort_json(value: &mut serde_json::Value) {
  match value {
    serde_json::Value::Object(obj) => {
      let sorted_obj: BTreeMap<String, serde_json::Value> = obj
        .into_iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
      obj.extend(sorted_obj);
    }
    serde_json::Value::Array(arr) => {
      arr.sort_by(|a, b| {
        a.as_str()
          .unwrap_or("")
          .partial_cmp(b.as_str().unwrap_or(""))
          .unwrap_or(std::cmp::Ordering::Equal)
      });
    }
    _ => {}
  }
}
