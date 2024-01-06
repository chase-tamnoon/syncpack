use crate::dependency_type::DependencyType;
use crate::semver_group::SemverGroup;
use crate::specifier::SpecifierType;

#[derive(Debug)]
pub struct Instance<'a> {
  /// The dependency type to use to read/write this instance
  pub dependency_type: &'a DependencyType,
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// The semver group this instance belongs to
  pub semver_group: Option<&'a SemverGroup<'a>>,
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
      semver_group: None,
      specifier_type: SpecifierType::new(specifier.as_str()),
      specifier,
    }
  }
}
