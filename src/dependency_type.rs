use std::vec;

use serde_json::Value;

use crate::config;
use crate::instance::Instance;
use crate::package_json;

#[derive(Clone, Debug)]
pub enum StrategyType {
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

impl StrategyType {
  pub fn new(strategy: &str) -> StrategyType {
    match strategy {
      "name~version" => StrategyType::NameAndVersionProps,
      "name@version" => StrategyType::NamedVersionString,
      "version" => StrategyType::UnnamedVersionString,
      "versionsByName" => StrategyType::VersionsByName,
      _ => StrategyType::InvalidConfig,
    }
  }
}

impl StrategyType {
  fn read(
    &self,
    dependency_type: &DependencyType,
    file: &package_json::PackageJson,
  ) -> Vec<Instance> {
    match *self {
      StrategyType::NameAndVersionProps => {
        if let (Some(Value::String(name)), Some(Value::String(version))) = (
          file.get_prop(&dependency_type.name_path),
          file.get_prop(&dependency_type.path),
        ) {
          let instance = Instance::new(
            name.to_string(),
            version.to_string(),
            dependency_type.clone(),
          );
          return vec![instance];
        }
        vec![]
      }
      StrategyType::NamedVersionString => {
        if let Some(Value::String(specifier)) = file.get_prop(&dependency_type.path) {
          if let Some((name, version)) = specifier.split_once('@') {
            let instance = Instance::new(
              name.to_string(),
              version.to_string(),
              dependency_type.clone(),
            );
            return vec![instance];
          }
        }
        vec![]
      }
      StrategyType::UnnamedVersionString => {
        if let Some(Value::String(version)) = file.get_prop(&dependency_type.path) {
          let instance = Instance::new(
            dependency_type.name.clone(),
            version.to_string(),
            dependency_type.clone(),
          );
          return vec![instance];
        }
        vec![]
      }
      StrategyType::VersionsByName => {
        if let Some(Value::Object(versions_by_name)) = file.get_prop(&dependency_type.path) {
          let instances = versions_by_name
            .iter()
            .filter_map(|(name, version)| {
              if let Value::String(version) = version {
                Some(Instance::new(
                  dependency_type.name.to_string(),
                  version.to_string(),
                  dependency_type.clone(),
                ))
              } else {
                None
              }
            })
            .collect::<Vec<Instance>>();
          return instances;
        }
        vec![]
      }
      _ => vec![],
    }
  }
}

#[derive(Clone, Debug)]
pub struct DependencyType {
  /// The path to the property that contains the dependency name
  pub name_path: String,
  /// The dependency type name this strategy is referred to as
  pub name: String,
  /// The path to the property that contains the version string
  pub path: String,
  /// The strategy to use when reading/writing the version string
  pub strategy_type: StrategyType,
}

impl DependencyType {
  /// Get all instances of this dependency type from the given package.json
  pub fn read(&self, file: &package_json::PackageJson) -> Vec<Instance> {
    self.strategy_type.read(&self, &file)
  }

  pub fn write(&self, file: &package_json::PackageJson) {
    println!("Writing NameAndVersionPropsStrategy...");
  }

  pub fn new(name: &String, config: &config::CustomType) -> DependencyType {
    DependencyType {
      name_path: if config.name_path.is_some() {
        normalize_path(config.name_path.clone().unwrap())
      } else {
        String::from("")
      },
      name: name.clone(),
      path: normalize_path(config.path.clone()),
      strategy_type: StrategyType::new(config.strategy.as_str()),
    }
  }
}

/// Converts a "some.nested.prop.name" selector to "/some/nested/prop/name"
fn normalize_path(path: String) -> String {
  let mut normalized_path = String::from("/");
  normalized_path.push_str(&path.replace(".", "/"));
  normalized_path
}
