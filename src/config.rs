use serde::Deserialize;
use std::path;

use crate::file_paths;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rcfile {
  pub custom_types: CustomTypes,
  pub dependency_types: Vec<String>,
  pub filter: String,
  pub format_bugs: bool,
  pub format_repository: bool,
  pub indent: String,
  pub semver_groups: Vec<AnySemverGroup>,
  pub sort_az: Vec<String>,
  pub sort_exports: Vec<String>,
  pub sort_first: Vec<String>,
  pub sort_packages: bool,
  pub source: Vec<String>,
  pub specifier_types: Vec<String>,
  pub version_groups: Vec<AnyVersionGroup>,
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Strategy {
  pub strategy: String,
  pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStrategy {
  pub strategy: String,
  pub name_path: String,
  pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnySemverGroup {
  pub dependencies: Option<Vec<String>>,
  pub dependency_types: Option<Vec<String>>,
  pub label: Option<String>,
  pub packages: Option<Vec<String>>,
  pub specifier_types: Option<Vec<String>>,
  //
  pub is_disabled: Option<bool>,
  pub is_ignored: Option<bool>,
  pub range: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyVersionGroup {
  pub dependencies: Option<Vec<String>>,
  pub dependency_types: Option<Vec<String>>,
  pub label: Option<String>,
  pub packages: Option<Vec<String>>,
  pub specifier_types: Option<Vec<String>>,
  //
  pub is_banned: Option<bool>,
  pub is_ignored: Option<bool>,
  pub pin_version: Option<String>,
  pub policy: Option<String>,
  pub snap_to: Option<Vec<String>>,
  pub prefer_version: Option<String>,
}

#[derive(Debug)]
pub struct GroupSelector {
  pub dependencies: Vec<String>,
  pub dependency_types: Vec<String>,
  pub label: String,
  pub packages: Vec<String>,
  pub specifier_types: Vec<String>,
}

// =============================================================================

#[derive(Debug)]
pub struct DisabledSemverGroup {
  pub selector: GroupSelector,
  pub is_disabled: bool,
}

#[derive(Debug)]
pub struct IgnoredSemverGroup {
  pub selector: GroupSelector,
  pub is_ignored: bool,
}

#[derive(Debug)]
pub struct WithRangeSemverGroup {
  pub selector: GroupSelector,
  pub range: String,
}

pub enum SemverGroup {
  Disabled(DisabledSemverGroup),
  Ignored(IgnoredSemverGroup),
  WithRange(WithRangeSemverGroup),
}

pub struct BannedVersionGroup {
  pub selector: GroupSelector,
  pub is_banned: bool,
}

pub struct IgnoredVersionGroup {
  pub selector: GroupSelector,
  pub is_ignored: bool,
}

pub struct PinnedVersionGroup {
  pub selector: GroupSelector,
  pub pin_version: String,
}

pub struct SameRangeVersionGroup {
  pub selector: GroupSelector,
  pub policy: String,
}

pub struct SnappedToVersionGroup {
  pub selector: GroupSelector,
  pub snap_to: Vec<String>,
}

pub struct StandardVersionGroup {
  pub selector: GroupSelector,
  pub prefer_version: String,
}

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
