use icu::{
  collator::{Collator, CollatorOptions},
  locid::{locale, Locale},
};
use regex::Regex;
use serde_json::{self, json, Map, Value};
use std::{cmp::Ordering, collections::HashSet};

use crate::{
  config::{Config, Rcfile},
  package_json::PackageJson,
  packages::Packages,
};

/// Packages have been formatted in memory, but not written to disk. This struct
/// describes what state each package was in prior to formatting.
pub struct InMemoryFormattingStatus<'a> {
  /// On disk, these packages have invalid formatting
  pub was_invalid: Vec<&'a PackageJson>,
  /// On disk, these packages have valid formatting
  pub was_valid: Vec<&'a PackageJson>,
}

/// Fix the formatting of every package in-memory and report on their status
pub fn fix<'a>(config: &'a Config, packages: &'a mut Packages) -> InMemoryFormattingStatus<'a> {
  let mut status = InMemoryFormattingStatus {
    was_invalid: Vec::new(),
    was_valid: Vec::new(),
  };
  packages.by_name.values_mut().for_each(|package| {
    // to lint, apply all configured formatting to the clone...
    fix_package(&config.rcfile, package);
    // ...and if it has changed we know it is invalid
    if package.has_changed(&config.rcfile.indent) {
      status.was_invalid.push(package);
    } else {
      status.was_valid.push(package);
    }
  });
  status
}

fn fix_package(rcfile: &Rcfile, package: &mut PackageJson) {
  if rcfile.format_bugs {
    format_bugs(package);
  }
  if rcfile.format_repository {
    format_repository(package);
  }
  if !rcfile.sort_az.is_empty() {
    sort_az(rcfile, package);
  }
  if !rcfile.sort_first.is_empty() {
    sort_first(rcfile, package);
  }
  if !rcfile.sort_exports.is_empty() {
    sort_exports(rcfile, package);
  }
}

/// Sorts conditional exports and conditional exports subpaths
fn sort_exports(rcfile: &Rcfile, package: &mut PackageJson) {
  if let Some(exports) = package.get_prop_mut("/exports") {
    visit_node(&rcfile.sort_exports, exports);
  }

  /// Recursively visit and sort nested objects of the exports object
  fn visit_node(sort_exports: &Vec<String>, value: &mut Value) {
    if let Value::Object(obj) = value {
      sort_keys_with_priority(sort_exports, obj, false);
      for next_value in obj.values_mut() {
        visit_node(sort_exports, next_value);
      }
    }
  }
}

/// Sort the values of the given keys alphabetically
fn sort_az(rcfile: &Rcfile, package: &mut PackageJson) {
  rcfile.sort_az.iter().for_each(|key| {
    package
      .contents
      .pointer_mut(format!("/{}", key).as_str())
      .map(sort_alphabetically);
  });
}

/// Sort package.json with the given keys first
fn sort_first(rcfile: &Rcfile, package: &mut PackageJson) {
  if let Value::Object(obj) = &mut package.contents {
    sort_keys_with_priority(&rcfile.sort_first, obj, rcfile.sort_packages);
  }
}

/// Sort the keys in a JSON object, with the given keys first
///
/// # Parameters
///
/// * `order`: The keys to sort first, in order.
/// * `obj`: The JSON object to sort.
/// * `sort_remaining_keys`: Whether to sort the remaining keys alphabetically.
fn sort_keys_with_priority(
  order: &[String],
  obj: &mut Map<String, Value>,
  sort_remaining_keys: bool,
) {
  let order_set: HashSet<_> = order.iter().collect();
  let mut sorted_obj: Map<String, Value> = Map::new();
  let mut remaining_keys: Vec<_> = obj
    .keys()
    .filter(|k| !order_set.contains(*k))
    .cloned()
    .collect();

  if sort_remaining_keys {
    let collator = get_locale_collator();
    remaining_keys.sort_by(|a, b| collator.compare(a, b));
  }

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

/// Sort an array or object alphabetically by EN locale
fn sort_alphabetically(value: &mut Value) {
  let collator = get_locale_collator();
  match value {
    Value::Object(obj) => {
      let mut entries: Vec<_> = obj.clone().into_iter().collect();
      entries.sort_by(|a, b| collator.compare(&a.0, &b.0));
      *value = Value::Object(Map::from_iter(entries));
    }
    Value::Array(arr) => {
      arr.sort_by(|a, b| {
        if let (Some(a), Some(b)) = (a.as_str(), b.as_str()) {
          collator.compare(a, b)
        } else {
          Ordering::Equal
        }
      });
    }
    _ => {}
  }
}

/// Get a collator for the EN locale to sort strings
fn get_locale_collator() -> Collator {
  let locale_en: Locale = locale!("en");
  let options = CollatorOptions::new();
  let collator: Collator = Collator::try_new(&locale_en.into(), options).unwrap();
  collator
}

/// Use a shorthand format for the bugs URL when possible
fn format_bugs(package: &mut PackageJson) {
  if let Some(bugs) = get_formatted_bugs(package) {
    package.set_prop("/bugs", bugs);
  }
}

fn get_formatted_bugs(package: &PackageJson) -> Option<Value> {
  package.get_prop("/bugs/url").cloned()
}

fn format_bugs_is_valid(package: &PackageJson) -> bool {
  get_formatted_bugs(package).is_none()
}

/// Use a shorthand format for the repository URL when possible
fn format_repository(package: &mut PackageJson) {
  if let Some(bugs) = get_formatted_repository(package) {
    package.set_prop("/repository", bugs);
  }
}

fn get_formatted_repository(package: &PackageJson) -> Option<Value> {
  if package.get_prop("/repository/directory").is_none() {
    package
      .get_prop("/repository/url")
      .and_then(|repository_url| repository_url.as_str())
      .and_then(|url| {
        Regex::new(r#".+github\.com/"#)
          .ok()
          .map(|re| re.replace(url, "").to_string())
      })
      .map(|next_url| json!(next_url))
  } else {
    None
  }
}

fn format_repository_is_valid(package: &PackageJson) -> bool {
  get_formatted_repository(package).is_none()
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn format_repository() {
    let packages = Packages::from_mocks(vec![json!({
      "name": "a",
      "bugs": {
        "url": "https://github.com/User/repo/issues"
      }
    })]);
    let package = packages.by_name.get("a").unwrap();

    assert_eq!(
      get_formatted_bugs(package),
      Some(json!("https://github.com/User/repo/issues"))
    );
  }
}
