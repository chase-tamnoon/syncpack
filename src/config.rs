use serde::{Deserialize, Serialize};
use std::path;

use crate::file_paths;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rcfile {
  pub custom_types: CustomTypes,
  pub dependency_types: Vec<String>,
  pub filter: String,
  pub format_bugs: bool,
  pub format_repository: bool,
  pub indent: String,
  pub semver_groups: Vec<String>,
  pub sort_az: Vec<String>,
  pub sort_exports: Vec<String>,
  pub sort_first: Vec<String>,
  pub sort_packages: bool,
  pub source: Vec<String>,
  pub specifier_types: Vec<String>,
  pub version_groups: Vec<String>,
}

impl Rcfile {
  pub fn get_sources(&self, cwd: path::PathBuf) -> Vec<path::PathBuf> {
    let pattern = cwd.join("fixtures/**/package.json");
    let pattern_str = pattern.to_str().unwrap();
    file_paths::get_file_paths(pattern_str)
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTypes {
  pub dev: Strategy,
  pub local: LocalStrategy,
  pub overrides: Strategy,
  pub peer: Strategy,
  pub pnpm_overrides: Strategy,
  pub prod: Strategy,
  pub resolutions: Strategy,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Strategy {
  pub strategy: String,
  pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStrategy {
  pub strategy: String,
  pub name_path: String,
  pub path: String,
}

pub fn get() -> Rcfile {
  let raw_json = r#"
    {
      "customTypes": {
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
      },
      "dependencyTypes": ["**"],
      "filter": ".",
      "formatBugs": true,
      "formatRepository": true,
      "indent": "  ",
      "semverGroups": [],
      "sortAz": [
        "bin",
        "contributors",
        "dependencies",
        "devDependencies",
        "keywords",
        "peerDependencies",
        "resolutions",
        "scripts"
      ],
      "sortExports": [
        "types",
        "node-addons",
        "node",
        "browser",
        "module",
        "import",
        "require",
        "development",
        "production",
        "script",
        "default"
      ],
      "sortFirst": ["name", "description", "version", "author"],
      "sortPackages": true,
      "source": ["package.json", "packages/*/package.json"],
      "specifierTypes": ["**"],
      "versionGroups": []
    }
  "#;

  let deserialized: Rcfile = serde_json::from_str(raw_json).unwrap();
  deserialized
}

// pub fn get_sort_first() -> Vec<String> {
//   let rcfile = get();
//   rcfile.sort_first
//   convert_to_vec_string(rcfile.pointer("sortFirst").unwrap().clone()).unwrap()
// }

// fn convert_to_vec_string(value: Value) -> Option<Vec<String>> {
//   match value {
//     Value::Array(arr) => {
//       let string_vec: Vec<String> = arr
//         .into_iter()
//         .filter_map(|item| item.as_str().map(|s| s.to_string()))
//         .collect();
//       Some(string_vec)
//     }
//     _ => None,
//   }
// }
