use std::collections::HashMap;

use serde::Deserialize;

use crate::config;
use crate::strategy;

#[derive(Debug)]
pub struct NameAndVersionPropsStrategy {
  pub name: String,
  pub path: String,
  pub name_path: String,
}

#[derive(Debug)]
pub struct NamedVersionStringStrategy {
  pub name: String,
  pub path: String,
}

#[derive(Debug)]
pub struct UnnamedVersionStringStrategy {
  pub name: String,
  pub path: String,
}

#[derive(Debug)]
pub struct VersionsByNameStrategy {
  pub name: String,
  pub path: String,
}

#[derive(Debug)]
pub enum Strategy {
  NameAndVersionProps(NameAndVersionPropsStrategy),
  NamedVersionString(NamedVersionStringStrategy),
  UnnamedVersionString(UnnamedVersionStringStrategy),
  VersionsByName(VersionsByNameStrategy),
}

impl Strategy {
  pub fn from_rcfile(rcfile: &config::Rcfile) -> HashMap<String, Strategy> {
    let dependency_types = &rcfile.dependency_types;
    let custom_types = &rcfile.custom_types;
    let default_types = get_default_types();
    let mut strategies: HashMap<String, Strategy> = HashMap::new();
    let len = dependency_types.len();
    let include_all = len == 0 || len == 1 && dependency_types[0] == "**";
    let contains_explicitly_excluded = dependency_types
      .iter()
      .any(|dep_type| dep_type.starts_with('!'));

    let is_included = |dep_type: &String| -> bool {
      // All are included by default
      if include_all {
        return true;
      }
      // Is explicitly included
      if dependency_types.contains(dep_type) {
        return true;
      }
      // Is explicitly excluded
      if dependency_types.contains(&get_negated(dep_type)) {
        return false;
      }
      // Is implicitly included when another type is explicitly excluded and
      // this one is not named
      if contains_explicitly_excluded {
        return true;
      }
      false
    };

    default_types.iter().for_each(|(key, value)| {
      if is_included(key) {
        strategies.insert(key.clone(), from_config(key, value));
      }
    });

    custom_types.iter().for_each(|(key, value)| {
      if is_included(key) {
        strategies.insert(key.clone(), from_config(key, value));
      }
    });

    strategies
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyStrategy {
  pub strategy: String,
  pub name_path: Option<String>,
  pub path: String,
}

fn from_config(name: &String, config: &AnyStrategy) -> Strategy {
  match config.strategy.as_str() {
    "name~version" => Strategy::NameAndVersionProps(NameAndVersionPropsStrategy {
      name: name.clone(),
      path: normalize_path(config.path.clone()),
      name_path: config
        .name_path
        .clone()
        .expect("A name~version strategy must have a namePath"),
    }),
    "name@version" => Strategy::NamedVersionString(NamedVersionStringStrategy {
      name: name.clone(),
      path: normalize_path(config.path.clone()),
    }),
    "version" => Strategy::UnnamedVersionString(UnnamedVersionStringStrategy {
      name: name.clone(),
      path: normalize_path(config.path.clone()),
    }),
    "versionsByName" => Strategy::VersionsByName(VersionsByNameStrategy {
      name: name.clone(),
      path: normalize_path(config.path.clone()),
    }),
    _ => panic!("Unknown strategy: {}", config.strategy),
  }
}

fn get_default_types() -> HashMap<String, strategy::AnyStrategy> {
  serde_json::from_str(
    r#"
    {
      "dev": {
        "strategy": "versionsByName",
        "path": "devDependencies"
      },
      "local": {
        "strategy": "name~version",
        "namePath": "name",
        "path": "version"
      },
      "overrides": {
        "strategy": "versionsByName",
        "path": "overrides"
      },
      "peer": {
        "strategy": "versionsByName",
        "path": "peerDependencies"
      },
      "pnpmOverrides": {
        "strategy": "versionsByName",
        "path": "pnpm.overrides"
      },
      "prod": {
        "strategy": "versionsByName",
        "path": "dependencies"
      },
      "resolutions": {
        "strategy": "versionsByName",
        "path": "resolutions"
      }
    }
    "#,
  )
  .unwrap()
}

/// Adds a forward slash to the start of the String and replaces every "."
/// inside the String with a "/"
fn normalize_path(path: String) -> String {
  let mut normalized_path = String::from("/");
  normalized_path.push_str(&path.replace(".", "/"));
  normalized_path
}

/// Adds "!" to the start of the String
fn get_negated(path: &String) -> String {
  let mut negated_path = String::from("!");
  negated_path.push_str(&path);
  negated_path
}
