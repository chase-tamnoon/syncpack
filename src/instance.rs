use serde_json::Value;
use std::path::PathBuf;

use crate::{
  dependency_type::{DependencyType, Strategy},
  package_json::PackageJson,
  semver_group::SemverGroup,
  specifier::{semver::Semver, semver_range::SemverRange, Specifier},
};

pub type InstanceId = String;

#[derive(Debug)]
pub struct Instance {
  /// The original version specifier, which should never be mutated.
  /// eg. `Specifier::Exact("16.8.0")`, `Specifier::Range("^16.8.0")`
  pub actual: Specifier,
  /// The dependency type to use to read/write this instance
  pub dependency_type: DependencyType,
  /// The latest version specifier which is mutated by Syncpack
  pub expected: Specifier,
  /// The file path of the package.json file this instance belongs to
  pub file_path: PathBuf,
  /// A unique identifier for this instance
  pub id: InstanceId,
  /// Whether this is a package developed in this repo
  pub is_local: bool,
  /// eg. "in /devDependencies of @foo/numberwang"
  pub location_hint: String,
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// The `.name` of the package.json this file is in
  pub package_name: String,
  /// If this instance belongs to a `WithRange` semver group, this is the range.
  /// This is used by Version Groups while determining the preferred version,
  /// to try to also satisfy any applicable semver group ranges
  pub prefer_range: Option<SemverRange>,
}

impl Instance {
  pub fn new(
    name: String,
    // The initial, unwrapped specifier (eg. "1.1.0") from the package.json file
    raw_specifier: String,
    dependency_type: &DependencyType,
    package: &PackageJson,
  ) -> Instance {
    let package_name = package.get_name();
    let specifier = Specifier::new(&raw_specifier);
    Instance {
      actual: specifier.clone(),
      dependency_type: dependency_type.clone(),
      expected: specifier,
      file_path: package.file_path.clone(),
      id: format!("{} in {} of {}", name, &dependency_type.path, package_name),
      is_local: dependency_type.path == "/version",
      location_hint: format!("in {} of {}", &dependency_type.path, package_name),
      name,
      package_name,
      prefer_range: None,
    }
  }

  /// Updated the expected version specifier for this instance to match the
  /// preferred semver range of the given semver group
  pub fn apply_semver_group(&mut self, group: &SemverGroup) {
    group.range.as_ref().map(|range| {
      self.prefer_range = Some(range.clone());
      if let Some(expected) = self.expected.get_simple_semver() {
        self.expected = Specifier::Semver(Semver::Simple(expected.with_range(range)));
      }
    });
  }

  /// Does this instance's specifier match the expected specifier for this
  /// dependency except for by its own semver group's preferred semver range?
  ///
  /// ✓ it has a semver group
  /// ✓ its own version matches its expected version (eg. "1.1.0" == "1.1.0")
  /// ✓ its expected version matches the expected version of the group
  /// ✘ only its own semver range is different
  pub fn has_range_mismatch(&self, other: &Specifier) -> bool {
    // it has a semver group
    self.prefer_range.is_some()
      && match (
        self.actual.get_simple_semver(),
        self.expected.get_simple_semver(),
        other.get_simple_semver(),
      ) {
        // all versions are simple semver
        (Some(actual), Some(expected), Some(other)) => {
          // its own version matches its expected version (eg. "1.1.0" == "1.1.0")
          actual.has_same_version(&expected)
          // its expected version matches the expected version of the group
          && expected.has_same_version(&other)
          // only its own semver range is different
          && !expected.has_same_range(&actual)
        }
        _ => false,
      }
  }

  pub fn get_fixed_range_mismatch(&self) -> Specifier {
    self
      .prefer_range
      .as_ref()
      .and_then(|prefer_range| {
        self
          .expected
          .get_simple_semver()
          .map(|expected| expected.with_range(prefer_range))
      })
      .map(|simple_semver| Specifier::Semver(Semver::Simple(simple_semver)))
      .expect("Failed to fix semver range mismatch")
  }

  /// Write a version to the package.json
  pub fn set_specifier(&mut self, package: &mut PackageJson, specifier: &Specifier) {
    match self.dependency_type.strategy {
      Strategy::NameAndVersionProps => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        let raw_specifier = specifier.unwrap();
        package.set_prop(path_to_prop_str, Value::String(raw_specifier));
      }
      Strategy::NamedVersionString => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        let raw_specifier = specifier.unwrap();
        let full_value = format!("{}@{}", self.name, raw_specifier);
        package.set_prop(path_to_prop_str, Value::String(full_value));
      }
      Strategy::UnnamedVersionString => {
        let path_to_prop = &self.dependency_type.path;
        let path_to_prop_str = path_to_prop.as_str();
        let raw_specifier = specifier.unwrap();
        package.set_prop(path_to_prop_str, Value::String(raw_specifier));
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
        let raw_specifier = specifier.unwrap();
        *value = Value::String(raw_specifier);
      }
      Strategy::InvalidConfig => {
        panic!("unrecognised strategy");
      }
    };
    // update in-memory state
    self.expected = specifier.clone();
  }

  /// Delete a version/dependency/instance from the package.json
  pub fn remove_from(&self, package: &mut PackageJson) {
    match self.dependency_type.strategy {
      Strategy::NameAndVersionProps => {
        println!("@TODO: remove instance for NameAndVersionProps");
      }
      Strategy::NamedVersionString => {
        println!("@TODO: remove instance for NamedVersionString");
      }
      Strategy::UnnamedVersionString => {
        println!("@TODO: remove instance for UnnamedVersionString");
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
