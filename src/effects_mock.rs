#[cfg(test)]
use crate::{
  effects::{Effects, Event},
  instance::InstanceId,
  specifier::Specifier,
};

#[cfg(test)]
#[derive(Debug)]
pub struct PartialMatchEvent {
  /// the instance that did match
  pub instance_id: InstanceId,
  /// eg. "react"
  pub dependency_name: String,
  /// the version specifier the instance has
  pub specifier: Specifier,
}

#[cfg(test)]
#[derive(Debug)]
pub struct PartialMismatchEvent {
  /// the instance that was expected to match
  pub instance_id: InstanceId,
  /// eg. "react"
  pub dependency_name: String,
  /// the correct version specifier the instance should have had
  pub expected_specifier: Specifier,
  /// the incorrect version specifier the instance actually has
  pub actual_specifier: Specifier,
  /// other instances which do have the correct version specifier
  pub matching_instance_ids: Vec<InstanceId>,
}

#[cfg(test)]
#[derive(Debug)]
pub struct PartialUnsupportedMismatchEvent {
  /// the instance that should be banned
  pub instance_id: InstanceId,
  /// eg. "react"
  pub dependency_name: String,
  /// the incorrect version specifier the instance actually has
  pub specifier: Specifier,
}

#[cfg(test)]
#[derive(Debug)]
pub struct PartialBannedEvent {
  /// the instance that should be banned
  pub instance_id: InstanceId,
  /// eg. "react"
  pub dependency_name: String,
}

#[cfg(test)]
#[derive(Debug)]
pub struct PartialSameRangeMismatchEvent {
  /// the instance that was expected to match
  pub instance_id: InstanceId,
  /// eg. "react"
  pub dependency_name: String,
  /// the range specifier which does not match every other range
  pub specifier: Specifier,
  pub specifier_outside_range: Specifier,
  pub instance_ids_outside_range: Vec<InstanceId>,
}

#[cfg(test)]
#[derive(Debug)]
pub struct PartialSnapToMismatchEvent {
  /// the unique identifier for the instance which was found
  pub instance_id: InstanceId,
  /// all instances of this dependency (eg. "react") in this version group
  pub dependency_name: String,
  /// the correct version specifier the instance should have had
  pub expected_specifier: Specifier,
  /// the incorrect version specifier the instance actually has
  pub actual_specifier: Specifier,
  /// the instance with the version specifier to be snapped to
  pub snap_to_instance_id: InstanceId,
}

// We'll store data later but for now use `Vec<()>` to keep a count of events
#[cfg(test)]
#[derive(Debug)]
pub struct EventsByType {
  pub packages_loaded: Vec<()>,
  pub enter_versions_and_ranges: Vec<()>,
  pub enter_format: Vec<()>,
  pub exit_command: Vec<()>,

  pub packages_match_formatting: Vec<()>,
  pub packages_mismatch_formatting: Vec<()>,

  pub group_visited: Vec<()>,

  pub dependency_ignored: Vec<()>,
  pub dependency_banned: Vec<()>,
  pub dependency_matches_with_range: Vec<String>,
  pub dependency_mismatches_with_range: Vec<String>,
  pub dependency_matches_pinned_version: Vec<String>,
  pub dependency_mismatches_pinned_version: Vec<()>,
  pub dependency_matches_range: Vec<String>,
  pub dependency_mismatches_range: Vec<()>,
  pub dependency_matches_snap_to: Vec<String>,
  pub dependency_mismatches_snap_to: Vec<()>,
  pub dependency_matches_standard: Vec<String>,
  pub dependency_mismatches_standard: Vec<()>,

  pub instance_matches_standard: Vec<PartialMatchEvent>,
  pub instance_banned: Vec<PartialBannedEvent>,
  pub instance_matches_semver_range: Vec<PartialMatchEvent>,
  pub instance_mismatches_semver_range: Vec<PartialMismatchEvent>,
  pub instance_mismatches_pinned_version: Vec<PartialMismatchEvent>,
  pub instance_mismatches_range: Vec<PartialSameRangeMismatchEvent>,
  pub instance_mismatches_snap_to: Vec<PartialSnapToMismatchEvent>,
  pub instance_mismatch_changes_local_version: Vec<PartialMismatchEvent>,
  pub instance_mismatches_local_version: Vec<PartialMismatchEvent>,
  pub instance_unsupported_mismatch: Vec<PartialUnsupportedMismatchEvent>,
  pub instance_mismatches_lowest_version: Vec<PartialMismatchEvent>,
  pub instance_mismatches_highest_version: Vec<PartialMismatchEvent>,
}

