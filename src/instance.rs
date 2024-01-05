use crate::dependency_type::DependencyType;
use crate::specifier::SpecifierType;

#[derive(Debug)]
pub struct Instance<'a> {
  /// The dependency type to use to read/write this instance
  pub dependency_type: &'a DependencyType,
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// The parsed dependency specifier
  pub specifier_type: SpecifierType,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  pub specifier: String,
}

impl<'a> Instance<'a> {
  pub fn new(name: String, specifier: String, dependency_type: &'a DependencyType) -> Instance {
    Instance {
      dependency_type,
      name,
      specifier_type: SpecifierType::new(specifier.as_str()),
      specifier,
    }
  }
}
