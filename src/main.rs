// Standard library imports
use std::{io, collections::HashSet};

// External crates
extern crate glob;
extern crate serde;
extern crate serde_json;

// Local modules
mod dependencies;
mod file_paths;
mod package_json;

// Imports from external crates
use serde_json::{json, Map};

fn main() -> io::Result<()> {
  let pattern = "/Users/foldleft/Dev/tightrope/package.json";
  let paths = file_paths::get_file_paths(pattern);

  paths
    .into_iter()
    .map(|file_path| package_json::read_file(&file_path))
    .filter_map(Result::ok)
    .for_each(|mut package| {
      package.set_prop("/name", json!("new name"));
      package.set_prop("/c8/cache-dir", json!("new cache-dir"));
      sort_json(&mut package.contents);
      sort_first(
        &mut package.contents,
        &vec![
          "private".to_string(),
          "homepage".to_string(),
          "name".to_string(),
          "version".to_string(),
          "description".to_string(),
        ],
      );
      package.pretty_print();
    });

  Ok(())
}

/// Sort the keys in a JSON object, with the given keys first
fn sort_first(value: &mut serde_json::Value, order: &Vec<String>) {
  match value {
    serde_json::Value::Object(obj) => {
      let order_set: HashSet<_> = order.into_iter().collect();
      let mut sorted_obj: Map<String, serde_json::Value> = Map::new();
      let mut remaining_keys: Vec<_> = obj
        .keys()
        .filter(|k| !order_set.contains(*k))
        .cloned()
        .collect();

      remaining_keys.sort();

      for key in order.clone() {
        if let Some(val) = obj.remove(&key) {
          sorted_obj.insert(key, val);
        }
      }

      for key in remaining_keys {
        if let Some(val) = obj.remove(&key) {
          sorted_obj.insert(key, val);
        }
      }

      *value = serde_json::Value::Object(sorted_obj);
    }
    _ => {}
  }
}

fn sort_json(value: &mut serde_json::Value) {
  match value {
    serde_json::Value::Object(obj) => {
      let mut entries: Vec<_> =
        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
      entries.sort_by(|a, b| a.0.cmp(&b.0));
      let sorted_obj: Map<String, serde_json::Value> =
        entries.into_iter().collect();

      *value = serde_json::Value::Object(sorted_obj);
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
