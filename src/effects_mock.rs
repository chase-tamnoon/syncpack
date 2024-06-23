#[cfg(test)]
use crate::{
  config::Config,
  context::InstancesById,
  effects::{Effects, Event, InstanceEvent, InstanceEventVariant},
  expect::{ActualMatchEvent, ActualMismatchEvent},
  packages::Packages,
};

// We'll store data later but for now use `Vec<()>` to keep a count of events
#[cfg(test)]
#[derive(Debug)]
pub struct EventsByType {
  pub enter_versions_and_ranges: Vec<()>,
  pub enter_format: Vec<()>,
  pub group_visited: Vec<()>,
  pub dependency_valid: Vec<()>,
  pub dependency_invalid: Vec<()>,
  pub dependency_warning: Vec<()>,
  pub format_match: Vec<()>,
  pub format_mismatch: Vec<()>,
  pub exit_command: Vec<()>,
  /* Ignored */
  pub instance_is_ignored: Vec<ActualMatchEvent>,
  /* Matches */
  pub local_instance_is_preferred: Vec<ActualMatchEvent>,
  pub instance_matches_local: Vec<ActualMatchEvent>,
  pub instance_matches_highest_or_lowest_semver: Vec<ActualMatchEvent>,
  pub instance_matches_but_is_unsupported: Vec<ActualMatchEvent>,
  pub instance_matches_pinned: Vec<ActualMatchEvent>,
  pub instance_matches_same_range_group: Vec<ActualMatchEvent>,
  /* Warnings */
  pub local_instance_mistakenly_banned: Vec<ActualMatchEvent>,
  pub local_instance_mistakenly_mismatches_semver_group: Vec<ActualMismatchEvent>,
  pub local_instance_mistakenly_mismatches_pinned: Vec<ActualMismatchEvent>,
  /* Fixable Mismatches */
  pub instance_is_banned: Vec<ActualMismatchEvent>,
  pub instance_matches_highest_or_lowest_semver_but_mismatches_semver_group:
    Vec<ActualMismatchEvent>,
  pub instance_matches_local_but_mismatches_semver_group: Vec<ActualMismatchEvent>,
  pub instance_mismatches_local: Vec<ActualMismatchEvent>,
  pub instance_mismatches_highest_or_lowest_semver: Vec<ActualMismatchEvent>,
  pub instance_mismatches_pinned: Vec<ActualMismatchEvent>,
  /* Unfixable Mismatches */
  pub instance_mismatches_and_is_unsupported: Vec<ActualMatchEvent>,
  pub instance_matches_pinned_but_mismatches_semver_group: Vec<ActualMatchEvent>,
  pub instance_mismatches_both_same_range_and_conflicting_semver_groups: Vec<ActualMatchEvent>,
  pub instance_mismatches_both_same_range_and_compatible_semver_groups: Vec<ActualMatchEvent>,
  pub instance_matches_same_range_group_but_mismatches_conflicting_semver_group:
    Vec<ActualMatchEvent>,
  pub instance_matches_same_range_group_but_mismatches_compatible_semver_group:
    Vec<ActualMatchEvent>,
  pub instance_mismatches_same_range_group: Vec<ActualMatchEvent>,
}

#[cfg(test)]
impl EventsByType {
  pub fn new() -> Self {
    Self {
      enter_versions_and_ranges: vec![],
      enter_format: vec![],
      group_visited: vec![],
      dependency_valid: vec![],
      dependency_invalid: vec![],
      dependency_warning: vec![],
      format_match: vec![],
      format_mismatch: vec![],
      exit_command: vec![],
      /* Ignored */
      instance_is_ignored: vec![],
      /* Matches */
      local_instance_is_preferred: vec![],
      instance_matches_local: vec![],
      instance_matches_highest_or_lowest_semver: vec![],
      instance_matches_but_is_unsupported: vec![],
      instance_matches_pinned: vec![],
      instance_matches_same_range_group: vec![],
      /* Warnings */
      local_instance_mistakenly_banned: vec![],
      local_instance_mistakenly_mismatches_semver_group: vec![],
      local_instance_mistakenly_mismatches_pinned: vec![],
      /* Fixable Mismatches */
      instance_is_banned: vec![],
      instance_matches_highest_or_lowest_semver_but_mismatches_semver_group: vec![],
      instance_matches_local_but_mismatches_semver_group: vec![],
      instance_mismatches_local: vec![],
      instance_mismatches_highest_or_lowest_semver: vec![],
      instance_mismatches_pinned: vec![],
      /* Unfixable Mismatches */
      instance_mismatches_and_is_unsupported: vec![],
      instance_matches_pinned_but_mismatches_semver_group: vec![],
      instance_mismatches_both_same_range_and_conflicting_semver_groups: vec![],
      instance_mismatches_both_same_range_and_compatible_semver_groups: vec![],
      instance_matches_same_range_group_but_mismatches_conflicting_semver_group: vec![],
      instance_matches_same_range_group_but_mismatches_compatible_semver_group: vec![],
      instance_mismatches_same_range_group: vec![],
    }
  }
}

/// A mock implementation of a command's side effects for the purpose of testing
#[cfg(test)]
#[derive(Debug)]
pub struct MockEffects<'a> {
  pub config: &'a Config,
  pub events: EventsByType,
  pub is_valid: bool,
  pub packages: Option<Packages>,
}

