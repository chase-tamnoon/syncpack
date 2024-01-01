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
  pub semver_groups: Vec<SemverGroup>,
  pub sort_az: Vec<String>,
  pub sort_exports: Vec<String>,
  pub sort_first: Vec<String>,
  pub sort_packages: bool,
  pub source: Vec<String>,
  pub specifier_types: Vec<String>,
  pub version_groups: Vec<VersionGroup>,
}

impl Rcfile {
  pub fn get_sources(&self, cwd: &path::PathBuf) -> Vec<path::PathBuf> {
    let pattern = &cwd.join("fixtures/**/package.json");
    let pattern_str = pattern.to_str().unwrap();
    file_paths::get_file_paths(pattern_str)
  }

  pub fn pretty_print(&self) -> () {
    println!("{:#?}", &self);
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisabledSemverGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  pub is_disabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredSemverGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  pub is_ignored: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithRangeSemverGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  pub range: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum SemverGroup {
  Disabled(DisabledSemverGroup),
  Ignored(IgnoredSemverGroup),
  WithRange(WithRangeSemverGroup),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BannedVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  is_banned: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  is_ignored: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  pin_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SameRangeVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  policy: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnappedToVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  snap_to: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  prefer_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum VersionGroup {
  Banned(BannedVersionGroup),
  Ignored(IgnoredVersionGroup),
  Pinned(PinnedVersionGroup),
  SameRange(SameRangeVersionGroup),
  SnappedTo(SnappedToVersionGroup),
  Standard(StandardVersionGroup),
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

      ]
    }
  "#;

  let deserialized: Rcfile = serde_json::from_str(raw_json).unwrap();
  deserialized
}

// { "dependencies": ["string-width"], "pinVersion": "<5.0.0" },
// { "dependencies": ["strip-ansi"], "pinVersion": "<7.0.0" },
// { "dependencies": ["wrap-ansi"], "pinVersion": "<8.0.0" },
// { "dependencies": ["chalk"], "pinVersion": "4.1.2" },
// { "dependencies": ["globby"], "pinVersion": "11.1.0" },
// { "dependencies": ["ora"], "pinVersion": "5.4.1" }

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