#[cfg(test)]
impl EventsByType {
  pub fn new() -> Self {
    Self {
      packages_loaded: vec![],
      enter_versions_and_ranges: vec![],
      enter_format: vec![],
      exit_command: vec![],

      packages_match_formatting: vec![],
      packages_mismatch_formatting: vec![],

      group_visited: vec![],

      dependency_ignored: vec![],
      dependency_banned: vec![],
      dependency_matches_with_range: vec![],
      dependency_mismatches_with_range: vec![],
      dependency_matches_pinned_version: vec![],
      dependency_mismatches_pinned_version: vec![],
      dependency_matches_range: vec![],
      dependency_mismatches_range: vec![],
      dependency_matches_snap_to: vec![],
      dependency_mismatches_snap_to: vec![],
      dependency_matches_standard: vec![],
      dependency_mismatches_standard: vec![],

      instance_matches_standard: vec![],
      instance_banned: vec![],
      instance_matches_semver_range: vec![],
      instance_mismatches_semver_range: vec![],
      instance_mismatches_pinned_version: vec![],
      instance_mismatches_range: vec![],
      instance_mismatches_snap_to: vec![],
      instance_mismatch_changes_local_version: vec![],
      instance_mismatches_local_version: vec![],
      instance_unsupported_mismatch: vec![],
      instance_mismatches_lowest_version: vec![],
      instance_mismatches_highest_version: vec![],
    }
  }
}

/// A mock implementation of a command's side effects for the purpose of testing
#[cfg(test)]
#[derive(Debug)]
pub struct MockEffects {
  pub events: EventsByType,
}

#[cfg(test)]
impl MockEffects {
  pub fn new() -> Self {
    Self {
      events: EventsByType::new(),
    }
  }
}

