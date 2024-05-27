use colored::*;
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fs, path};

use crate::{dependency_type, semver_group, version_group};

fn empty_custom_types() -> HashMap<String, CustomType> {
  HashMap::new()
}

fn default_true() -> bool {
  true
}

fn default_filter() -> String {
  ".".to_string()
}

fn default_indent() -> String {
  "  ".to_string()
}

fn default_sort_az() -> Vec<String> {
  vec![
    "bin".to_string(),
    "contributors".to_string(),
    "dependencies".to_string(),
    "devDependencies".to_string(),
    "keywords".to_string(),
    "peerDependencies".to_string(),
    "resolutions".to_string(),
    "scripts".to_string(),
  ]
}

fn default_sort_exports() -> Vec<String> {
  vec![
    "types".to_string(),
    "node-addons".to_string(),
    "node".to_string(),
    "browser".to_string(),
    "module".to_string(),
    "import".to_string(),
    "require".to_string(),
    "development".to_string(),
    "production".to_string(),
    "script".to_string(),
    "default".to_string(),
  ]
}

fn sort_first() -> Vec<String> {
  vec![
    "name".to_string(),
    "description".to_string(),
    "version".to_string(),
    "author".to_string(),
  ]
}

fn default_source() -> Vec<String> {
  vec![]
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomType {
  pub strategy: String,
  pub name_path: Option<String>,
  pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rcfile {
  #[serde(default = "empty_custom_types")]
  pub custom_types: HashMap<String, CustomType>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default = "default_filter")]
  pub filter: String,
  #[serde(default = "default_true")]
  pub format_bugs: bool,
  #[serde(default = "default_true")]
  pub format_repository: bool,
  #[serde(default = "default_indent")]
  pub indent: String,
  #[serde(default)]
  pub semver_groups: Vec<semver_group::AnySemverGroup>,
  #[serde(default = "default_sort_az")]
  pub sort_az: Vec<String>,
  #[serde(default = "default_sort_exports")]
  pub sort_exports: Vec<String>,
  #[serde(default = "sort_first")]
  pub sort_first: Vec<String>,
  #[serde(default = "default_true")]
  pub sort_packages: bool,
  #[serde(default = "default_source")]
  pub source: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  #[serde(default)]
  pub version_groups: Vec<version_group::AnyVersionGroup>,
}

impl Rcfile {
  pub fn get_filter(&self) -> Regex {
    Regex::new(&self.filter).expect("filter config value is not a valid Regex string")
  }

  pub fn get_enabled_dependency_types(&self) -> Vec<dependency_type::DependencyType> {
    // Dependency type names referenced in the rcfile
    let named_types = &self.dependency_types;
    // Custom dependency types defined in the rcfile
    let custom_types = &self.custom_types;
    // Internal dependency types are also defined as custom types
    let default_types = get_default_dependency_types();
    // Collect which dependency types are enabled
    let mut dependency_types: Vec<dependency_type::DependencyType> = vec![];
    // When no dependency types are referenced in the rcfile, all are enabled
    let len = named_types.len();
    let include_all = len == 0 || len == 1 && named_types[0] == "**";
    // When any dependency types are explicitly disabled, all others are enabled
    let contains_explicitly_disabled = named_types
      .iter()
      .any(|named_type| named_type.starts_with('!'));

    let is_enabled = |type_name: &String| -> bool {
      // All are enabled by default
      if include_all {
        return true;
      }
      // Is explicitly enabled
      if named_types.contains(type_name) {
        return true;
      }
      // Is explicitly disabled
      if named_types.contains(&get_negated(type_name)) {
        return false;
      }
      // Is implicitly enabled when another type is explicitly disabled and
      // this one is not named
      if contains_explicitly_disabled {
        return true;
      }
      false
    };

    default_types.iter().for_each(|(name, custom_type)| {
      if is_enabled(name) {
        dependency_types.push(dependency_type::DependencyType::new(name, custom_type));
      }
    });

    custom_types.iter().for_each(|(name, custom_type)| {
      if is_enabled(name) {
        dependency_types.push(dependency_type::DependencyType::new(name, custom_type));
      }
    });

    dependency_types
  }
}

/// Adds "!" to the start of the String
fn get_negated(str: &String) -> String {
  let mut negated_str = String::from("!");
  negated_str.push_str(&str);
  negated_str
}

fn get_default_dependency_types() -> HashMap<String, CustomType> {
  HashMap::from([
    (
      String::from("dev"),
      CustomType {
        strategy: String::from("versionsByName"),
        name_path: None,
        path: String::from("devDependencies"),
      },
    ),
    (
      String::from("local"),
      CustomType {
        strategy: String::from("name~version"),
        name_path: Some(String::from("name")),
        path: String::from("version"),
      },
    ),
    (
      String::from("overrides"),
      CustomType {
        strategy: String::from("versionsByName"),
        name_path: None,
        path: String::from("overrides"),
      },
    ),
    (
      String::from("peer"),
      CustomType {
        strategy: String::from("versionsByName"),
        name_path: None,
        path: String::from("peerDependencies"),
      },
    ),
    (
      String::from("pnpmOverrides"),
      CustomType {
        strategy: String::from("versionsByName"),
        name_path: None,
        path: String::from("pnpm.overrides"),
      },
    ),
    (
      String::from("prod"),
      CustomType {
        strategy: String::from("versionsByName"),
        name_path: None,
        path: String::from("dependencies"),
      },
    ),
    (
      String::from("resolutions"),
      CustomType {
        strategy: String::from("versionsByName"),
        name_path: None,
        path: String::from("resolutions"),
      },
    ),
  ])
}

/// Try to read the rcfile from the current working directory and fall back to
/// defaults if one is not found
pub fn get(cwd: &path::PathBuf) -> Rcfile {
  let short_path = ".syncpackrc.json";
  let file_path = cwd.join(short_path);
  fs::read_to_string(&file_path)
    .inspect_err(|_| {
      println!(
        "{}",
        format!(
          "? using default config: {} not found",
          &file_path.to_str().unwrap()
        )
        .dimmed()
      );
    })
    .or_else(|_| Ok("{}".to_string()))
    .and_then(|json| serde_json::from_str::<Rcfile>(&json))
    .unwrap()
}
