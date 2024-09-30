use std::collections::HashMap;

use crate::{
  config::Config,
  context::InstancesById,
  effects::{Effects, Event, InstanceEvent, InstanceEventVariant},
  packages::Packages,
  test::expect::{ActualMatchEvent, ActualMismatchEvent},
};

// We'll store data later but for now use `Vec<()>` to keep a count of events
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
  pub package_format_match: Vec<()>,
  pub package_format_mismatch: Vec<()>,
  pub exit_command: Vec<()>,
}

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
      package_format_match: vec![],
      package_format_mismatch: vec![],
      exit_command: vec![],
    }
  }
}

/// A mock implementation of a command's side effects for the purpose of testing
#[derive(Debug)]
pub struct MockEffects<'a> {
  pub config: &'a Config,
  pub events: EventsByType,
  pub is_valid: bool,
  pub matches: HashMap<InstanceEventVariant, Vec<ActualMatchEvent>>,
  pub mismatches: HashMap<InstanceEventVariant, Vec<ActualMismatchEvent>>,
  pub packages: Option<Packages>,
}

impl Effects for MockEffects<'_> {
  fn on(&mut self, event: Event) {
    match &event {
      Event::EnterVersionsAndRanges => self.events.enter_versions_and_ranges.push(()),
      Event::EnterFormat => self.events.enter_format.push(()),
      Event::GroupVisited(_) => self.events.group_visited.push(()),
      Event::DependencyValid(_, _) => self.events.dependency_valid.push(()),
      Event::DependencyInvalid(_, _) => self.events.dependency_invalid.push(()),
      Event::DependencyWarning(_, _) => self.events.dependency_warning.push(()),
      Event::PackageFormatMatch(_) => self.events.package_format_match.push(()),
      Event::PackageFormatMismatch(_) => self.events.package_format_mismatch.push(()),
      Event::ExitCommand => self.events.exit_command.push(()),
    };
  }

  fn on_instance(&mut self, event: InstanceEvent, instances_by_id: &mut InstancesById) {
    let instance_id = &event.instance_id;
    let dependency = &event.dependency;
    let instance = instances_by_id.get(instance_id).unwrap();

    let mut record_match_event = || {
      self
        .matches
        .entry(event.variant.clone())
        .or_default()
        .push(ActualMatchEvent::new(&event, instance))
    };

    let mut record_mismatch_event = || {
      self
        .mismatches
        .entry(event.variant.clone())
        .or_default()
        .push(ActualMismatchEvent::new(&event, instance));
    };

    match &event.variant {
      InstanceEventVariant::InstanceIsIgnored => record_match_event(),
      InstanceEventVariant::LocalInstanceIsPreferred => record_match_event(),
      InstanceEventVariant::InstanceMatchesLocal => record_match_event(),
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemver => record_match_event(),
      InstanceEventVariant::InstanceMatchesButIsUnsupported => record_match_event(),
      InstanceEventVariant::InstanceMatchesPinned => record_match_event(),
      InstanceEventVariant::InstanceMatchesSameRangeGroup => record_match_event(),
      InstanceEventVariant::LocalInstanceMistakenlyBanned => record_match_event(),

      InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup => record_mismatch_event(),
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesAndIsUnsupported => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesLocalWithMissingVersion => record_mismatch_event(),
      InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups => record_mismatch_event(),
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup => record_mismatch_event(),
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesSameRangeGroup => record_mismatch_event(),
      InstanceEventVariant::InstanceIsBanned => record_mismatch_event(),
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup => {
        record_mismatch_event()
      }
      InstanceEventVariant::InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed => record_mismatch_event(),
      InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesLocal => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver => record_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesPinned => record_mismatch_event(),
    };
  }
}
