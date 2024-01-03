use crate::strategy;

#[derive(Debug)]
pub struct Instance {
  /// The dependency name eg. "react", "react-dom"
  dependency_name: String,
  /// The raw dependency specifier eg. "16.8.0", "^16.8.0"
  dependency_specifier: String,
  /// The strategy to use for this instance
  strategy: strategy::Strategy,
}

impl Instance {
  pub fn new(
    dependency_name: String,
    dependency_specifier: String,
    strategy: strategy::Strategy,
  ) -> Instance {
    Instance {
      dependency_name,
      dependency_specifier,
      strategy,
    }
  }
}
