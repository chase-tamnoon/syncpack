use std::vec;

use serde_json::Value;

use crate::config;
use crate::instance::Instance;
use crate::package_json;

#[derive(Debug)]
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

impl Strategy {
  fn get_instances<'a>(
    &'a self,
    dependency_type: &'a DependencyType,
    file: &package_json::PackageJson,
  ) -> Vec<Instance> {
    match *self {
      Strategy::NameAndVersionProps => {
        if let (Some(Value::String(name)), Some(Value::String(version))) = (
          file.get_prop(&dependency_type.name_path),
          file.get_prop(&dependency_type.path),
        ) {
          let instance = Instance::new(name.to_string(), version.to_string(), &dependency_type);
          return vec![instance];
        }
        vec![]
      }
      Strategy::NamedVersionString => {
        if let Some(Value::String(specifier)) = file.get_prop(&dependency_type.path) {
          if let Some((name, version)) = specifier.split_once('@') {
            let instance = Instance::new(name.to_string(), version.to_string(), &dependency_type);
            return vec![instance];
          }
        }
        vec![]
      }
      Strategy::UnnamedVersionString => {
        if let Some(Value::String(version)) = file.get_prop(&dependency_type.path) {
          let instance = Instance::new(
            dependency_type.name.clone(),
            version.to_string(),
            &dependency_type,
          );
          return vec![instance];
        }
        vec![]
      }
      Strategy::VersionsByName => {
        if let Some(Value::Object(versions_by_name)) = file.get_prop(&dependency_type.path) {
          let mut instances: Vec<Instance> = vec![];
          for (name, version) in versions_by_name {
            if let Value::String(version) = version {
              let instance = Instance::new(name.to_string(), version.to_string(), &dependency_type);
              instances.push(instance);
            }
          }
          return instances;
        }
        vec![]
      }
      _ => vec![],
    }
  }
}

#[derive(Debug)]
pub struct DependencyType {
  /// The path to the property that contains the dependency name
  pub name_path: String,
  /// The dependency type name this strategy is referred to as
  pub name: String,
  /// The path to the property that contains the version string
  pub path: String,
  /// The strategy to use when reading/writing the version string
  pub strategy: Strategy,
}

impl DependencyType {
  /// Get all instances of this dependency type from the given package.json
  pub fn get_instances(&self, file: &package_json::PackageJson) -> Vec<Instance> {
    self.strategy.get_instances(&self, &file)
  }

  pub fn write(&self, file: &package_json::PackageJson) {
    println!("Writing NameAndVersionPropsStrategy...");
  }

  pub fn new(name: &String, config: &config::CustomType) -> DependencyType {
    DependencyType {
      name_path: if let Some(name_path) = &config.name_path {
        normalize_path(&name_path)
      } else {
        String::from("")
      },
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
