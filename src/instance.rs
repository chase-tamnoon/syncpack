use crate::dependency_type::DependencyType;
use crate::specifier::Specifier;

#[derive(Debug)]
pub struct Instance {
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  pub specifier: String,
  /// The parsed dependency specifier
  pub package_spec: Specifier,
  /// The strategy to use for this instance
  pub strategy: DependencyType,
}

impl Instance {
  pub fn new(name: String, specifier: String, strategy: DependencyType) -> Instance {
    Instance {
      name,
      package_spec: Specifier::new(specifier.as_str()),
      specifier,
      strategy,
    }
  }
}
