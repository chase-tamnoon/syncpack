use log::debug;
use serde_json::Value;
use std::{cell::RefCell, path::PathBuf};

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
  pub expected: RefCell<Specifier>,
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
  pub prefer_range: RefCell<Option<SemverRange>>,
}

impl Instance {
  pub fn new(
    name: String,
    // The initial, unwrapped specifier (eg. "1.1.0") from the package.json file
    raw_specifier: String,
    dependency_type: &DependencyType,
    package: &PackageJson,
  ) -> Instance {
    let package_name = package.get_name_unsafe();
    let specifier = Specifier::new(&raw_specifier);
    Instance {
      actual: specifier.clone(),
      dependency_type: dependency_type.clone(),
      expected: RefCell::new(specifier),
      file_path: package.file_path.clone(),
      id: format!("{} in {} of {}", name, &dependency_type.path, package_name),
      is_local: dependency_type.path == "/version",
      location_hint: format!("in {} of {}", &dependency_type.path, package_name),
      name,
      package_name,
      prefer_range: RefCell::new(None),
    }
  }

  /// Log every property of this instance
  pub fn log_debug(&self) {
    debug!("Instance:");
    debug!("  actual          {:?}", self.actual);
    debug!("  dependency_type {:?}", self.dependency_type);
    debug!("  expected        {:?}", self.expected);
    debug!("  file_path       {:?}", self.file_path);
    debug!("  id              {:?}", self.id);
    debug!("  is_local        {:?}", self.is_local);
    debug!("  location_hint   {:?}", self.location_hint);
    debug!("  name            {:?}", self.name);
    debug!("  package_name    {:?}", self.package_name);
    debug!("  prefer_range    {:?}", self.prefer_range);
  }

  /// Updated the expected version specifier for this instance to match the
  /// preferred semver range of the given semver group
  pub fn apply_semver_group(&self, group: &SemverGroup) {
    if let Some(range) = &group.range {
      let mut prefer_range = self.prefer_range.borrow_mut();
      let mut expected = self.expected.borrow_mut();
      *prefer_range = Some(range.clone());
      if let Some(simple_semver) = expected.get_simple_semver() {
        *expected = Specifier::Semver(Semver::Simple(simple_semver.with_range(range)));
      }
      std::mem::drop(prefer_range);
      std::mem::drop(expected);
    }
  }

  /// Does this instance's specifier match the expected specifier for this
  /// dependency except for by its own semver group's preferred semver range?
  ///
  /// ✓ it has a semver group
  /// ✓ its own version matches its expected version (eg. "1.1.0" == "1.1.0")
  /// ✓ its expected version matches the expected version of the group
  /// ✘ only its own semver range is different
  pub fn has_range_mismatch(&self, preferred: &Specifier) -> bool {
    // it has a semver group
    self.prefer_range.borrow().is_some()
      && match (
        self.actual.get_simple_semver(),
        self.expected.borrow().get_simple_semver(),
        preferred.get_simple_semver(),
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

  /// Does the given semver specifier have the expected range for this
  /// instance's semver group?
  pub fn matches_semver_group(&self, specifier: &Specifier) -> bool {
    self
      .prefer_range
      .borrow()
      .as_ref()
      .and_then(|range| {
        specifier
          .get_simple_semver()
          .map(|simple_semver| simple_semver.get_range() == *range)
      })
      .unwrap_or(false)
  }

  /// Get the expected version specifier for this instance with the semver
  /// group's preferred range applied
  pub fn get_fixed_range_mismatch(&self) -> Specifier {
    self
      .prefer_range
      .borrow()
      .as_ref()
      .and_then(|range| {
        self
          .expected
          .borrow()
          .get_simple_semver()
          .map(|expected| expected.with_range(range))
      })
      .map(|simple_semver| Specifier::Semver(Semver::Simple(simple_semver)))
      .expect("Failed to fix semver range mismatch")
  }

  /// Write a version to the package.json
  pub fn set_specifier(&self, package: &PackageJson, specifier: &Specifier) {
    match self.dependency_type.strategy {
      Strategy::NameAndVersionProps => {
        let path_to_prop_str = &self.dependency_type.path.as_str();
        let raw_specifier = specifier.unwrap();
        package.set_prop(path_to_prop_str, Value::String(raw_specifier));
      }
      Strategy::NamedVersionString => {
        let path_to_prop_str = &self.dependency_type.path.as_str();
        let raw_specifier = specifier.unwrap();
        let full_value = format!("{}@{}", self.name, raw_specifier);
        package.set_prop(path_to_prop_str, Value::String(full_value));
      }
      Strategy::UnnamedVersionString => {
        let path_to_prop_str = &self.dependency_type.path.as_str();
        let raw_specifier = specifier.unwrap();
        package.set_prop(path_to_prop_str, Value::String(raw_specifier));
      }
      Strategy::VersionsByName => {
        let path_to_obj_str = &self.dependency_type.path.as_str();
        let raw_specifier = specifier.unwrap();
        let mut contents = package.contents.borrow_mut();
        let versions_by_name = contents.pointer_mut(path_to_obj_str).unwrap().as_object_mut().unwrap();
        let old_specifier = versions_by_name.get_mut(&self.name).unwrap();
        *old_specifier = Value::String(raw_specifier);
        std::mem::drop(contents);
      }
      Strategy::InvalidConfig => {
        panic!("unrecognised strategy");
      }
    };
    // update in-memory state
    *self.expected.borrow_mut() = specifier.clone();
  }

  /// Delete a version/dependency/instance from the package.json
  pub fn remove_from(&self, package: &PackageJson) {
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
        if let Some(Value::Object(obj)) = package.contents.borrow_mut().pointer_mut(path_to_obj) {
          obj.remove(name);
        }
      }
      Strategy::InvalidConfig => {
        panic!("unrecognised strategy");
      }
    };
  }
}
