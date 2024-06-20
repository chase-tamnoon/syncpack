#[cfg(test)]
use crate::effects_mock::{MockEffects, PartialMatchEvent, PartialMismatchEvent};

#[cfg(test)]
pub struct ExpectedMatchEvent<'a> {
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub specifier: &'a str,
}

#[cfg(test)]
pub struct ExpectedMismatchEvent<'a> {
  pub dependency_name: &'a str,
  pub instance_id: &'a str,
  pub actual_specifier: &'a str,
  pub expected_specifier: &'a str,
}

#[cfg(test)]
pub fn expect(effects: &MockEffects) -> Expects {
  Expects::new(effects)
}

#[cfg(test)]
pub struct Expects<'a> {
  pub effects: &'a MockEffects,
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

  fn expect_instance_matches(&self, label: &str, expected_matches: &Vec<ExpectedMatchEvent>, actual_matches: &Vec<PartialMatchEvent>) -> &Self {
    if expected_matches.len() != actual_matches.len() {
      panic!("expected {} {} matches but found {}", expected_matches.len(), label, actual_matches.len());
    }

    'expected: for expected in expected_matches {
      let dependency_name = expected.dependency_name.to_string();
      let instance_id = expected.instance_id.to_string();
      let specifier = expected.specifier.to_string();

      for actual in actual_matches {
        if actual.dependency_name == dependency_name && actual.specifier.unwrap().clone() == specifier && actual.instance_id == instance_id {
          continue 'expected;
        }
      }

      panic!("expected {} to be a {} match with {}\n{:#?}", instance_id, label, specifier, actual_matches);
    }

    self
  }

  fn expect_instance_mismatches(&self, label: &str, expected_mismatches: &Vec<ExpectedMismatchEvent>, actual_mismatches: &Vec<PartialMismatchEvent>) -> &Self {
    if expected_mismatches.len() != actual_mismatches.len() {
      panic!("expected {} {} mismatches but found {}", expected_mismatches.len(), label, actual_mismatches.len());
    }

    'expected: for expected in expected_mismatches {
      let dependency_name = expected.dependency_name.to_string();
      let instance_id = expected.instance_id.to_string();
      let actual_specifier = expected.actual_specifier.to_string();
      let expected_specifier = expected.expected_specifier.to_string();

      for actual in actual_mismatches {
        if actual.dependency_name == dependency_name
          && actual.expected_specifier.unwrap().clone() == expected_specifier
          && actual.actual_specifier.unwrap().clone() == actual_specifier
          && actual.expected_specifier.unwrap().clone() == expected_specifier
        {
          continue 'expected;
        }
      }

      panic!(
        "expected {} mismatch for {} from {} to {}\n{:#?}",
        label, instance_id, actual_specifier, expected_specifier, actual_mismatches
      );
    }

    self
  }
}
