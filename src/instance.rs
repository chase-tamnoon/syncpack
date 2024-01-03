use crate::dependency_type;

#[derive(Debug)]
pub struct Instance {
  /// The dependency name eg. "react", "react-dom"
  name: String,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  specifier: String,
  /// The strategy to use for this instance
  strategy: dependency_type::DependencyType,
}

impl Instance {
  pub fn new(name: String, specifier: String, strategy: dependency_type::DependencyType) -> Instance {
    Instance {
      name,
      specifier,
      strategy,
    }
  }
}
