use std::collections::HashMap;

use crate::{
  config::Config,
  effects::{Effects, Event, InstanceEvent, InstanceState},
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
  pub matches: HashMap<InstanceState, Vec<ActualMatchEvent>>,
  pub unfixable_mismatches: HashMap<InstanceState, Vec<ActualUnfixableMismatchEvent>>,
  pub fixable_mismatches: HashMap<InstanceState, Vec<ActualFixableMismatchEvent>>,
  pub packages: Option<Packages>,
}

impl Effects for MockEffects<'_> {
  fn on(&mut self, event: Event) {
    match &event {
      Event::EnterVersionsAndRanges => self.events.enter_versions_and_ranges.push(()),
      Event::EnterFormat => self.events.enter_format.push(()),
      Event::GroupVisited(_) => self.events.group_visited.push(()),
      Event::DependencyValid(_) => self.events.dependency_valid.push(()),
      Event::DependencyInvalid(_) => self.events.dependency_invalid.push(()),
      Event::DependencyWarning(_) => self.events.dependency_warning.push(()),
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
      InstanceState::Unknown => {
        panic!("InstanceState::Unknown encountered")
      }
      InstanceState::MatchesIgnored => record_match_event(),
      InstanceState::LocalWithValidVersion => record_match_event(),
      InstanceState::MatchesLocal => record_match_event(),
      InstanceState::MatchesPreferVersion => record_match_event(),
      InstanceState::MatchesButUnsupported => record_match_event(),
      InstanceState::MatchesPin => record_match_event(),
      InstanceState::MatchesSameRangeGroup => record_match_event(),
      InstanceState::RefuseToBanLocal => record_unfixable_mismatch_event(),

      InstanceState::MissingLocalVersion => record_unfixable_mismatch_event(),
      InstanceState::RefuseToPinLocal => record_unfixable_mismatch_event(),
      InstanceState::MismatchesMissingLocalVersion => record_unfixable_mismatch_event(),
      InstanceState::PinMatchConflictsWithSemverGroup => record_unfixable_mismatch_event(),
      InstanceState::PreferVersionMatchConflictsWithSemverGroup => record_unfixable_mismatch_event(),
      InstanceState::LocalMatchConflictsWithSemverGroup => record_unfixable_mismatch_event(),

      InstanceState::RefuseToChangeLocalSemverRange => record_fixable_mismatch_event(),
      InstanceState::MismatchesUnsupported => record_fixable_mismatch_event(),
      InstanceState::SemverRangeMismatchWontFixSameRangeGroup => record_fixable_mismatch_event(),
      InstanceState::SemverRangeMismatchWillFixSameRangeGroup => record_fixable_mismatch_event(),
      InstanceState::SameRangeMatchConflictsWithSemverGroup => record_fixable_mismatch_event(),
      InstanceState::SemverRangeMismatchWillMatchSameRangeGroup => record_fixable_mismatch_event(),
      InstanceState::MismatchesSameRangeGroup => record_fixable_mismatch_event(),
      InstanceState::Banned => record_fixable_mismatch_event(),
      InstanceState::SemverRangeMismatchWillFixPreferVersion => record_fixable_mismatch_event(),
      InstanceState::MismatchesLocal => record_fixable_mismatch_event(),
      InstanceState::MismatchesPreferVersion => record_fixable_mismatch_event(),
      InstanceState::MismatchesPin => record_fixable_mismatch_event(),
    };
  }
}
