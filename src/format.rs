use icu::{
  collator::{Collator, CollatorOptions},
  locid::{locale, Locale},
};
use regex::Regex;
use serde_json::{self, json, Map, Value};
use std::{cmp::Ordering, collections::HashSet};

use crate::{config::Rcfile, package_json::PackageJson};

/// Packages have been formatted in memory, but not written to disk. This struct
/// describes what state each package was in prior to formatting.
pub struct InMemoryFormattingStatus<'a> {
  /// On disk, these packages have invalid formatting
  pub was_invalid: Vec<&'a PackageJson>,
  /// On disk, these packages have valid formatting
  pub was_valid: Vec<&'a PackageJson>,
}

/// Use a shorthand format for the bugs URL when possible
pub fn get_formatted_bugs(package: &PackageJson) -> Option<Value> {
  package.get_prop("/bugs/url").cloned()
}

/// Use a shorthand format for the repository URL when possible
pub fn get_formatted_repository(package: &PackageJson) -> Option<Value> {
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

/// Get sorted conditional exports and conditional exports subpaths
pub fn get_sorted_exports(rcfile: &Rcfile, package: &PackageJson) -> Option<Value> {
  /// Recursively visit and sort nested objects of the exports object
  fn sort_nested_objects(sort_exports: &Vec<String>, value: &mut Value) {
    if let Value::Object(obj) = value {
      sort_keys_with_priority(sort_exports, false, obj);
      for next_value in obj.values_mut() {
        sort_nested_objects(sort_exports, next_value);
      }
    }
  }
  if let Some(exports) = package.get_prop("/exports") {
    let mut sorted_exports = exports.clone();
    sort_nested_objects(&rcfile.sort_exports, &mut sorted_exports);
    if !is_identical(exports, &sorted_exports) {
      return Some(sorted_exports);
    }
  }
  None
}

/// Get a sorted version of the given property from package.json
pub fn get_sorted_az(key: &str, package: &PackageJson) -> Option<Value> {
  if let Some(value) = package.get_prop(format!("/{}", key).as_str()) {
    let mut sorted = value.clone();
    sort_alphabetically(&mut sorted);
    if !is_identical(value, &sorted) {
      return Some(sorted);
    }
  }
  None
}

/// Get a new package.json with its keys sorted to match the rcfile
pub fn get_sorted_first(rcfile: &Rcfile, package: &PackageJson) -> Option<Value> {
  if let Value::Object(value) = &package.contents {
    let mut sorted = value.clone();
    sort_keys_with_priority(&rcfile.sort_first, rcfile.sort_packages, &mut sorted);
    if !has_same_key_order(value, &sorted) {
      return Some(serde_json::Value::Object(sorted));
    }
  }
  None
}

/// Do both of these objects have the same order keys?
fn has_same_key_order(a: &Map<String, Value>, b: &Map<String, Value>) -> bool {
  let a_keys = a.keys().collect::<Vec<_>>();
  let b_keys = b.keys().collect::<Vec<_>>();
  a_keys == b_keys
}

/// Are these two values identical including their order?
#[allow(clippy::cmp_owned)]
fn is_identical(a: &Value, b: &Value) -> bool {
  // @TODO: serde_json with feature = "preserve_order" ignores order when compared
  a.to_string() == b.to_string()
}

/// Sort the keys in a JSON object, with the given keys first
///
/// # Parameters
///
/// * `order`: The keys to sort first, in order.
/// * `obj`: The JSON object to sort.
/// * `sort_remaining_keys`: Whether to sort the remaining keys alphabetically.
fn sort_keys_with_priority(order: &[String], sort_remaining_keys: bool, obj: &mut Map<String, Value>) {
  let order_set: HashSet<_> = order.iter().collect();
  let mut sorted_obj: Map<String, Value> = Map::new();
  let mut remaining_keys: Vec<_> = obj.keys().filter(|k| !order_set.contains(*k)).cloned().collect();

  if sort_remaining_keys {
    let collator = get_locale_collator();
    remaining_keys.sort_by(|a, b| collator.compare(a, b));
  }

  for (i, key) in order.iter().enumerate() {
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

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn formats_bugs_into_github_shorthand() {
    assert_eq!(
      get_formatted_bugs(&PackageJson::from_value(json!({
        "name": "a",
        "bugs": {
          "url": "https://github.com/User/repo/issues"
        }
      }))),
      Some(json!("https://github.com/User/repo/issues"))
    );
  }

  #[test]
  fn formats_repository_into_gitlab_shorthand() {
    assert_eq!(
      get_formatted_repository(&PackageJson::from_value(json!({
        "name": "a",
        "repository": {
          "url": "git://gitlab.com/User/repo",
          "type": "git",
        },
      }))),
      Some(json!("git://gitlab.com/User/repo"))
    );
  }

  #[test]
  fn formats_repository_into_github_shorthand() {
    assert_eq!(
      get_formatted_repository(&PackageJson::from_value(json!({
        "name": "a",
        "repository": {
          "url": "git://github.com/User/repo",
          "type": "git",
        },
      }))),
      Some(json!("User/repo"))
    );
  }

  #[test]
  fn retains_long_format_when_directory_property_used() {
    assert_eq!(
      get_formatted_repository(&PackageJson::from_value(json!({
        "name": "a",
        "repository": {
          "url": "git://gitlab.com/User/repo",
          "type": "git",
          "directory": "packages/foo",
        },
      }))),
      None
    );
  }

  #[test]
  fn sorts_conditional_exports() {
    assert_eq!(
      get_sorted_exports(
        &Rcfile::new(),
        &PackageJson::from_value(json!({
          "name": "a",
          "exports": {
              "require": "./index-require.cjs",
              "import": "./index-module.js",
          },
        }))
      ),
      Some(json!({
        "import": "./index-module.js",
        "require": "./index-require.cjs",
      })),
    )
  }

  #[test]
  fn returns_none_when_conditional_exports_already_sorted() {
    assert_eq!(
      get_sorted_exports(
        &Rcfile::new(),
        &PackageJson::from_value(json!({
          "name": "a",
          "exports": {
              "import": "./index-module.js",
              "require": "./index-require.cjs",
          },
        }))
      ),
      None
    )
  }

  #[test]
  fn sorts_conditional_exports_sub_paths() {
    assert_eq!(
      get_sorted_exports(
        &Rcfile::new(),
        &PackageJson::from_value(json!({
          "name": "a",
          "exports": {
            ".": "./index.js",
            "./feature.js": {
              "default": "./feature.js",
              "node": "./feature-node.js",
            },
          },
        }))
      ),
      Some(json!({
        ".": "./index.js",
        "./feature.js": {
          "node": "./feature-node.js",
          "default": "./feature.js",
        },
      })),
    )
  }

  #[test]
  fn returns_none_when_conditional_exports_sub_paths_already_sorted() {
    assert_eq!(
      get_sorted_exports(
        &Rcfile::new(),
        &PackageJson::from_value(json!({
          "name": "a",
          "exports": {
              ".": "./index.js",
              "./feature.js": {
                "node": "./feature-node.js",
                "default": "./feature.js",
              },
          },
        }))
      ),
      None
    )
  }

  #[test]
  fn sorts_object_properties_alphabetically_by_key() {
    assert_eq!(
      get_sorted_az(
        "dependencies",
        &PackageJson::from_value(json!({
            "dependencies": {
                "B": "",
                "@B": "",
                "1B": "",
                "A": "",
                "@A": "",
                "1A": "",
            },
        }))
      ),
      Some(json!({
          "@A": "",
          "@B": "",
          "1A": "",
          "1B": "",
          "A": "",
          "B": "",
      }))
    );
  }
  #[test]
  fn sorts_array_members_alphabetically_by_value() {
    assert_eq!(
      get_sorted_az(
        "keywords",
        &PackageJson::from_value(json!({
            "keywords": ["B", "@B", "1B", "A", "@A", "1A"],
        }))
      ),
      Some(json!(["@A", "@B", "1A", "1B", "A", "B"]))
    );
  }

  #[test]
  fn sorts_named_root_properties_first_leaving_the_rest_alone() {
    assert_eq!(
      get_sorted_first(
        &Rcfile::from_mock(json!({
            "sortFirst": ["name", "F", "E", "D"],
            "sortPackages": false,
        })),
        &PackageJson::from_value(json!({
            "D": "",
            "B": "",
            "name": "a",
            "F": "",
            "A": "",
            "E": "",
        }))
      ),
      Some(json!({
          "name": "a",
          "F": "",
          "E": "",
          "D": "",
          "B": "",
          "A": "",
      }))
    );
  }

  #[test]
  fn sorts_all_root_properties_alphabetically() {
    assert_eq!(
      get_sorted_first(
        &Rcfile::from_mock(json!({
            "sortFirst": [],
            "sortPackages": true,
        })),
        &PackageJson::from_value(json!({
            "D": "",
            "B": "",
            "name": "a",
            "F": "",
            "A": "",
            "E": "",
        }))
      ),
      Some(json!({
          "A": "",
          "B": "",
          "D": "",
          "E": "",
          "F": "",
          "name": "a",
      }))
    );
  }

  #[test]
  fn sorts_named_properties_first_then_the_rest_alphabetically() {
    assert_eq!(
      get_sorted_first(
        &Rcfile::from_mock(json!({
            "sortFirst": ["name", "F", "E", "D"],
            "sortPackages": true,
        })),
        &PackageJson::from_value(json!({
            "name": "a",
            "A": "",
            "F": "",
            "B": "",
            "D": "",
            "E": "",
        }))
      ),
      Some(json!({
          "name": "a",
          "F": "",
          "E": "",
          "D": "",
          "A": "",
          "B": "",
      }))
    );
  }
}
