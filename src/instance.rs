use log::debug;

use crate::dependency_type::DependencyType;
use crate::package_json;
use crate::specifier::Specifier;

#[derive(Debug)]
pub struct Instance<'a> {
  /// The dependency type to use to read/write this instance
  pub dependency_type: &'a DependencyType,
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// Whether this is a package developed in this repo
  pub is_local: bool,
  /// The package.json file this instance belongs to
  pub package_json: &'a package_json::PackageJson,
  /// The parsed dependency specifier
  pub specifier_type: Specifier,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  pub specifier: String,
}

impl<'a> Instance<'a> {
  pub fn new(
    name: String,
    specifier: String,
    dependency_type: &'a DependencyType,
    file: &'a package_json::PackageJson,
  ) -> Instance<'a> {
    Instance {
      dependency_type,
      is_local: match file.get_prop("/name") {
        Some(package_name) => package_name == &name,
        None => false,
      },
      name,
      package_json: file,
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
