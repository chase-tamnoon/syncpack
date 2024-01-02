use serde::Deserialize;

use crate::config;

#[derive(Debug)]
pub struct NameAndVersionPropsStrategy {
  name: String,
  path: String,
  name_path: String,
}

#[derive(Debug)]
pub struct NamedVersionStringStrategy {
  name: String,
  path: String,
}

#[derive(Debug)]
pub struct UnnamedVersionStringStrategy {
  name: String,
  path: String,
}

#[derive(Debug)]
pub struct VersionsByNameStrategy {
  name: String,
  path: String,
}

#[derive(Debug)]
pub enum Strategy {
  NameAndVersionProps(NameAndVersionPropsStrategy),
  NamedVersionString(NamedVersionStringStrategy),
  UnnamedVersionString(UnnamedVersionStringStrategy),
  VersionsByName(VersionsByNameStrategy),
}

impl Strategy {
  pub fn from_rcfile(rcfile: &config::Rcfile) -> Vec<Strategy> {
    rcfile
      .custom_types
      .iter()
      .map(|(name, config)| Strategy::from_config(name, config))
      .collect()
  }

  pub fn from_config(name: &String, config: &AnyStrategy) -> Strategy {
    match &config.strategy {
      Some(strategy) => match strategy.as_str() {
        "name~version" => Strategy::NameAndVersionProps(NameAndVersionPropsStrategy {
          name: name.clone(),
          path: config.path.clone().unwrap(),
          name_path: config.name_path.clone().unwrap(),
        }),
        "name@version" => Strategy::NamedVersionString(NamedVersionStringStrategy {
          name: name.clone(),
          path: config.path.clone().unwrap(),
        }),
        "version" => Strategy::UnnamedVersionString(UnnamedVersionStringStrategy {
          name: name.clone(),
          path: config.path.clone().unwrap(),
        }),
        "versionsByName" => Strategy::VersionsByName(VersionsByNameStrategy {
          name: name.clone(),
          path: config.path.clone().unwrap(),
        }),
        _ => panic!("Unknown strategy: {}", strategy),
      },
      None => panic!("Strategy not provided"),
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyStrategy {
  pub strategy: Option<String>,
  pub name_path: Option<String>,
  pub path: Option<String>,
}
