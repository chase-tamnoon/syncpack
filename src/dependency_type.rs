use crate::config;
use crate::instance;
use crate::package_json;

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
  pub name_path: String,
  /// The dependency type name this strategy is referred to as
  pub name: String,
  /// The path to the property that contains the version string
  pub path: String,
  /// The strategy to use when reading/writing the version string
  pub strategy: Strategy,
}

impl DependencyType {
  pub fn read(&self, file: &package_json::PackageJson) -> Vec<instance::Instance> {
    match &self.strategy {
      Strategy::NameAndVersionProps => {
        let name = file.get_prop(&self.name_path);
        let version = file.get_prop(&self.path);
        if let Some(name) = name {
          if let Some(version) = version {
            if let serde_json::Value::String(name) = name {
              if let serde_json::Value::String(version) = version {
                let instance =
                  instance::Instance::new(name.to_string(), version.to_string(), self.clone());
                return vec![instance];
              }
              return vec![];
            }
            return vec![];
          }
          return vec![];
        }
        return vec![];
      }
      Strategy::NamedVersionString => {
        let specifier = file.get_prop(&self.path);
        println!("specifier: {:?}", specifier);
        if let Some(specifier) = specifier {
          if let serde_json::Value::String(specifier) = specifier {
            let parts: Vec<&str> = specifier.split("@").collect();
            let name = parts[0].to_string();
            let version = parts[1].to_string();
            let instance = instance::Instance::new(name, version, self.clone());
            return vec![instance];
          }
          return vec![];
        }
        return vec![];
      }
      Strategy::UnnamedVersionString => {
        let version = file.get_prop(&self.path);
        if let Some(version) = version {
          if let serde_json::Value::String(version) = version {
            let instance =
              instance::Instance::new(self.name.clone(), version.to_string(), self.clone());
            return vec![instance];
          }
          return vec![];
        }
        return vec![];
      }
      Strategy::VersionsByName => {
        let versions_by_name = file.get_prop(&self.path);
        if let Some(versions_by_name) = versions_by_name {
          if let serde_json::Value::Object(versions_by_name) = versions_by_name {
            let mut instances: Vec<instance::Instance> = vec![];
            for (name, version) in versions_by_name {
              if let serde_json::Value::String(version) = version {
                let instance =
                  instance::Instance::new(name.to_string(), version.to_string(), self.clone());
                instances.push(instance);
              }
            }
            return instances;
          }
          return vec![];
        }
        return vec![];
      }
      _ => vec![],
    }
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
      strategy: Strategy::new(config.strategy.as_str()),
    }
  }
}

/// Converts a "some.nested.prop.name" selector to "/some/nested/prop/name"
fn normalize_path(path: String) -> String {
  let mut normalized_path = String::from("/");
  normalized_path.push_str(&path.replace(".", "/"));
  normalized_path
}