#[cfg(test)]
impl Effects for MockEffects {
  fn on(&mut self, event: Event) -> () {
    match event {
      Event::PackagesLoaded(packages) => {
        self.events.packages_loaded.push(());
      }

      Event::EnterVersionsAndRanges => {
        self.events.enter_versions_and_ranges.push(());
      }
      Event::EnterFormat => {
        self.events.enter_format.push(());
      }
      Event::ExitCommand => {
        self.events.exit_command.push(());
      }

      Event::PackagesMatchFormatting(valid_packages) => {
        self.events.packages_match_formatting.push(());
      }
      Event::PackagesMismatchFormatting(invalid_packages) => {
        self.events.packages_mismatch_formatting.push(());
      }

      Event::GroupVisited(selector) => {
        self.events.group_visited.push(());
      }

      Event::DependencyIgnored(dependency) => {
        self.events.dependency_ignored.push(());
      }
      Event::DependencyBanned(dependency) => {
        self.events.dependency_banned.push(());
      }
      Event::DependencyMatchesWithRange(dependency) => {
        self
          .events
          .dependency_matches_with_range
          .push(dependency.name.clone());
      }
      Event::DependencyMismatchesWithRange(dependency) => {
        self
          .events
          .dependency_mismatches_with_range
          .push(dependency.name.clone());
      }
      Event::DependencyMatchesPinnedVersion(dependency) => {
        self
          .events
          .dependency_matches_pinned_version
          .push(dependency.name.clone());
      }
      Event::DependencyMismatchesPinnedVersion(dependency) => {
        self.events.dependency_mismatches_pinned_version.push(());
      }
      Event::DependencyMatchesSameRange(dependency) => {
        self
          .events
          .dependency_matches_range
          .push(dependency.name.clone());
      }
      Event::DependencyMismatchesSameRange(dependency) => {
        self.events.dependency_mismatches_range.push(());
      }
      Event::DependencyMatchesSnapTo(dependency) => {
        self
          .events
          .dependency_matches_snap_to
          .push(dependency.name.clone());
      }
      Event::DependencyMismatchesSnapTo(dependency) => {
        self.events.dependency_mismatches_snap_to.push(());
      }
      Event::DependencyMatchesStandard(dependency) => {
        self
          .events
          .dependency_matches_standard
          .push(dependency.name.clone());
      }
      Event::DependencyMismatchesStandard(dependency) => {
        self.events.dependency_mismatches_standard.push(());
      }

      Event::InstanceMatchesStandard(event) => {
        self
          .events
          .instance_matches_standard
          .push(PartialMatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            specifier: event.specifier.clone(),
          });
      }
      Event::InstanceBanned(event) => {
        self.events.instance_banned.push(PartialBannedEvent {
          instance_id: event.instance_id.clone(),
          dependency_name: event.dependency.name.clone(),
        });
      }
      Event::InstanceMatchesWithRange(event) => {
        self
          .events
          .instance_matches_semver_range
          .push(PartialMatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            specifier: event.specifier.clone(),
          });
      }
      Event::InstanceMismatchesWithRange(event) => {
        self
          .events
          .instance_mismatches_semver_range
          .push(PartialMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            expected_specifier: event.expected_specifier.clone(),
            matching_instance_ids: event.matching_instance_ids.clone(),
            actual_specifier: event.actual_specifier.clone(),
          });
      }
      Event::InstanceMismatchesPinnedVersion(event) => {
        self
          .events
          .instance_mismatches_pinned_version
          .push(PartialMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            expected_specifier: event.expected_specifier.clone(),
            matching_instance_ids: event.matching_instance_ids.clone(),
            actual_specifier: event.actual_specifier.clone(),
          });
      }
      Event::InstanceMismatchesSameRange(event) => {
        self
          .events
          .instance_mismatches_range
          .push(PartialSameRangeMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            specifier: event.specifier.clone(),
            specifier_outside_range: event.specifier_outside_range.clone(),
            instance_ids_outside_range: event.instance_ids_outside_range.clone(),
          });
      }
      Event::InstanceMismatchesSnapTo(event) => {
        self
          .events
          .instance_mismatches_snap_to
          .push(PartialSnapToMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            expected_specifier: event.expected_specifier.clone(),
            actual_specifier: event.actual_specifier.clone(),
            snap_to_instance_id: event.snap_to_instance_id.clone(),
          });
      }
      Event::InstanceMismatchCorruptsLocalVersion(event) => {
        self
          .events
          .instance_mismatch_changes_local_version
          .push(PartialMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            expected_specifier: event.expected_specifier.clone(),
            matching_instance_ids: event.matching_instance_ids.clone(),
            actual_specifier: event.actual_specifier.clone(),
          });
      }
      Event::InstanceMismatchesLocalVersion(event) => {
        self
          .events
          .instance_mismatches_local_version
          .push(PartialMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            expected_specifier: event.expected_specifier.clone(),
            matching_instance_ids: event.matching_instance_ids.clone(),
            actual_specifier: event.actual_specifier.clone(),
          });
      }
      Event::InstanceUnsupportedMismatch(event) => {
        self
          .events
          .instance_unsupported_mismatch
          .push(PartialUnsupportedMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            specifier: event.specifier.clone(),
          });
      }
      Event::InstanceMismatchesLowestVersion(event) => {
        self
          .events
          .instance_mismatches_lowest_version
          .push(PartialMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            expected_specifier: event.expected_specifier.clone(),
            matching_instance_ids: event.matching_instance_ids.clone(),
            actual_specifier: event.actual_specifier.clone(),
          });
      }
      Event::InstanceMismatchesHighestVersion(event) => {
        self
          .events
          .instance_mismatches_highest_version
          .push(PartialMismatchEvent {
            instance_id: event.instance_id.clone(),
            dependency_name: event.dependency.name.clone(),
            expected_specifier: event.expected_specifier.clone(),
            matching_instance_ids: event.matching_instance_ids.clone(),
            actual_specifier: event.actual_specifier.clone(),
          });
      }
    };
  }
}
