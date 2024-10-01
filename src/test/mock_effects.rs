use std::collections::HashMap;

use crate::{
  config::Config,
  effects::{Effects, Event, InstanceEvent, InstanceEventVariant},
  packages::Packages,
  test::expect::{ActualFixableMismatchEvent, ActualMatchEvent},
};

use super::expect::ActualUnfixableMismatchEvent;

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
  pub unfixable_mismatches: HashMap<InstanceEventVariant, Vec<ActualUnfixableMismatchEvent>>,
  pub fixable_mismatches: HashMap<InstanceEventVariant, Vec<ActualFixableMismatchEvent>>,
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

  fn on_instance(&mut self, event: InstanceEvent) {
    let instance = &event.instance;
    let dependency = &event.dependency;

    let mut record_match_event = || {
      self
        .matches
        .entry(event.variant.clone())
        .or_default()
        .push(ActualMatchEvent::new(&event, instance))
    };

    let mut record_fixable_mismatch_event = || {
      self
        .fixable_mismatches
        .entry(event.variant.clone())
        .or_default()
        .push(ActualFixableMismatchEvent::new(&event, instance));
    };

    let mut record_unfixable_mismatch_event = || {
      self
        .unfixable_mismatches
        .entry(event.variant.clone())
        .or_default()
        .push(ActualUnfixableMismatchEvent::new(&event, instance));
    };

    match &event.variant {
      InstanceEventVariant::InstanceIsIgnored => record_match_event(),
      InstanceEventVariant::LocalInstanceIsValid => record_match_event(),
      InstanceEventVariant::InstanceMatchesLocal => record_match_event(),
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemver => record_match_event(),
      InstanceEventVariant::InstanceMatchesButIsUnsupported => record_match_event(),
      InstanceEventVariant::InstanceMatchesPinned => record_match_event(),
      InstanceEventVariant::InstanceMatchesSameRangeGroup => record_match_event(),
      InstanceEventVariant::LocalInstanceMistakenlyBanned => record_match_event(),
      InstanceEventVariant::LocalInstanceWithMissingVersion => record_unfixable_mismatch_event(),

      InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup => record_fixable_mismatch_event(),
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned => record_unfixable_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesAndIsUnsupported => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesLocalWithMissingVersion => record_unfixable_mismatch_event(),
      InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups => {
        record_fixable_mismatch_event()
      }
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup => {
        record_fixable_mismatch_event()
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup => {
        record_fixable_mismatch_event()
      }
      InstanceEventVariant::InstanceMismatchesSameRangeGroup => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceIsBanned => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup => {
        record_fixable_mismatch_event()
      }
      InstanceEventVariant::InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesLocal => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver => record_fixable_mismatch_event(),
      InstanceEventVariant::InstanceMismatchesPinned => record_fixable_mismatch_event(),
    };
  }
}
