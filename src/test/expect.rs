use crate::{
  effects::{InstanceEvent, InstanceState},
  instance::Instance,
  test::mock_effects::MockEffects,
};

#[derive(Debug)]
pub struct ExpectedMatchEvent<'a> {
  pub variant: InstanceState,
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub actual: &'a str,
}

#[derive(Debug)]
pub struct ActualMatchEvent {
  pub dependency_name: String,
  pub instance_id: String,
  pub actual: String,
}

impl ActualMatchEvent {
  pub fn new(event: &InstanceEvent, instance: &Instance) -> Self {
    Self {
      dependency_name: event.dependency.name.clone(),
      instance_id: event.instance.id.clone(),
      actual: instance.actual_specifier.unwrap().clone(),
    }
  }
}

#[derive(Debug)]
pub struct ExpectedUnfixableMismatchEvent<'a> {
  pub variant: InstanceState,
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub actual: &'a str,
}

#[derive(Debug)]
pub struct ActualUnfixableMismatchEvent {
  pub dependency_name: String,
  pub instance_id: String,
  pub actual: String,
}

impl ActualUnfixableMismatchEvent {
  pub fn new(event: &InstanceEvent, instance: &Instance) -> Self {
    Self {
      dependency_name: event.dependency.name.clone(),
      instance_id: event.instance.id.clone(),
      actual: instance.actual_specifier.unwrap().clone(),
    }
  }
}

#[derive(Debug)]
pub struct ExpectedFixableMismatchEvent<'a> {
  pub variant: InstanceState,
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub actual: &'a str,
  pub expected: &'a str,
}

#[derive(Debug)]
pub struct ActualFixableMismatchEvent {
  pub dependency_name: String,
  pub instance_id: String,
  pub actual: String,
  pub expected: String,
}

impl ActualFixableMismatchEvent {
  pub fn new(event: &InstanceEvent, instance: &Instance) -> Self {
    Self {
      dependency_name: event.dependency.name.clone(),
      instance_id: event.instance.id.clone(),
      actual: instance.actual_specifier.unwrap().clone(),
      expected: instance.expected_specifier.borrow().as_ref().unwrap().unwrap().clone(),
    }
  }
}

#[derive(Debug)]
pub struct ExpectedOverrideEvent<'a> {
  pub variant: InstanceState,
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub actual: &'a str,
  pub expected: &'a str,
  pub overridden: &'a str,
}

#[derive(Debug)]
pub struct ActualOverrideEvent {
  pub dependency_name: String,
  pub instance_id: String,
  pub actual: String,
  pub expected: String,
  pub overridden: String,
}

impl ActualOverrideEvent {
  pub fn new(event: &InstanceEvent, instance: &Instance, overridden: String) -> Self {
    Self {
      dependency_name: event.dependency.name.clone(),
      instance_id: event.instance.id.clone(),
      actual: instance.actual_specifier.unwrap().clone(),
      expected: instance.expected_specifier.borrow().as_ref().unwrap().unwrap().clone(),
      overridden,
    }
  }
}

pub fn expect<'a>(effects: &'a MockEffects) -> Expects<'a> {
  Expects::new(effects)
}

pub struct Expects<'a> {
  pub effects: &'a MockEffects<'a>,
}

impl<'a> Expects<'a> {
  pub fn new(effects: &'a MockEffects) -> Self {
    Self { effects }
  }

  /// Print internal test state for debugging
  pub fn debug(&self) -> &Self {
    println!("{:#?}", self.effects);
    self
  }

