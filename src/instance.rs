use crate::strategy;

struct Instance {}

impl Instance {
  pub fn new<T: strategy::StrategyTrait>(name: &String, dependency_type: T) -> Instance {
    Instance {}
  }
}
