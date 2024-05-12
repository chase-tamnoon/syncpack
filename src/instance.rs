use std::path::PathBuf;

use log::debug;

use crate::dependency_type::DependencyType;
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
