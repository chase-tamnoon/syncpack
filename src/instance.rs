use crate::dependency_type::DependencyType;
use crate::package_json;
use crate::specifier::SpecifierType;

#[derive(Debug)]
pub struct Instance<'a> {
  /// The dependency type to use to read/write this instance
  pub dependency_type: &'a DependencyType,
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// The range from the semver group this instance belongs to
  pub expected_range: Option<String>,
  /// Whether this is a package developed in this repo
  pub is_local: bool,
  /// The package.json file this instance belongs to
  pub package_json: &'a package_json::PackageJson,
  /// The parsed dependency specifier
  pub specifier_type: SpecifierType,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  pub specifier: String,
  /// The parsed dependency specifier after it has been fixed, if it was fixed
  pub fixed_specifier_type: Option<SpecifierType>,
  /// The raw dependency specifier after it has been fixed, if it was fixed
  pub fixed_specifier: Option<String>,
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
      expected_range: None,
      is_local: match file.get_prop("/name") {
        Some(package_name) => package_name == &name,
        None => false,
      },
      name,
      package_json: file,
      specifier_type: SpecifierType::new(specifier.as_str()),
      specifier,
      fixed_specifier_type: None,
      fixed_specifier: None,
    }
  }
}
