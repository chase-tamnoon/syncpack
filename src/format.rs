use regex::Regex;
use serde_json;
use std::collections;

use crate::config;
use crate::package_json;

/// Format a package.json file
pub fn format_package(
  package: &mut package_json::Package,
  rcfile: &config::RcFile,
) {
  if rcfile.format_bugs {
    format_bugs(package);
  }

  if rcfile.format_repository {
    format_repository(package);
  }

  if rcfile.sort_az.len() > 0 {
    sort_az(rcfile, package);
  }

  if rcfile.sort_first.len() > 0 {
    sort_first(rcfile, package);
  }

  if rcfile.sort_exports.len() > 0 {
    sort_exports(rcfile, package);
  }
}

/// Sorts conditional exports and conditional exports subpaths
fn sort_exports(rcfile: &config::RcFile, package: &mut package_json::Package) {
  if let Some(exports) = package.get_prop_mut("/exports") {
    visit_node(&rcfile.sort_exports, exports);
  }

  /// Recursively visit and sort nested objects of the exports object
  fn visit_node(sort_exports: &Vec<String>, value: &mut serde_json::Value) {
    if let serde_json::Value::Object(obj) = value {
      sort_object_first(sort_exports, obj, false);
      for next_value in obj.values_mut() {
        visit_node(sort_exports, next_value);
      }
    }
  }
}

/// Sort the values of the given keys alphabetically
fn sort_az(rcfile: &config::RcFile, package: &mut package_json::Package) {
  rcfile.sort_az.iter().for_each(|key| {
    package
      .contents
      .pointer_mut(format!("/{}", key).as_str())
      .map(sort_alphabetically);
  });
}

/// Sort package.json with the given keys first
pub fn sort_first(
  rcfile: &config::RcFile,
  package: &mut package_json::Package,
) {
  if let serde_json::Value::Object(obj) = &mut package.contents {
    sort_object_first(&rcfile.sort_first, obj, rcfile.sort_packages);
  }
}

/// Sort the keys in a JSON object, with the given keys first
///
/// # Parameters
///
/// * `order`: The keys to sort first, in order.
/// * `obj`: The JSON object to sort.
/// * `sort_remaining_keys`: Whether to sort the remaining keys alphabetically.
pub fn sort_object_first(
  order: &Vec<String>,
  obj: &mut serde_json::Map<String, serde_json::Value>,
  sort_remaining_keys: bool,
) {
  let order_set: collections::HashSet<_> = order.into_iter().collect();
  let mut sorted_obj: serde_json::Map<String, serde_json::Value> =
    serde_json::Map::new();
  let mut remaining_keys: Vec<_> = obj
    .keys()
    .filter(|k| !order_set.contains(*k))
    .cloned()
    .collect();

  if sort_remaining_keys {
    remaining_keys.sort();
  }

  // for key in order.clone() {
  //   if let Some(val) = obj.remove(&key) {
  //     sorted_obj.insert(key, val);
  //   }
  // }
  for key in order.iter() {
    if let Some(val) = obj.remove(key) {
      sorted_obj.insert(key.clone(), val);
    }
  }

  for key in remaining_keys {
    if let Some(val) = obj.remove(&key) {
      sorted_obj.insert(key, val);
    }
  }

  *obj = sorted_obj;
}

/// Sort an array or object alphabetically
pub fn sort_alphabetically(value: &mut serde_json::Value) {
  match value {
    serde_json::Value::Object(obj) => {
      let mut entries: Vec<_> =
        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
      entries.sort_by(|a, b| a.0.cmp(&b.0));
      let sorted_obj: serde_json::Map<String, serde_json::Value> =
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

/// Use a shorthand format for the bugs URL when possible
fn format_bugs(package: &mut package_json::Package) {
  let bugs_url = package.get_prop("/bugs/url");
  if let Some(bugs_url) = bugs_url {
    package.set_prop("/bugs", bugs_url.clone());
  }
}

/// Use a shorthand format for the repository URL when possible
pub fn format_repository(package: &mut package_json::Package) {
  if package.get_prop("/repository/directory").is_none() {
    if let Some(repository_url) = package.get_prop("/repository/url") {
      if let Some(url) = repository_url.as_str() {
        let re = Regex::new(r#".+github\.com/"#).unwrap();
        let next_url = re.replace(&url, "").to_string();
        package.set_prop("/repository", serde_json::json!(next_url));
      }
    }
  }
}
