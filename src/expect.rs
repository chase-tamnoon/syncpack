#[cfg(test)]
use crate::effects_mock::{InstanceEventCopy, MockEffects};

#[cfg(test)]
pub struct ExpectedMismatch<'a> {
  pub dependency_name: &'a str,
  pub mismatch_id: &'a str,
  pub mismatching_version: &'a str,
  pub expected_version: &'a str,
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

  pub fn to_have_highest_version_mismatches(&self, expected_mismatches: Vec<ExpectedMismatch>) {
    self.expect_instance_mismatches(
      "highest semver",
      &expected_mismatches,
      &self.effects.events.instance_mismatches_highest_version,
    );
  }

  pub fn to_have_local_version_mismatches(&self, expected_mismatches: Vec<ExpectedMismatch>) {
    self.expect_instance_mismatches(
      "local version",
      &expected_mismatches,
      &self.effects.events.instance_mismatches_local_version,
    );
  }

  fn expect_instance_mismatches(
    &self,
    label: &str,
    expected_mismatches: &Vec<ExpectedMismatch>,
    actual_mismatches: &Vec<InstanceEventCopy>,
  ) {
    if expected_mismatches.len() != actual_mismatches.len() {
      panic!(
        "expected {} {} mismatches but found {}",
        expected_mismatches.len(),
        label,
        actual_mismatches.len()
      );
    }

    'expected: for expected in expected_mismatches {
      let dependency_name = expected.dependency_name.to_string();
      let mismatch_id = expected.mismatch_id.to_string();
      let mismatch_version = expected.mismatching_version.to_string();
      let expected_version = expected.expected_version.to_string();

      for event in actual_mismatches {
        if event.dependency_name == dependency_name
          && event.mismatches_with.0 == expected_version
          && event.target.0 == mismatch_version
          && event.target.1.contains(&mismatch_id)
        {
          continue 'expected;
        }
      }

      panic!(
        "expected {} mismatch for {} from {} to {}\n{:#?}",
        label, mismatch_id, mismatch_version, expected_version, actual_mismatches
      );
    }
  }
}
