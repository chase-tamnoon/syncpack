use crate::config;
use crate::instance;
use crate::package_json;

#[derive(Clone, Debug)]
pub struct Strategy {
  pub name_path: Option<String>,
  pub name: String,
  pub path: String,
  pub strategy: String,
}

impl Strategy {
  pub fn read(&self, file: &package_json::PackageJson) -> Vec<instance::Instance> {
    vec![instance::Instance::new(
      "foo".to_string(),
      "0.0.0".to_string(),
      self.clone(),
    )]
  }

  pub fn write(&self, file: &package_json::PackageJson) {
    println!("Writing NameAndVersionPropsStrategy...");
  }

  pub fn new(name: &String, config: &config::AnyStrategy) -> Strategy {
    Strategy {
      name_path: config.name_path.clone(),
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
