use crate::package_json;
use crate::strategy;

#[derive(Debug)]
pub struct Instance<'a> {
  /// The name of the dependency_type eg. "dev", "peer"
  dependency_type: String,
  /// The dependency name eg. "react", "react-dom"
  dependency_name: String,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  dependency_specifier: String,
  /// The strategy to use for this instance
  strategy: &'a strategy::Strategy,
  /// The package.json file this instance belongs to
  file: &'a package_json::PackageJson,
}

impl<'a> Instance<'a> {
  pub fn new(
    dependency_type: String,
    dependency_name: String,
    dependency_specifier: String,
    strategy: &'a strategy::Strategy,
    file: &'a package_json::PackageJson,
  ) -> Instance<'a> {
    Instance {
      dependency_type,
      dependency_name,
      dependency_specifier,
      strategy,
      file,
    }
  }
}
