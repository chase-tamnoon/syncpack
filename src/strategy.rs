use std::collections::HashMap;

use serde::Deserialize;

use crate::config;

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
    let mut strategies_by_name: HashMap<String, Strategy> = get_strategies_by_name();

    rcfile.custom_types.iter().for_each(|(name, config)| {
      let strategy = Strategy::from_config(name, config);
      let name = match &strategy {
        Strategy::NameAndVersionProps(s) => &s.name,
        Strategy::NamedVersionString(s) => &s.name,
        Strategy::UnnamedVersionString(s) => &s.name,
        Strategy::VersionsByName(s) => &s.name,
      };
      strategies_by_name.insert(name.clone(), strategy);
    });
    strategies_by_name
  }

  pub fn from_config(name: &String, config: &AnyStrategy) -> Strategy {
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyStrategy {
  pub strategy: String,
  pub name_path: Option<String>,
  pub path: String,
}

pub fn get_strategies_by_name() -> HashMap<String, Strategy> {
  let mut strategies_by_name: HashMap<String, Strategy> = HashMap::new();
  strategies_by_name.insert(
    String::from("local"),
    Strategy::NameAndVersionProps(NameAndVersionPropsStrategy {
      name: String::from("local"),
      path: String::from("/version"),
      name_path: String::from("name"),
    }),
  );
  strategies_by_name.insert(
    String::from("dev"),
    Strategy::VersionsByName(VersionsByNameStrategy {
      name: String::from("dev"),
      path: String::from("/devDependencies"),
    }),
  );
  strategies_by_name.insert(
    String::from("overrides"),
    Strategy::VersionsByName(VersionsByNameStrategy {
      name: String::from("overrides"),
      path: String::from("/overrides"),
    }),
  );
  strategies_by_name.insert(
    String::from("peer"),
    Strategy::VersionsByName(VersionsByNameStrategy {
      name: String::from("peer"),
      path: String::from("/peerDependencies"),
    }),
  );
  strategies_by_name.insert(
    String::from("pnpmOverrides"),
    Strategy::VersionsByName(VersionsByNameStrategy {
      name: String::from("pnpmOverrides"),
      path: String::from("/pnpm/overrides"),
    }),
  );
  strategies_by_name.insert(
    String::from("prod"),
    Strategy::VersionsByName(VersionsByNameStrategy {
      name: String::from("prod"),
      path: String::from("/dependencies"),
    }),
  );
  strategies_by_name.insert(
    String::from("resolutions"),
    Strategy::VersionsByName(VersionsByNameStrategy {
      name: String::from("resolutions"),
      path: String::from("/resolutions"),
    }),
  );
  strategies_by_name
}

/// Adds a forward slash to the start of the String and replaces every "."
/// inside the String with a "/"
fn normalize_path(path: String) -> String {
  let mut normalized_path = String::from("/");
  normalized_path.push_str(&path.replace(".", "/"));
  normalized_path
}
