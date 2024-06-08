use serde_json::Value;
use std::path::PathBuf;

use crate::{
  dependency_type::{DependencyType, Strategy},
  package_json::PackageJson,
  specifier::Specifier,
};

pub type InstanceId = String;

#[derive(Debug)]
pub struct Instance {
  /// A unique identifier for this instance
  pub id: InstanceId,
  /// The dependency type to use to read/write this instance
  pub dependency_type: DependencyType,
  /// The file path of the package.json file this instance belongs to
  pub file_path: PathBuf,
  /// Whether this is a package developed in this repo
  pub is_local: bool,
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// The `.name` of the package.json this file is in
  pub package_name: String,
  /// eg. Specifier::Exact("16.8.0"), Specifier::Range("^16.8.0")
  pub specifier: Specifier,
}

impl Instance {
  pub fn new(
    name: String,
    raw_specifier: String,
    dependency_type: DependencyType,
    package: &PackageJson,
  ) -> Instance {
    let package_name = package.get_name();
    Instance {
      id: format!("{} in {} of {}", name, dependency_type.path, package_name),
      dependency_type,
      file_path: package.file_path.clone(),
      is_local: package_name == name,
      name,
      package_name,
      specifier: Specifier::new(&raw_specifier),
    }
  }

  /// Write a version to the package.json
  pub fn set_specifier(&mut self, package: &mut PackageJson, specifier: &Specifier) {
    let raw_specifier = specifier.unwrap();
    match self.dependency_type.strategy {
      Strategy::NameAndVersionProps => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        package.set_prop(path_to_prop_str, Value::String(raw_specifier.clone()));
      }
      Strategy::NamedVersionString => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        let full_value = format!("{}@{}", self.name, &raw_specifier);
        package.set_prop(path_to_prop_str, Value::String(full_value));
      }
      Strategy::UnnamedVersionString => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        package.set_prop(path_to_prop_str, Value::String(raw_specifier.clone()));
      }
      Strategy::VersionsByName => {
        let path_to_obj = &self.dependency_type.path;
        let name = &self.name;
        let path_to_obj_str = path_to_obj.as_str();
        let obj = package
          .contents
          .pointer_mut(path_to_obj_str)
          .unwrap()
          .as_object_mut()
          .unwrap();
        let value = obj.get_mut(name).unwrap();
        *value = Value::String(raw_specifier.clone());
      }
      Strategy::InvalidConfig => {
        panic!("unrecognised strategy");
      }
    };
    // update in-memory state
    self.specifier = specifier.clone();
  }

  /// Delete a version/dependency/instance from the package.json
  pub fn remove_from(&self, package: &mut PackageJson) {
    match self.dependency_type.strategy {
      Strategy::NameAndVersionProps => {
        //
      }
      Strategy::NamedVersionString => {
        //
      }
      Strategy::UnnamedVersionString => {
        //
      }
      Strategy::VersionsByName => {
        let path_to_obj = &self.dependency_type.path;
        let name = &self.name;
        if let Some(value) = package.contents.pointer_mut(path_to_obj) {
          if let Value::Object(obj) = value {
            obj.remove(name);
          }
        }
      }
      Strategy::InvalidConfig => {
        panic!("unrecognised strategy");
      }
    };
  }
}
