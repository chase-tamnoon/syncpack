use crate::config;

#[derive(Clone, Debug)]
pub enum Strategy {
  /// "name~version"
  NameAndVersionProps,
  /// "name@version"
  NamedVersionString,
  /// "version"
  UnnamedVersionString,
  /// "versionsByName"
  VersionsByName,
  /// Not recognised
  InvalidConfig,
}

impl Strategy {
  pub fn new(strategy: &str) -> Strategy {
    match strategy {
      "name~version" => Strategy::NameAndVersionProps,
      "name@version" => Strategy::NamedVersionString,
      "version" => Strategy::UnnamedVersionString,
      "versionsByName" => Strategy::VersionsByName,
      _ => Strategy::InvalidConfig,
    }
  }
}

#[derive(Clone, Debug)]
pub struct DependencyType {
  /// The path to the property that contains the dependency name
  pub name_path: Option<String>,
  /// The dependency type name this strategy is referred to as
  pub name: String,
  /// The path to the property that contains the version string
  pub path: String,
  /// The strategy to use when reading/writing the version string
  pub strategy: Strategy,
}

impl DependencyType {
  pub fn new(name: &String, config: &config::CustomType) -> DependencyType {
    DependencyType {
      name_path: config.name_path.as_ref().map(|name_path| normalize_path(&name_path)),
      name: name.clone(),
      path: normalize_path(&config.path),
      strategy: Strategy::new(config.strategy.as_str()),
    }
  }
}

/// Converts a "some.nested.prop.name" selector to "/some/nested/prop/name"
fn normalize_path(path: &String) -> String {
  let mut normalized_path = String::from("/");
  normalized_path.push_str(&path.replace(".", "/"));
  normalized_path
}
