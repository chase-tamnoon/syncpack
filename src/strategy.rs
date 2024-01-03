use crate::config;
use crate::instance;
use crate::package_json;

#[derive(Clone, Debug)]
pub struct Strategy {
  pub name_path: String,
  pub name: String,
  pub path: String,
  pub strategy: String,
}

impl Strategy {
  pub fn read(&self, file: &package_json::PackageJson) -> Vec<instance::Instance> {
    if self.strategy == "versionsByName" {
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
      }
    } else if self.strategy == "name~version" {
      let name = file.get_prop(&self.name_path);
      let version = file.get_prop(&self.path);
      println!("name: {:?}", name);
      println!("version: {:?}", version);
      if let Some(name) = name {
        if let Some(version) = version {
          if let serde_json::Value::String(name) = name {
            if let serde_json::Value::String(version) = version {
              let instance =
                instance::Instance::new(name.to_string(), version.to_string(), self.clone());
              return vec![instance];
            }
          }
        }
      }
    }
    vec![]
  }

  pub fn write(&self, file: &package_json::PackageJson) {
    println!("Writing NameAndVersionPropsStrategy...");
  }

  pub fn new(name: &String, config: &config::AnyStrategy) -> Strategy {
    Strategy {
      name_path: if config.name_path.is_some() {
        normalize_path(config.name_path.clone().unwrap())
      } else {
        String::from("")
      },
      name: name.clone(),
      path: normalize_path(config.path.clone()),
      strategy: config.strategy.clone(),
    }
  }
}

/// Converts a "some.nested.prop.name" selector to "/some/nested/prop/name"
fn normalize_path(path: String) -> String {
  let mut normalized_path = String::from("/");
  normalized_path.push_str(&path.replace(".", "/"));
  normalized_path
}
