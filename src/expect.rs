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

  pub fn to_have_local_instance_is_preferred(
    &self,
    expected_matches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "local instance is preferred";
    let actual_matches = &self.effects.events.local_instance_is_preferred;
    self.expect_instance_matches(label, &expected_matches, actual_matches)
  }

  /* Matches */

  pub fn to_have_instance_matches_local(&self, expected_matches: Vec<ExpectedMatchEvent>) -> &Self {
    let label = "instance matches local";
    let actual_matches = &self.effects.events.instance_matches_local;
    self.expect_instance_matches(label, &expected_matches, actual_matches)
  }

  pub fn to_have_instance_matches_highest_or_lowest_semver(
    &self,
    expected_matches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance matches highest or lowest semver";
    let actual_matches = &self
      .effects
      .events
      .instance_matches_highest_or_lowest_semver;
    self.expect_instance_matches(label, &expected_matches, actual_matches)
  }

  pub fn to_have_instance_matches_but_is_unsupported(
    &self,
    expected_matches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance matches but is unsupported";
    let actual_matches = &self.effects.events.instance_matches_but_is_unsupported;
    self.expect_instance_matches(label, &expected_matches, actual_matches)
  }

  pub fn to_have_instance_matches_pinned(
    &self,
    expected_matches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance matches pinned";
    let actual_matches = &self.effects.events.instance_matches_pinned;
    self.expect_instance_matches(label, &expected_matches, actual_matches)
  }

  pub fn to_have_instance_matches_same_range_group(
    &self,
    expected_matches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance matches same range group";
    let actual_matches = &self.effects.events.instance_matches_same_range_group;
    self.expect_instance_matches(label, &expected_matches, actual_matches)
  }

  /* Warnings */

  pub fn to_have_local_instance_mistakenly_banned(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "local instance mistakenly banned";
    let actual_matches = &self.effects.events.local_instance_mistakenly_banned;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  pub fn to_have_local_instance_mistakenly_mismatches_semver_group(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "local instance mistakenly mismatches semver group";
    let actual_mismatches = &self
      .effects
      .events
      .local_instance_mistakenly_mismatches_semver_group;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  pub fn to_have_local_instance_mistakenly_mismatches_pinned(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "local instance mistakenly mismatches pinned";
    let actual_mismatches = &self
      .effects
      .events
      .local_instance_mistakenly_mismatches_pinned;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  pub fn to_have_instance_matches_highest_or_lowest_semver_but_mismatches_conflicting_semver_group(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "instance matches highest or lowest semver but mismatches conflicting semver group";
    let actual_mismatches = &self
      .effects
      .events
      .instance_matches_highest_or_lowest_semver_but_mismatches_conflicting_semver_group;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  /* Fixable Mismatches */

  pub fn to_have_instance_is_banned(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "instance is banned";
    let actual_mismatches = &self.effects.events.instance_is_banned;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  pub fn to_have_instance_is_highest_or_lowest_semver_once_semver_group_is_fixed(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "instance is highest or lowest semver once semver group is fixed";
    let actual_mismatches = &self
      .effects
      .events
      .instance_is_highest_or_lowest_semver_once_semver_group_is_fixed;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  pub fn to_have_instance_matches_local_but_mismatches_semver_group(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "instance matches local but mismatches semver group";
    let actual_mismatches = &self
      .effects
      .events
      .instance_matches_local_but_mismatches_semver_group;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  pub fn to_have_instance_mismatches_local(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "instance mismatches local";
    let actual_mismatches = &self.effects.events.instance_mismatches_local;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  pub fn to_have_instance_mismatches_highest_or_lowest_semver(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "instance mismatches highest or lowest semver";
    let actual_mismatches = &self
      .effects
      .events
      .instance_mismatches_highest_or_lowest_semver;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  pub fn to_have_instance_mismatches_pinned(
    &self,
    expected_mismatches: Vec<ExpectedMismatchEvent>,
  ) -> &Self {
    let label = "instance mismatches pinned";
    let actual_mismatches = &self.effects.events.instance_mismatches_pinned;
    self.expect_instance_mismatches(label, &expected_mismatches, actual_mismatches)
  }

  /* Unfixable Mismatches */

  pub fn to_have_instance_mismatches_and_is_unsupported(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance mismatches and is unsupported";
    let actual_matches = &self.effects.events.instance_mismatches_and_is_unsupported;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  pub fn to_have_instance_matches_pinned_but_mismatches_semver_group(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance matches pinned but mismatches semver group";
    let actual_matches = &self
      .effects
      .events
      .instance_matches_pinned_but_mismatches_semver_group;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  pub fn to_have_instance_mismatches_both_same_range_and_conflicting_semver_groups(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance mismatches both same range and conflicting semver groups";
    let actual_matches = &self
      .effects
      .events
      .instance_mismatches_both_same_range_and_conflicting_semver_groups;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  pub fn to_have_instance_mismatches_both_same_range_and_compatible_semver_groups(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance mismatches both same range and compatible semver groups";
    let actual_matches = &self
      .effects
      .events
      .instance_mismatches_both_same_range_and_compatible_semver_groups;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  pub fn to_have_instance_matches_same_range_group_but_mismatches_conflicting_semver_group(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance matches same range group but mismatches conflicting semver group";
    let actual_matches = &self
      .effects
      .events
      .instance_matches_same_range_group_but_mismatches_conflicting_semver_group;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  pub fn to_have_instance_matches_same_range_group_but_mismatches_compatible_semver_group(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance matches same range group but mismatches compatible semver group";
    let actual_matches = &self
      .effects
      .events
      .instance_matches_same_range_group_but_mismatches_compatible_semver_group;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  pub fn to_have_instance_mismatches_same_range_group(
    &self,
    expected_mismatches: Vec<ExpectedMatchEvent>,
  ) -> &Self {
    let label = "instance mismatches same range group";
    let actual_matches = &self.effects.events.instance_mismatches_same_range_group;
    self.expect_instance_matches(label, &expected_mismatches, actual_matches)
  }

  fn expect_instance_matches(
    &self,
    label: &str,
    expected_matches: &Vec<ExpectedMatchEvent>,
    actual_matches: &Vec<ActualMatchEvent>,
  ) -> &Self {
    if expected_matches.len() != actual_matches.len() {
      self.debug();
      panic!(
        "expected {} {} matches but found {}",
        expected_matches.len(),
        label,
        actual_matches.len()
      );
    }
    'expected: for expected in expected_matches {
      let expected_dependency_name = expected.dependency_name.to_string();
      let expected_instance_id = expected.instance_id.to_string();
      let expected_actual_specifier = expected.actual.to_string();
      for actual in actual_matches {
        let actual_dependency_name = actual.dependency_name.clone();
        let actual_instance_id = actual.instance_id.clone();
        let actual_actual_specifier = actual.actual.clone();
        if actual_dependency_name == expected_dependency_name
          && actual_actual_specifier == expected_actual_specifier
          && actual_instance_id == expected_instance_id
        {
          continue 'expected;
        }
      }
      println!("{:#?}", expected_matches);
      println!("{:#?}", actual_matches);
      panic!(
        "expected a '{label}' for '{expected_instance_id}' with '{expected_actual_specifier}'"
      );
    }
    self
  }

  fn expect_instance_mismatches(
    &self,
    label: &str,
    expected_mismatches: &Vec<ExpectedMismatchEvent>,
    actual_mismatches: &Vec<ActualMismatchEvent>,
  ) -> &Self {
    if expected_mismatches.len() != actual_mismatches.len() {
      self.debug();
      panic!(
        "expected {} {} mismatches but found {}",
        expected_mismatches.len(),
        label,
        actual_mismatches.len()
      );
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
      println!("{:#?}", expected_mismatches);
      println!("{:#?}", actual_mismatches);
      panic!("expected a '{label}' for '{expected_instance_id}' from '{expected_actual_specifier}' to '{expected_expected_specifier}'");
    }
    self
  }
}
