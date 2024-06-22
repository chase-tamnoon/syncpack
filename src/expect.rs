#[cfg(test)]
use crate::effects_mock::MockEffects;

#[cfg(test)]
#[derive(Debug)]
pub struct ExpectedMatchEvent<'a> {
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
#[derive(Debug)]
pub struct ExpectedMismatchEvent<'a> {
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

  pub fn to_have_standard_version_group_matches(&self, expected_matches: Vec<ExpectedMatchEvent>) -> &Self {
    self.expect_instance_matches("standard version group", &expected_matches, &self.effects.events.instance_matches_standard)
  }

  pub fn to_have_highest_version_mismatches(&self, expected_mismatches: Vec<ExpectedMismatchEvent>) -> &Self {
    self.expect_instance_mismatches("highest semver", &expected_mismatches, &self.effects.events.instance_mismatches_highest_version)
  }

  pub fn to_have_rejected_local_version_mismatches(&self, expected_mismatches: Vec<ExpectedMismatchEvent>) -> &Self {
    self.expect_instance_mismatches("rejected local version", &expected_mismatches, &self.effects.events.instance_mismatch_changes_local_version)
  }

  pub fn to_have_local_version_mismatches(&self, expected_mismatches: Vec<ExpectedMismatchEvent>) -> &Self {
    self.expect_instance_mismatches("local version", &expected_mismatches, &self.effects.events.instance_mismatches_local_version)
  }

  pub fn to_have_pinned_version_mismatches(&self, expected_mismatches: Vec<ExpectedMismatchEvent>) -> &Self {
    self.expect_instance_mismatches("highest semver", &expected_mismatches, &self.effects.events.instance_mismatches_pinned_version)
  }

  pub fn to_have_semver_range_mismatches(&self, expected_mismatches: Vec<ExpectedMismatchEvent>) -> &Self {
    self.expect_instance_mismatches("semver range", &expected_mismatches, &self.effects.events.instance_mismatches_semver_range)
  }

  fn expect_instance_matches(&self, label: &str, expected_matches: &Vec<ExpectedMatchEvent>, actual_matches: &Vec<ActualMatchEvent>) -> &Self {
    if expected_matches.len() != actual_matches.len() {
      panic!("expected {} {} matches but found {}", expected_matches.len(), label, actual_matches.len());
    }
    'expected: for expected in expected_matches {
      let expected_dependency_name = expected.dependency_name.to_string();
      let expected_instance_id = expected.instance_id.to_string();
      let expected_actual_specifier = expected.actual.to_string();
      for actual in actual_matches {
        let actual_dependency_name = actual.dependency_name.clone();
        let actual_instance_id = actual.instance_id.clone();
        let actual_actual_specifier = actual.actual.clone();
        if actual_dependency_name == expected_dependency_name && actual_actual_specifier == expected_actual_specifier && actual_instance_id == expected_instance_id {
          continue 'expected;
        }
      }
      panic!("expected {expected_instance_id} to be a {label} match with {expected_actual_specifier}\n{actual_matches:#?}");
    }
    self
  }

  fn expect_instance_mismatches(&self, label: &str, expected_mismatches: &Vec<ExpectedMismatchEvent>, actual_mismatches: &Vec<ActualMismatchEvent>) -> &Self {
    if expected_mismatches.len() != actual_mismatches.len() {
      panic!("expected {} {} mismatches but found {}", expected_mismatches.len(), label, actual_mismatches.len());
    }
    'expected: for expected in expected_mismatches {
      let expected_dependency_name = expected.dependency_name.to_string();
      let expected_instance_id = expected.instance_id.to_string();
      let expected_actual_specifier = expected.actual.to_string();
      let expected_expected_specifier = expected.expected.to_string();
      for actual in actual_mismatches {
        let actual_dependency_name = actual.dependency_name.clone();
        let actual_instance_id = actual.instance_id.clone();
        let actual_actual_specifier = actual.actual.clone();
        let actual_expected_specifier = actual.expected.clone();
        if actual_dependency_name == expected_dependency_name
          && actual_expected_specifier == expected_expected_specifier
          && actual_actual_specifier == expected_actual_specifier
          && actual_expected_specifier == expected_expected_specifier
        {
          continue 'expected;
        }
      }
      panic!("expected {label} mismatch for {expected_instance_id} from {expected_actual_specifier} to {expected_expected_specifier}\n{actual_mismatches:#?}");
    }
    self
  }
}
