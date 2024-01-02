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
      .map(|custom_type| Strategy::from_config(custom_type))
      .collect()
  }

  pub fn from_config(name: &String, config: &AnyStrategy) -> Strategy {
    match &config.strategy {
      Some(strategy) => match strategy.as_str() {
        "name~version" => Strategy::NameAndVersionProps(NameAndVersionPropsStrategy {
          name: name.clone(),
          path: config.path.clone(),
          name_path: config.name_path.clone(),
        }),
        "name@version" => Strategy::NamedVersionString(NamedVersionStringStrategy {
          name: name.clone(),
          path: config.path.clone(),
        }),
        "version" => Strategy::UnnamedVersionString(UnnamedVersionStringStrategy {
          name: name.clone(),
          path: config.path.clone(),
        }),
        "versionsByName" => Strategy::VersionsByName(VersionsByNameStrategy {
          name: name.clone(),
          path: config.path.clone(),
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
