#[derive(Debug)]
pub struct Instance {
  /// The name of the dependency_type eg. "dev", "peer"
  dependency_type: String,
  dependency_name: String,
  dependency_specifier: String,
}

impl Instance {
  pub fn new(
    dependency_type: String,
    dependency_name: String,
    dependency_specifier: String,
  ) -> Instance {
    Instance {
      dependency_type,
      dependency_name,
      dependency_specifier,
    }
  }
}
