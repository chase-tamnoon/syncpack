use std::collections::HashMap;

use crate::{
  config::Config,
  effects::{Effects, Event, InstanceEvent, InstanceState},
  packages::Packages,
  test::expect::{ActualFixableMismatchEvent, ActualMatchEvent},
};

use super::expect::{ActualOverrideEvent, ActualUnfixableMismatchEvent};

// We'll store data later but for now use `Vec<()>` to keep a count of events
#[derive(Debug)]
pub struct EventsByType {
  pub dependency_invalid: Vec<()>,
  pub dependency_valid: Vec<()>,
  pub dependency_warning: Vec<()>,
  pub enter_format: Vec<()>,
  pub enter_versions_and_ranges: Vec<()>,
  pub exit_command: Vec<()>,
  pub format_match: Vec<()>,
  pub format_mismatch: Vec<()>,
  pub group_visited: Vec<()>,
  pub package_format_match: Vec<()>,
  pub package_format_mismatch: Vec<()>,
}

impl EventsByType {
  pub fn new() -> Self {
    Self {
      dependency_invalid: vec![],
      dependency_valid: vec![],
      dependency_warning: vec![],
      enter_format: vec![],
      enter_versions_and_ranges: vec![],
      exit_command: vec![],
      format_match: vec![],
      format_mismatch: vec![],
      group_visited: vec![],
      package_format_match: vec![],
      package_format_mismatch: vec![],
    }
  }
}

/// A mock implementation of a command's side effects for the purpose of testing
#[derive(Debug)]
pub struct MockEffects<'a> {
  pub config: &'a Config,
  pub events: EventsByType,
  pub fixable_mismatches: HashMap<InstanceState, Vec<ActualFixableMismatchEvent>>,
  pub is_valid: bool,
  pub matches: HashMap<InstanceState, Vec<ActualMatchEvent>>,
  pub overrides: HashMap<InstanceState, Vec<ActualOverrideEvent>>,
  pub packages: Option<Packages>,
  pub unfixable_mismatches: HashMap<InstanceState, Vec<ActualUnfixableMismatchEvent>>,
  pub warnings_of_instance_changes: HashMap<InstanceState, Vec<ActualFixableMismatchEvent>>,
  pub warnings: HashMap<InstanceState, Vec<ActualUnfixableMismatchEvent>>,
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

    let mut record_warning_event = || {
      self
        .warnings
        .entry(event.variant.clone())
        .or_default()
        .push(ActualUnfixableMismatchEvent::new(&event, instance))
    };

    let mut record_warning_of_instance_change_event = || {
      self
        .warnings_of_instance_changes
        .entry(event.variant.clone())
        .or_default()
        .push(ActualFixableMismatchEvent::new(&event, instance));
    };

    let mut record_fixable_mismatch_event = || {
      self
        .fixable_mismatches
        .entry(event.variant.clone())
        .or_default()
        .push(ActualFixableMismatchEvent::new(&event, instance));
    };

    let mut record_override_event = |overridden: String| {
      self
        .overrides
        .entry(event.variant.clone())
        .or_default()
        .push(ActualOverrideEvent::new(&event, instance, overridden));
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
      /* = Matches ============================================================== */
      InstanceState::Ignored => record_match_event(),
      InstanceState::ValidLocal => record_match_event(),
      InstanceState::EqualsLocal => record_match_event(),
      InstanceState::MatchesLocal => record_match_event(),
      InstanceState::EqualsPreferVersion => record_match_event(),
      InstanceState::EqualsNonSemverPreferVersion => record_match_event(),
      InstanceState::EqualsPin => record_match_event(),
      InstanceState::MatchesSameRangeGroup => record_match_event(),
      /* = Warnings ============================================================= */
      // @FIXME: record these accurately
      InstanceState::RefuseToBanLocal => record_warning_event(),
      InstanceState::RefuseToPinLocal => record_warning_event(),
      InstanceState::InvalidLocalVersion => record_warning_event(),
      InstanceState::MatchesPreferVersion => record_warning_of_instance_change_event(),
      /* = Overrides ============================================================ */
      InstanceState::PinMatchOverridesSemverRangeMatch => {
        record_override_event(instance.actual_specifier.unwrap().clone());
      }
      InstanceState::PinMatchOverridesSemverRangeMismatch => {
        record_override_event(instance.get_specifier_with_preferred_semver_range().unwrap().unwrap().clone());
      }
      /* = Fixable ============================================================== */
      InstanceState::Banned => record_fixable_mismatch_event(),
      InstanceState::MismatchesLocal => record_fixable_mismatch_event(),
      InstanceState::MismatchesPreferVersion => record_fixable_mismatch_event(),
      InstanceState::MismatchesPin => record_fixable_mismatch_event(),
      InstanceState::SemverRangeMismatch => record_fixable_mismatch_event(),
      InstanceState::SemverRangeMismatchWillFixSameRangeGroup => record_fixable_mismatch_event(),
      InstanceState::SemverRangeMismatchWillMatchSameRangeGroup => record_fixable_mismatch_event(),
      /* = Conflict ============================================================= */
      InstanceState::PinMatchConflictsWithSemverGroup => record_unfixable_mismatch_event(),
      InstanceState::SameRangeMatchConflictsWithSemverGroup => record_unfixable_mismatch_event(),
      InstanceState::SemverRangeMatchConflictsWithPreferVersion => record_unfixable_mismatch_event(),
      InstanceState::SemverRangeMismatchConflictsWithPreferVersion => record_unfixable_mismatch_event(),
      /* = Unfixable ============================================================ */
      InstanceState::MismatchesInvalidLocalVersion => record_unfixable_mismatch_event(),
      InstanceState::MismatchesNonSemverPreferVersion => record_unfixable_mismatch_event(),
      InstanceState::SemverRangeMismatchWontFixSameRangeGroup => record_unfixable_mismatch_event(),
      InstanceState::MismatchesSameRangeGroup => record_unfixable_mismatch_event(),
    };
  }
}
