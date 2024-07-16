#[cfg(test)]
use crate::{
  effects::{mock::MockEffects, InstanceEvent, InstanceEventVariant},
  instance::Instance,
};

#[cfg(test)]
#[derive(Debug)]
pub struct ExpectedMatchEvent<'a> {
  pub variant: InstanceEventVariant,
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub actual: &'a str,
}

#[cfg(test)]
#[derive(Debug)]
pub struct ActualMatchEvent {
  pub dependency_name: String,
  pub instance_id: String,
  pub actual: String,
}

#[cfg(test)]
impl ActualMatchEvent {
  pub fn new(event: &InstanceEvent, instance: &Instance) -> Self {
    Self {
      dependency_name: event.dependency.name.clone(),
      instance_id: event.instance_id.clone(),
      actual: instance.actual.unwrap().clone(),
    }
  }
}

#[cfg(test)]
#[derive(Debug)]
pub struct ExpectedMismatchEvent<'a> {
  pub variant: InstanceEventVariant,
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub actual: &'a str,
  pub expected: &'a str,
}

#[cfg(test)]
#[derive(Debug)]
pub struct ActualMismatchEvent {
  pub dependency_name: String,
  pub instance_id: String,
  pub actual: String,
  pub expected: String,
}

#[cfg(test)]
impl ActualMismatchEvent {
  pub fn new(event: &InstanceEvent, instance: &Instance) -> Self {
    Self {
      dependency_name: event.dependency.name.clone(),
      instance_id: event.instance_id.clone(),
      actual: instance.actual.unwrap().clone(),
      expected: instance.expected.unwrap().clone(),
    }
  }
}

#[cfg(test)]
pub fn expect<'a>(effects: &'a MockEffects) -> Expects<'a> {
  Expects::new(effects)
}

#[cfg(test)]
pub struct Expects<'a> {
  pub effects: &'a MockEffects<'a>,
}

#[cfg(test)]
impl<'a> Expects<'a> {
  pub fn new(effects: &'a MockEffects) -> Self {
    Self { effects }
  }

  /// Print internal test state for debugging
  pub fn debug(&self) -> &Self {
    println!("{:#?}", self.effects);
    self
  }

  pub fn to_have_matches(&self, expected_matches: Vec<ExpectedMatchEvent>) -> &Self {
    let actual_matches = &self.effects.matches;
    let expected_len = expected_matches.len();
    let actual_len = actual_matches.values().fold(0, |acc, x| acc + x.len());
    if actual_len != expected_len {
      self.debug();
      panic!("expected {expected_len} matches but found {actual_len}");
    }
    'expected: for expected in &expected_matches {
      let variant = &expected.variant;
      let dependency_name = &expected.dependency_name;
      let instance_id = &expected.instance_id;
      let actual = &expected.actual;
      let matches_of_type = actual_matches.get(variant);
      if matches_of_type.is_none() {
        self.debug();
        panic!("expected {variant:?} match but found none");
      }
      for event in matches_of_type.unwrap() {
        if event.dependency_name == *dependency_name
          && event.instance_id == *instance_id
          && event.actual == *actual
        {
          continue 'expected;
        }
      }
      self.debug();
      println!("{expected:#?}");
      panic!("expected a '{variant:?}' for '{instance_id}' with '{actual}'");
    }
    self
  }

  pub fn to_have_mismatches(&self, expected_mismatches: Vec<ExpectedMismatchEvent>) -> &Self {
    let actual_mismatches = &self.effects.mismatches;
    let expected_len = expected_mismatches.len();
    let actual_len = actual_mismatches.values().fold(0, |acc, x| acc + x.len());
    if actual_len != expected_len {
      self.debug();
      panic!("expected {expected_len} mismatches but found {actual_len}");
    }
    'expected: for expected in &expected_mismatches {
      let variant = &expected.variant;
      let dependency_name = &expected.dependency_name;
      let instance_id = &expected.instance_id;
      let actual = &expected.actual;
      let mismatches_of_type = actual_mismatches.get(variant);
      if mismatches_of_type.is_none() {
        self.debug();
        panic!("expected {variant:?} mismatch but found none");
      }
      for event in mismatches_of_type.unwrap() {
        if event.dependency_name == *dependency_name
          && event.instance_id == *instance_id
          && event.actual == *actual
          && event.expected == *expected.expected
        {
          continue 'expected;
        }
      }
      self.debug();
      println!("{expected:#?}");
      panic!("expected a '{variant:?}' for '{instance_id}' with '{actual}'");
    }
    self
  }
}