  pub fn to_have_overrides(&self, expected_overrides: Vec<ExpectedOverrideEvent>) -> &Self {
    let actual_overrides = &self.effects.overrides;
    let expected_len = expected_overrides.len();
    let actual_len = actual_overrides.values().fold(0, |acc, x| acc + x.len());
    if actual_len != expected_len {
      self.debug();
      panic!("expected {expected_len} overrides but found {actual_len}");
    }
    'expected: for expected in &expected_overrides {
      let variant = &expected.variant;
      let dependency_name = &expected.dependency_name;
      let instance_id = &expected.instance_id;
      let actual = &expected.actual;
      let overridden = &expected.overridden;
      let expected_specifier = &expected.expected;
      let overrides_of_type = actual_overrides.get(variant);
      if overrides_of_type.is_none() {
        self.debug();
        panic!("expected {variant:?} override but found none");
      }
      for event in overrides_of_type.unwrap() {
        if event.dependency_name == *dependency_name
          && event.instance_id == *instance_id
          && event.actual == *actual
          && event.overridden == *overridden
          && event.expected == *expected_specifier
        {
          continue 'expected;
        }
      }
      self.debug();
      panic!("expected a '{variant:?}' for '{instance_id}' with '{actual}' overridden by '{overridden}' instead of '{expected_specifier}'");
    }
    self
  }

  pub fn to_have_warnings(&self, expected_warnings: Vec<ExpectedUnfixableMismatchEvent>) -> &Self {
    let actual_warnings = &self.effects.warnings;
    let expected_len = expected_warnings.len();
    let actual_len = actual_warnings.values().fold(0, |acc, x| acc + x.len());
    if actual_len != expected_len {
      self.debug();
      panic!("expected {expected_len} warnings but found {actual_len}");
    }
    'expected: for expected in &expected_warnings {
      let variant = &expected.variant;
      let dependency_name = &expected.dependency_name;
      let instance_id = &expected.instance_id;
      let actual = &expected.actual;
      let matches_of_type = actual_warnings.get(variant);
      if matches_of_type.is_none() {
        self.debug();
        panic!("expected {variant:?} warning but found none");
      }
      for event in matches_of_type.unwrap() {
        if event.dependency_name == *dependency_name && event.instance_id == *instance_id && event.actual == *actual {
          continue 'expected;
        }
      }
      self.debug();
      panic!("expected a warning on '{variant:?}' for '{instance_id}' with '{actual}'");
    }
    self
  }

  pub fn to_have_warnings_of_instance_changes(&self, expected_warnings_of_instance_changes: Vec<ExpectedFixableMismatchEvent>) -> &Self {
    let actual_warnings_of_instance_changes = &self.effects.warnings_of_instance_changes;
    let expected_len = expected_warnings_of_instance_changes.len();
    let actual_len = actual_warnings_of_instance_changes.values().fold(0, |acc, x| acc + x.len());
    if actual_len != expected_len {
      self.debug();
      panic!("expected {expected_len} warnings of instance changes but found {actual_len}");
    }
    'expected: for expected in &expected_warnings_of_instance_changes {
      let variant = &expected.variant;
      let dependency_name = &expected.dependency_name;
      let instance_id = &expected.instance_id;
      let actual = &expected.actual;
      let expected_specifier = &expected.expected;
      let mismatches_of_type = actual_warnings_of_instance_changes.get(variant);
      if mismatches_of_type.is_none() {
        self.debug();
        panic!("expected {variant:?} warnings of instance change but found none");
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
      panic!(
        "expected a warning of instance change '{variant:?}' for '{instance_id}' with '{actual}' to be replaced by '{expected_specifier}'"
      );
    }
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
        if event.dependency_name == *dependency_name && event.instance_id == *instance_id && event.actual == *actual {
          continue 'expected;
        }
      }
      self.debug();
      panic!("expected a matching '{variant:?}' for '{instance_id}' with '{actual}'");
    }
    self
  }

  pub fn to_have_unfixable_mismatches(&self, expected_mismatches: Vec<ExpectedUnfixableMismatchEvent>) -> &Self {
    let actual_mismatches = &self.effects.unfixable_mismatches;
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
        if event.dependency_name == *dependency_name && event.instance_id == *instance_id && event.actual == *actual {
          continue 'expected;
        }
      }
      self.debug();
      panic!("expected an unfixable '{variant:?}' for '{instance_id}' with '{actual}'");
    }
    self
  }

  pub fn to_have_fixable_mismatches(&self, expected_mismatches: Vec<ExpectedFixableMismatchEvent>) -> &Self {
    let actual_mismatches = &self.effects.fixable_mismatches;
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
      let expected_specifier = &expected.expected;
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
      panic!("expected a fixable '{variant:?}' for '{instance_id}' with '{actual}' to be replaced by '{expected_specifier}'");
    }
    self
  }
}