#[cfg(test)]
impl<'a> MockEffects<'a> {
  pub fn new(config: &'a Config) -> Self {
    Self {
      config,
      events: EventsByType::new(),
      is_valid: true,
      packages: None,
    }
  }
}

#[cfg(test)]
impl Effects for MockEffects<'_> {
  fn get_packages(&mut self) -> Packages {
    let packages = self.packages.take().unwrap();
    self.packages = None;
    packages
  }

  fn set_packages(&mut self, packages: Packages) -> () {
    self.packages = Some(packages);
  }

  fn on(&mut self, event: Event, instances_by_id: &mut InstancesById) -> () {
    match &event {
      Event::EnterVersionsAndRanges => self.events.enter_versions_and_ranges.push(()),
      Event::EnterFormat => self.events.enter_format.push(()),
      Event::GroupVisited(_) => self.events.group_visited.push(()),
      Event::DependencyValid(_, _) => self.events.dependency_valid.push(()),
      Event::DependencyInvalid(_, _) => self.events.dependency_invalid.push(()),
      Event::DependencyWarning(_, _) => self.events.dependency_warning.push(()),
      Event::FormatMatch(_) => self.events.format_match.push(()),
      Event::FormatMismatch(_) => self.events.format_mismatch.push(()),
      Event::ExitCommand => self.events.exit_command.push(()),
    };
  }

  fn on_instance(&mut self, event: InstanceEvent, instances_by_id: &mut InstancesById) -> () {
    let instance_id = &event.instance_id;
    let dependency = &event.dependency;
    let instance = instances_by_id.get(instance_id).unwrap();
    match &event.variant {
      /* Ignored */
      InstanceEventVariant::InstanceIsIgnored => {
        self.events.instance_is_ignored.push(ActualMatchEvent {
          dependency_name: event.dependency.name.clone(),
          instance_id: event.instance_id.clone(),
          actual: instance.actual.unwrap().clone(),
        });
      }
      /* Matches */
      InstanceEventVariant::LocalInstanceIsPreferred => {
        self
          .events
          .local_instance_is_preferred
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMatchesLocal => {
        self.events.instance_matches_local.push(ActualMatchEvent {
          dependency_name: event.dependency.name.clone(),
          instance_id: event.instance_id.clone(),
          actual: instance.actual.unwrap().clone(),
        });
      }
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemver => {
        self
          .events
          .instance_matches_highest_or_lowest_semver
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMatchesButIsUnsupported => {
        self
          .events
          .instance_matches_but_is_unsupported
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMatchesPinned => {
        self.events.instance_matches_pinned.push(ActualMatchEvent {
          dependency_name: event.dependency.name.clone(),
          instance_id: event.instance_id.clone(),
          actual: instance.actual.unwrap().clone(),
        });
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroup => {
        self
          .events
          .instance_matches_same_range_group
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      /* Warnings */
      InstanceEventVariant::LocalInstanceMistakenlyBanned => {
        self
          .events
          .local_instance_mistakenly_banned
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup => {
        self
          .events
          .local_instance_mistakenly_mismatches_semver_group
          .push(ActualMismatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
            expected: instance.expected.unwrap().clone(),
          });
      }
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned => {
        self
          .events
          .local_instance_mistakenly_mismatches_pinned
          .push(ActualMismatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
            expected: instance.expected.unwrap().clone(),
          });
      }
      /* Fixable Mismatches */
      InstanceEventVariant::InstanceIsBanned => {
        self.events.instance_is_banned.push(ActualMismatchEvent {
          dependency_name: event.dependency.name.clone(),
          instance_id: event.instance_id.clone(),
          actual: instance.actual.unwrap().clone(),
          expected: instance.expected.unwrap().clone(),
        });
      }
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesSemverGroup => {
        self
          .events
          .instance_matches_highest_or_lowest_semver_but_mismatches_semver_group
          .push(ActualMismatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
            expected: instance.expected.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup => {
        self
          .events
          .instance_matches_local_but_mismatches_semver_group
          .push(ActualMismatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
            expected: instance.expected.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMismatchesLocal => {
        self
          .events
          .instance_mismatches_local
          .push(ActualMismatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
            expected: instance.expected.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver => {
        self
          .events
          .instance_mismatches_highest_or_lowest_semver
          .push(ActualMismatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
            expected: instance.expected.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMismatchesPinned => {
        self
          .events
          .instance_mismatches_pinned
          .push(ActualMismatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
            expected: instance.expected.unwrap().clone(),
          });
      }
      /* Unfixable Mismatches */
      InstanceEventVariant::InstanceMismatchesAndIsUnsupported => {
        self
          .events
          .instance_mismatches_and_is_unsupported
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup => {
        self
          .events
          .instance_matches_pinned_but_mismatches_semver_group
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups => {
        self
          .events
          .instance_mismatches_both_same_range_and_conflicting_semver_groups
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups => {
        self
          .events
          .instance_mismatches_both_same_range_and_compatible_semver_groups
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup => {
        self
          .events
          .instance_matches_same_range_group_but_mismatches_conflicting_semver_group
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup => {
        self
          .events
          .instance_matches_same_range_group_but_mismatches_compatible_semver_group
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
      InstanceEventVariant::InstanceMismatchesSameRangeGroup => {
        self
          .events
          .instance_mismatches_same_range_group
          .push(ActualMatchEvent {
            dependency_name: event.dependency.name.clone(),
            instance_id: event.instance_id.clone(),
            actual: instance.actual.unwrap().clone(),
          });
      }
    }
  }
}
