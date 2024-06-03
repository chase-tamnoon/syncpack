#[cfg(test)]
use crate::effects_mock::{MatchEventCopy, MismatchEventCopy, MockEffects};

#[cfg(test)]
pub struct ExpectedMatch<'a> {
  pub dependency_name: &'a str,
  pub match_ids: Vec<&'a str>,
  pub matching_version: &'a str,
}

#[cfg(test)]
pub struct ExpectedMismatch<'a> {
  pub dependency_name: &'a str,
  pub mismatch_ids: Vec<&'a str>,
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

  /// Print internal test state for debugging
  pub fn debug(&self) {
    println!("{:#?}", self.effects);
  }

  pub fn to_have_standard_version_group_matches(&self, expected_matches: Vec<ExpectedMatch>) {
    self.expect_instance_matches(
      "standard version group",
      &expected_matches,
      &self.effects.events.instance_matches_standard,
    );
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

  fn expect_instance_matches(
    &self,
    label: &str,
    expected_matches: &Vec<ExpectedMatch>,
    actual_matches: &Vec<MatchEventCopy>,
  ) {
    if expected_matches.len() != actual_matches.len() {
      panic!(
        "expected {} {} matches but found {}",
        expected_matches.len(),
        label,
        actual_matches.len()
      );
    }

    'expected: for expected in expected_matches {
      let dependency_name = expected.dependency_name.to_string();
      let match_ids = &expected
        .match_ids
        .iter()
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
      let matching_version = expected.matching_version.to_string();

      for event in actual_matches {
        if event.dependency_name == dependency_name
          && event.target.0 == matching_version
          && match_ids
            .iter()
            .all(|string| event.target.1.contains(&string))
          && event
            .target
            .1
            .iter()
            .all(|string| match_ids.contains(&string))
        {
          continue 'expected;
        }
      }

      panic!(
        "expected {} to be a {} match with {}\n{:#?}",
        match_ids.join(" and "),
        label,
        matching_version,
        actual_matches
      );
    }
  }

  fn expect_instance_mismatches(
    &self,
    label: &str,
    expected_mismatches: &Vec<ExpectedMismatch>,
    actual_mismatches: &Vec<MismatchEventCopy>,
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
      let mismatch_ids = expected
        .mismatch_ids
        .iter()
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
      let mismatch_version = expected.mismatching_version.to_string();
      let expected_version = expected.expected_version.to_string();

      for event in actual_mismatches {
        if event.dependency_name == dependency_name
          && event.mismatches_with.0 == expected_version
          && event.target.0 == mismatch_version
          && mismatch_ids
            .iter()
            .all(|string| event.target.1.contains(&string))
          && event
            .target
            .1
            .iter()
            .all(|string| mismatch_ids.contains(&string))
        {
          continue 'expected;
        }
      }

      panic!(
        "expected {} mismatch for {} from {} to {}\n{:#?}",
        label,
        mismatch_ids.join(" and "),
        mismatch_version,
        expected_version,
        actual_mismatches
      );
    }
  }
}
