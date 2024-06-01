use crate::effects::{Effects, Event};

/// A mock implementation of a command's side effects for the purpose of testing
pub struct MockEffects {
  pub packages_loaded: i32,
  pub enter_versions_and_ranges: i32,
  pub enter_format: i32,
  pub exit_command: i32,
  pub packages_match_formatting: i32,
  pub packages_mismatch_formatting: i32,
  pub group_visited: i32,
  pub dependency_ignored: i32,
  pub dependency_banned: i32,
  pub dependency_matches_pinned_version: i32,
  pub dependency_mismatches_pinned_version: i32,
  pub dependency_matches_range: i32,
  pub dependency_mismatches_range: i32,
  pub dependency_matches_snap_to: i32,
  pub dependency_mismatches_snap_to: i32,
  pub dependency_matches_standard: i32,
  pub dependency_mismatches_standard: i32,
  pub instance_banned: i32,
  pub instance_mismatches_pinned_version: i32,
  pub instance_mismatches_range: i32,
  pub instance_mismatches_snap_to: i32,
  pub instance_mismatches_local_version: i32,
  pub instance_unsupported_mismatch: i32,
  pub instance_mismatches_lowest_version: i32,
  pub instance_mismatches_highest_version: i32,
}

impl MockEffects {
  pub fn new() -> Self {
    Self {
      packages_loaded: 0,
      enter_versions_and_ranges: 0,
      enter_format: 0,
      exit_command: 0,
      packages_match_formatting: 0,
      packages_mismatch_formatting: 0,
      group_visited: 0,
      dependency_ignored: 0,
      dependency_banned: 0,
      dependency_matches_pinned_version: 0,
      dependency_mismatches_pinned_version: 0,
      dependency_matches_range: 0,
      dependency_mismatches_range: 0,
      dependency_matches_snap_to: 0,
      dependency_mismatches_snap_to: 0,
      dependency_matches_standard: 0,
      dependency_mismatches_standard: 0,
      instance_banned: 0,
      instance_mismatches_pinned_version: 0,
      instance_mismatches_range: 0,
      instance_mismatches_snap_to: 0,
      instance_mismatches_local_version: 0,
      instance_unsupported_mismatch: 0,
      instance_mismatches_lowest_version: 0,
      instance_mismatches_highest_version: 0,
    }
  }
}

impl Effects for MockEffects {
  fn on_event(&mut self, event: Event) -> () {
    match event {
      Event::PackagesLoaded(config, packages) => {
        self.packages_loaded += 1;
      }

      Event::EnterVersionsAndRanges(config) => {
        self.enter_versions_and_ranges += 1;
      }
      Event::EnterFormat(config) => {
        self.enter_format += 1;
      }
      Event::ExitCommand => {
        self.exit_command += 1;
      }

      Event::PackagesMatchFormatting(valid_packages, config) => {
        self.packages_match_formatting += 1;
      }
      Event::PackagesMismatchFormatting(invalid_packages, config) => {
        self.packages_mismatch_formatting += 1;
      }

      Event::GroupVisited(selector) => {
        self.group_visited += 1;
      }

      Event::DependencyIgnored(dependency) => {
        self.dependency_ignored += 1;
      }
      Event::DependencyBanned(dependency) => {
        self.dependency_banned += 1;
      }
      Event::DependencyMatchesPinnedVersion(dependency) => {
        self.dependency_matches_pinned_version += 1;
      }
      Event::DependencyMismatchesPinnedVersion(dependency) => {
        self.dependency_mismatches_pinned_version += 1;
      }
      Event::DependencyMatchesRange(dependency) => {
        self.dependency_matches_range += 1;
      }
      Event::DependencyMismatchesRange(dependency) => {
        self.dependency_mismatches_range += 1;
      }
      Event::DependencyMatchesSnapTo(dependency) => {
        self.dependency_matches_snap_to += 1;
      }
      Event::DependencyMismatchesSnapTo(dependency) => {
        self.dependency_mismatches_snap_to += 1;
      }
      Event::DependencyMatchesStandard(dependency) => {
        self.dependency_matches_standard += 1;
      }
      Event::DependencyMismatchesStandard(dependency) => {
        self.dependency_mismatches_standard += 1;
      }

      Event::InstanceBanned(event) => {
        self.instance_banned += 1;
      }
      Event::InstanceMismatchesPinnedVersion(event) => {
        self.instance_mismatches_pinned_version += 1;
      }
      Event::InstanceMismatchesRange(event) => {
        self.instance_mismatches_range += 1;
      }
      Event::InstanceMismatchesSnapTo(event) => {
        self.instance_mismatches_snap_to += 1;
      }
      Event::InstanceMismatchesLocalVersion(event) => {
        self.instance_mismatches_local_version += 1;
      }
      Event::InstanceUnsupportedMismatch(event) => {
        self.instance_unsupported_mismatch += 1;
      }
      Event::InstanceMismatchesLowestVersion(event) => {
        self.instance_mismatches_lowest_version += 1;
      }
      Event::InstanceMismatchesHighestVersion(event) => {
        self.instance_mismatches_highest_version += 1;
      }
    };
  }
}
