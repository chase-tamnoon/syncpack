use serde::Deserialize;
use std::collections::HashMap;
use std::path;

use crate::file_paths;
use crate::semver_group;
use crate::strategy;
use crate::version_group;

fn empty_custom_types() -> HashMap<String, strategy::AnyStrategy> {
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
  vec![
    "package.json".to_string(),
    "packages/*/package.json".to_string(),
  ]
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rcfile {
  #[serde(default = "empty_custom_types")]
  pub custom_types: HashMap<String, strategy::AnyStrategy>,
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
  pub fn get_sources(&self, cwd: &path::PathBuf) -> Vec<path::PathBuf> {
    let pattern = &cwd.join("fixtures/**/package.json");
    let pattern_str = pattern.to_str().unwrap();
    file_paths::get_file_paths(pattern_str)
  }
}

pub fn get() -> Rcfile {
  let raw_json = r#"
    {
      "customTypes": {
        "engines": {
          "strategy": "versionsByName",
          "path": "engines"
        }
      },
      "dependencyTypes": ["!peer", "!prod"],
      "filter": ".",
      "formatBugs": true,
      "formatRepository": true,
      "indent": "  ",
      "semverGroups": [
        {
          "dependencyTypes": ["overrides"],
          "isIgnored": true
        },
        {
          "range": ""
        }
      ],
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
      "versionGroups": [
        { "dependencies": ["string-width"], "pinVersion": "<5.0.0" },
        { "dependencies": ["strip-ansi"], "pinVersion": "<7.0.0" },
        { "dependencies": ["wrap-ansi"], "pinVersion": "<8.0.0" },
        { "dependencies": ["chalk"], "pinVersion": "4.1.2" },
        { "dependencies": ["globby"], "pinVersion": "11.1.0" },
        { "dependencies": ["ora"], "pinVersion": "5.4.1" }
      ]
    }
  "#;

  let deserialized: Rcfile = serde_json::from_str(raw_json).unwrap();
  deserialized
}
