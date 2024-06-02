use crate::{
  dependency::InstanceIdsBySpecifier,
  effects::{Effects, Event},
};

/// A mock implementation of a command's side effects for the purpose of testing
#[derive(Debug)]
pub struct MockEffects {
  pub events: EventsByType,
}

impl MockEffects {
  pub fn new() -> Self {
    Self {
      events: EventsByType::new(),
    }
  }
}

impl Effects for MockEffects {
  fn on(&mut self, event: Event) -> () {
    match event {
      Event::PackagesLoaded(config, packages) => {
        self.events.packages_loaded.push(());
      }

      Event::EnterVersionsAndRanges(config) => {
        self.events.enter_versions_and_ranges.push(());
      }
      Event::EnterFormat(config) => {
        self.events.enter_format.push(());
      }
      Event::ExitCommand => {
        self.events.exit_command.push(());
      }

      Event::PackagesMatchFormatting(valid_packages, config) => {
        self.events.packages_match_formatting.push(());
      }
      Event::PackagesMismatchFormatting(invalid_packages, config) => {
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
      Event::DependencyMatchesPinnedVersion(dependency) => {
        self.events.dependency_matches_pinned_version.push(());
      }
      Event::DependencyMismatchesPinnedVersion(dependency) => {
        self.events.dependency_mismatches_pinned_version.push(());
      }
      Event::DependencyMatchesRange(dependency) => {
        self.events.dependency_matches_range.push(());
      }
      Event::DependencyMismatchesRange(dependency) => {
        self.events.dependency_mismatches_range.push(());
      }
      Event::DependencyMatchesSnapTo(dependency) => {
        self.events.dependency_matches_snap_to.push(());
      }
      Event::DependencyMismatchesSnapTo(dependency) => {
        self.events.dependency_mismatches_snap_to.push(());
      }
      Event::DependencyMatchesStandard(dependency) => {
        self.events.dependency_matches_standard.push(());
      }
      Event::DependencyMismatchesStandard(dependency) => {
        self.events.dependency_mismatches_standard.push(());
      }

      Event::InstanceBanned(_) => {
        self.events.instance_banned.push(());
      }
      Event::InstanceMismatchesPinnedVersion(_) => {
        self.events.instance_mismatches_pinned_version.push(());
      }
      Event::InstanceMismatchesRange(_) => {
        self.events.instance_mismatches_range.push(());
      }
      Event::InstanceMismatchesSnapTo(_) => {
        self.events.instance_mismatches_snap_to.push(());
      }
      Event::InstanceMismatchesLocalVersion(_) => {
        self.events.instance_mismatches_local_version.push(());
      }
      Event::InstanceUnsupportedMismatch(_) => {
        self.events.instance_unsupported_mismatch.push(());
      }
      Event::InstanceMismatchesLowestVersion(_) => {
        self.events.instance_mismatches_lowest_version.push(());
      }
      Event::InstanceMismatchesHighestVersion(instance_event) => {
        self
          .events
          .instance_mismatches_highest_version
          .push(InstanceEventCopy {
            dependency_name: instance_event.dependency.name.clone(),
            mismatches_with: instance_event.mismatches_with.clone(),
            target: instance_event.target.clone(),
          });
      }
    };
  }
}

#[derive(Debug)]
pub struct InstanceEventCopy {
  pub dependency_name: String,
  pub mismatches_with: InstanceIdsBySpecifier,
  pub target: InstanceIdsBySpecifier,
}

// We'll store data later but for now use `Vec<()>` to keep a count of events
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
  pub dependency_matches_pinned_version: Vec<()>,
  pub dependency_mismatches_pinned_version: Vec<()>,
  pub dependency_matches_range: Vec<()>,
  pub dependency_mismatches_range: Vec<()>,
  pub dependency_matches_snap_to: Vec<()>,
  pub dependency_mismatches_snap_to: Vec<()>,
  pub dependency_matches_standard: Vec<()>,
  pub dependency_mismatches_standard: Vec<()>,
  pub instance_banned: Vec<()>,
  pub instance_mismatches_pinned_version: Vec<()>,
  pub instance_mismatches_range: Vec<()>,
  pub instance_mismatches_snap_to: Vec<()>,
  pub instance_mismatches_local_version: Vec<()>,
  pub instance_unsupported_mismatch: Vec<()>,
  pub instance_mismatches_lowest_version: Vec<()>,
  pub instance_mismatches_highest_version: Vec<InstanceEventCopy>,
}

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
      dependency_matches_pinned_version: vec![],
      dependency_mismatches_pinned_version: vec![],
      dependency_matches_range: vec![],
      dependency_mismatches_range: vec![],
      dependency_matches_snap_to: vec![],
      dependency_mismatches_snap_to: vec![],
      dependency_matches_standard: vec![],
      dependency_mismatches_standard: vec![],
      instance_banned: vec![],
      instance_mismatches_pinned_version: vec![],
      instance_mismatches_range: vec![],
      instance_mismatches_snap_to: vec![],
      instance_mismatches_local_version: vec![],
      instance_unsupported_mismatch: vec![],
      instance_mismatches_lowest_version: vec![],
      instance_mismatches_highest_version: vec![],
    }
  }
}
