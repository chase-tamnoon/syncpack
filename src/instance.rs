use std::path::PathBuf;

use log::debug;
use serde_json::Value;

use crate::dependency_type::{DependencyType, Strategy};
use crate::package_json::PackageJson;
use crate::specifier::Specifier;

#[derive(Debug)]
pub struct Instance {
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
  /// The parsed dependency specifier
  pub specifier_type: Specifier,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  pub specifier: String,
}

impl<'a> Instance {
  pub fn new(
    name: String,
    specifier: String,
    dependency_type: DependencyType,
    package: &PackageJson,
  ) -> Instance {
    let package_name = package.get_name();
    Instance {
      dependency_type,
      file_path: package.file_path.clone(),
      is_local: package_name == name,
      name,
      package_name,
      specifier_type: Specifier::new(specifier.as_str()),
      specifier: sanitise_specifier(specifier),
    }
  }

  /// Write a version to the package.json
  pub fn set_version(&self, package: &mut PackageJson, value: String) {
    match self.dependency_type.strategy {
      Strategy::NameAndVersionProps => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        package.set_prop(path_to_prop_str, Value::String(value));
      }
      Strategy::NamedVersionString => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        let full_value = format!("{}@{}", self.name, value);
        package.set_prop(path_to_prop_str, Value::String(full_value));
      }
      Strategy::UnnamedVersionString => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        package.set_prop(path_to_prop_str, Value::String(value));
      }
      Strategy::VersionsByName => {
        let path_to_obj = &self.dependency_type.path;
        let name = &self.name;
        let path_to_obj_str = path_to_obj.as_str();
        if let Some(obj) = package.contents.pointer_mut(path_to_obj_str) {
          if let Value::Object(obj) = obj {
            obj.insert(name.clone(), Value::String(value));
          }
        }
      }
      Strategy::InvalidConfig => {
        panic!("unrecognised strategy");
      }
    };
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
        let path_to_obj_str = path_to_obj.as_str();
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

/// Convert non-semver specifiers to semver when behaviour is identical
fn sanitise_specifier(specifier: String) -> String {
  if specifier == "latest" || specifier == "x" {
    debug!("Sanitising specifier: {} -> *", specifier);
    "*".to_string()
  } else {
    specifier
  }
}
