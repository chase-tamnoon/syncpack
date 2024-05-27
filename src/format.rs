use icu::{
  collator::{Collator, CollatorOptions},
  locid::{locale, Locale},
};
use regex::Regex;
use serde_json::{self, json, Map, Value};
use std::{cmp::Ordering, collections::HashSet};

use crate::{
  config::Rcfile,
  package_json::{PackageJson, Packages},
};

pub struct LintResult<'a> {
  pub invalid: Vec<&'a PackageJson>,
  pub valid: Vec<&'a PackageJson>,
}

/// Check whether every package is formatted according to config
pub fn lint<'a>(rcfile: &'a Rcfile, packages: &'a mut Packages) -> LintResult<'a> {
  let mut lint_result = LintResult {
    invalid: Vec::new(),
    valid: Vec::new(),
  };
  packages.by_name.values_mut().for_each(|package| {
    // to lint, apply all configured formatting to the clone...
    fix_package(&rcfile, package);
    // ...and if it has changed we know it is invalid
    if package.has_changed(&rcfile.indent) {
      lint_result.invalid.push(package);
    } else {
      lint_result.valid.push(package);
    }
  });
  lint_result
}

/// Format every package according to config
pub fn fix(rcfile: &Rcfile, packages: &mut Vec<PackageJson>) {
  packages.iter_mut().for_each(|package| {
    fix_package(&rcfile, package);
  });
}

fn fix_package(rcfile: &Rcfile, package: &mut PackageJson) {
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
  order: &Vec<String>,
  obj: &mut Map<String, Value>,
  sort_remaining_keys: bool,
) {
  let order_set: HashSet<_> = order.into_iter().collect();
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
      *value = Value::Object(Map::from_iter(entries.into_iter()));
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
  if let Some(bugs_url) = package.get_prop("/bugs/url") {
    package.set_prop("/bugs", bugs_url.clone());
  };
}

/// Use a shorthand format for the repository URL when possible
fn format_repository(package: &mut PackageJson) {
  if package.get_prop("/repository/directory").is_none() {
    package
      .get_prop("/repository/url")
      .and_then(|repository_url| repository_url.as_str())
      .and_then(|url| {
        Regex::new(r#".+github\.com/"#)
          .ok()
          .map(|re| re.replace(url, "").to_string())
      })
      .map(|next_url| package.set_prop("/repository", json!(next_url)));
  }
}
