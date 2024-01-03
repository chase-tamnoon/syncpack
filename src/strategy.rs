use crate::config;

pub trait StrategyTrait {
  fn read(&self);
  fn write(&self);
}

#[derive(Debug)]
pub struct NameAndVersionPropsStrategy {
  pub name: String,
  pub path: String,
  pub name_path: String,
}

impl StrategyTrait for NameAndVersionPropsStrategy {
  fn read(&self) {
    println!("Reading NameAndVersionPropsStrategy...");
  }

  fn write(&self) {
    println!("Writing NameAndVersionPropsStrategy...");
  }
}

#[derive(Debug)]
pub struct NamedVersionStringStrategy {
  pub name: String,
  pub path: String,
}

impl StrategyTrait for NamedVersionStringStrategy {
  fn read(&self) {
    println!("Reading NamedVersionStringStrategy...");
  }

  fn write(&self) {
    println!("Writing NamedVersionStringStrategy...");
  }
}

#[derive(Debug)]
pub struct UnnamedVersionStringStrategy {
  pub name: String,
  pub path: String,
}

impl StrategyTrait for UnnamedVersionStringStrategy {
  fn read(&self) {
    println!("Reading UnnamedVersionStringStrategy...");
  }

  fn write(&self) {
    println!("Writing UnnamedVersionStringStrategy...");
  }
}

#[derive(Debug)]
pub struct VersionsByNameStrategy {
  pub name: String,
  pub path: String,
}

impl StrategyTrait for VersionsByNameStrategy {
  fn read(&self) {
    println!("Reading VersionsByNameStrategy...");
  }

  fn write(&self) {
    println!("Writing VersionsByNameStrategy...");
  }
}

#[derive(Debug)]
pub enum Strategy {
  NameAndVersionProps(NameAndVersionPropsStrategy),
  NamedVersionString(NamedVersionStringStrategy),
  UnnamedVersionString(UnnamedVersionStringStrategy),
  VersionsByName(VersionsByNameStrategy),
}

impl Strategy {
  pub fn new(name: &String, config: &config::AnyStrategy) -> Strategy {
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

/// Converts a "some.nested.prop.name" selector to "/some/nested/prop/name"
fn normalize_path(path: String) -> String {
  let mut normalized_path = String::from("/");
  normalized_path.push_str(&path.replace(".", "/"));
  normalized_path
}
