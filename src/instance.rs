use std::str::FromStr;

use crate::dependency_type::DependencyType;
use oro_package_spec::{PackageSpec, PackageSpecError};

#[derive(Debug)]
pub struct Instance {
  /// The dependency name eg. "react", "react-dom"
  name: String,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  specifier: String,
  /// The parsed dependency specifier
  package_spec: Result<PackageSpec, PackageSpecError>,
  /// The strategy to use for this instance
  strategy: DependencyType,
}

impl Instance {
  pub fn new(name: String, specifier: String, strategy: DependencyType) -> Instance {
    // eg. "mypackage@1.0.0"
    let name_and_version: &str = &format!("{}@{}", name, specifier);

    Instance {
      name,
      package_spec: PackageSpec::from_str(name_and_version),
      specifier,
      strategy,
    }
  }
}
