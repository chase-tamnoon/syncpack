use log::debug;
use serde_json::Value;
use std::{cell::RefCell, cmp::Ordering, path::PathBuf, rc::Rc};

use crate::{
  dependency_type::{DependencyType, Strategy},
  package_json::PackageJson,
  semver_group::SemverGroup,
  specifier::{semver::Semver, semver_range::SemverRange, Specifier},
};

pub type InstanceId = String;

#[derive(Debug)]
pub struct Instance {
  /// The original version specifier, which should never be mutated.
  /// eg. `Specifier::Exact("16.8.0")`, `Specifier::Range("^16.8.0")`
  pub actual_specifier: Specifier,
  /// The dependency type to use to read/write this instance
  pub dependency_type: DependencyType,
  /// The version specifier which syncpack has determined this instance should
  /// be set to, if it was not possible to determine without user intervention,
  /// this will be a `None`.
  pub expected_specifier: RefCell<Option<Specifier>>,
  /// The file path of the package.json file this instance belongs to
  pub file_path: PathBuf,
  /// A unique identifier for this instance
  pub id: InstanceId,
  /// Whether this is a package developed in this repo
  pub is_local: bool,
  /// eg. "in /devDependencies of @foo/numberwang"
  pub location_hint: String,
  /// The dependency name eg. "react", "react-dom"
  pub name: String,
  /// The `.name` of the package.json this file is in
  pub package: Rc<RefCell<PackageJson>>,
  /// If this instance belongs to a `WithRange` semver group, this is the range.
  /// This is used by Version Groups while determining the preferred version,
  /// to try to also satisfy any applicable semver group ranges
  pub preferred_semver_range: RefCell<Option<SemverRange>>,
  /// The state of whether this instance has not been processed yet
  /// (InstanceState::Unknown) or when it has, what it was found to be
  pub state: RefCell<InstanceState>,
}

impl Instance {
  pub fn new(
    name: String,
    // The initial, unwrapped specifier (eg. "1.1.0") from the package.json file
    raw_specifier: String,
    dependency_type: &DependencyType,
    package: Rc<RefCell<PackageJson>>,
  ) -> Instance {
    let package_name = package.borrow().get_name_unsafe();
    let specifier = Specifier::new(&raw_specifier);
    Instance {
      actual_specifier: specifier.clone(),
      dependency_type: dependency_type.clone(),
      expected_specifier: RefCell::new(None),
      file_path: package.borrow().file_path.clone(),
      id: format!("{} in {} of {}", name, &dependency_type.path, package_name),
      is_local: dependency_type.path == "/version",
      location_hint: format!("in {} of {}", &dependency_type.path, package_name),
      name,
      package: Rc::clone(&package),
      preferred_semver_range: RefCell::new(None),
      state: RefCell::new(InstanceState::Unknown),
    }
  }

  /// Record what syncpack has determined the state of this instance is and what
  /// its expected specifier should be
  pub fn set_state(&self, state: InstanceState, expected_specifier: &Specifier) -> &Self {
    *self.state.borrow_mut() = state;
    *self.expected_specifier.borrow_mut() = Some(expected_specifier.clone());
    self
  }

  pub fn mark_valid(&self, state: ValidInstance, expected_specifier: &Specifier) -> &Self {
    self.set_state(InstanceState::Valid(state), expected_specifier)
  }

  pub fn mark_suspect(&self, state: SuspectInstance, expected_specifier: &Specifier) -> &Self {
    self.set_state(InstanceState::Suspect(state), expected_specifier)
  }

  pub fn mark_invalid(&self, state: InvalidInstance, expected_specifier: &Specifier) -> &Self {
    self.set_state(InstanceState::Invalid(state), expected_specifier)
  }

  pub fn mark_fixable(&self, state: FixableInstance, expected_specifier: &Specifier) -> &Self {
    self.mark_invalid(InvalidInstance::Fixable(state), expected_specifier)
  }

  pub fn mark_conflict(&self, state: SemverGroupAndVersionConflict, expected_specifier: &Specifier) -> &Self {
    self.mark_invalid(InvalidInstance::Conflict(state), expected_specifier)
  }

  pub fn mark_unfixable(&self, state: UnfixableInstance, expected_specifier: &Specifier) -> &Self {
    self.mark_invalid(InvalidInstance::Unfixable(state), expected_specifier)
  }

  /// If this instance should use a preferred semver range, store it
  pub fn set_semver_group(&self, group: &SemverGroup) {
    if let Some(range) = &group.range {
      *self.preferred_semver_range.borrow_mut() = Some(range.clone());
    }
  }

  /// Does this instance's actual specifier match the expected specifier?
  pub fn already_equals(&self, expected: &Specifier) -> bool {
    self.actual_specifier == *expected
  }

  pub fn will_match_with_preferred_semver_range(&self, expected: &Specifier) -> bool {
    self.get_specifier_with_preferred_semver_range().unwrap() == *expected
  }

  /// Does this instance belong to a `WithRange` semver group?
  pub fn must_match_preferred_semver_range(&self) -> bool {
    self.preferred_semver_range.borrow().is_some()
  }

  /// Does this instance belong to a `WithRange` semver group and which prefers
  /// a semver range other than the given range?
  ///
  /// This is a convenience method for the common case where a preferred semver
  /// range only matters if what is preferred is not the same as the expected
  /// version of a dependency which you are trying to synchronise to
  pub fn must_match_preferred_semver_range_which_is_not(&self, needed_range: &SemverRange) -> bool {
    self.must_match_preferred_semver_range() && !self.preferred_semver_range_is(needed_range)
  }

  /// Does this instance belong to a `WithRange` semver group and which prefers
  /// a semver range other than that used by the given specifier?
  pub fn must_match_preferred_semver_range_which_differs_to(&self, other_specifier: &Specifier) -> bool {
    other_specifier.get_semver_range().map_or(false, |range_of_other_specifier| {
      self.must_match_preferred_semver_range_which_is_not(&range_of_other_specifier)
    })
  }

  pub fn preferred_semver_range_is(&self, range: &SemverRange) -> bool {
    self.preferred_semver_range.borrow().as_ref().map(|r| r == range).unwrap_or(false)
  }

  /// Does this instance belong to a `WithRange` semver group and also have a
  /// specifier which matches its preferred semver range?
  pub fn matches_preferred_semver_range(&self) -> bool {
    self
      .preferred_semver_range
      .borrow()
      .as_ref()
      .map(|preferred_semver_range| self.actual_specifier.has_semver_range_of(preferred_semver_range))
      .unwrap_or(false)
  }

  /// Get the expected version specifier for this instance with the semver
  /// group's preferred range applied
  pub fn get_specifier_with_preferred_semver_range(&self) -> Option<Specifier> {
    self.preferred_semver_range.borrow().as_ref().and_then(|preferred_semver_range| {
      self
        .actual_specifier
        .get_simple_semver()
        .map(|actual| Specifier::Semver(Semver::Simple(actual.with_range(preferred_semver_range))))
    })
  }

  /// Does this instance's specifier match the specifier of every one of the
  /// given instances?
  pub fn already_satisfies_all(&self, instances: &[Rc<Instance>]) -> bool {
    !matches!(self.actual_specifier, Specifier::None)
      && self
        .actual_specifier
        .satisfies_all(instances.iter().map(|i| &i.actual_specifier).collect())
  }

  /// Will this instance's specifier, once fixed to match its semver group,
  /// satisfy the given specifier?
  pub fn specifier_with_preferred_semver_range_will_satisfy(&self, other: &Specifier) -> bool {
    self
      .get_specifier_with_preferred_semver_range()
      .map(|specifier| specifier.satisfies(other))
      .unwrap_or(false)
  }

  /// Delete a version/dependency/instance from the package.json
  pub fn remove_from(&self, package: &PackageJson) {
    match self.dependency_type.strategy {
      Strategy::NameAndVersionProps => {
        debug!("@TODO: remove instance for NameAndVersionProps");
      }
      Strategy::NamedVersionString => {
        debug!("@TODO: remove instance for NamedVersionString");
      }
      Strategy::UnnamedVersionString => {
        debug!("@TODO: remove instance for UnnamedVersionString");
      }
      Strategy::VersionsByName => {
        let path_to_obj = &self.dependency_type.path;
        let name = &self.name;
        if let Some(Value::Object(obj)) = package.contents.borrow_mut().pointer_mut(path_to_obj) {
          obj.remove(name);
        }
      }
      Strategy::InvalidConfig => {
        panic!("unrecognised strategy");
      }
    };
  }
}

#[derive(Clone, Debug)]
pub enum InstanceState {
  Unknown,
  Valid(ValidInstance),
  Invalid(InvalidInstance),
  Suspect(SuspectInstance),
}

impl InstanceState {
  pub fn valid(state: ValidInstance) -> Self {
    InstanceState::Valid(state)
  }
  pub fn suspect(state: SuspectInstance) -> Self {
    InstanceState::Suspect(state)
  }
  pub fn fixable(state: FixableInstance) -> Self {
    InstanceState::Invalid(InvalidInstance::Fixable(state))
  }
  pub fn conflict(state: SemverGroupAndVersionConflict) -> Self {
    InstanceState::Invalid(InvalidInstance::Conflict(state))
  }
  pub fn unfixable(state: UnfixableInstance) -> Self {
    InstanceState::Invalid(InvalidInstance::Unfixable(state))
  }
  pub fn get_name(&self) -> String {
    match self {
      InstanceState::Unknown => "Unknown".to_string(),
      InstanceState::Valid(variant) => format!("{:?}", variant),
      InstanceState::Invalid(variant) => match variant {
        InvalidInstance::Fixable(variant) => format!("{:?}", variant),
        InvalidInstance::Conflict(variant) => format!("{:?}", variant),
        InvalidInstance::Unfixable(variant) => format!("{:?}", variant),
      },
      InstanceState::Suspect(variant) => format!("{:?}", variant),
    }
  }
}

impl PartialEq for InstanceState {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

impl Eq for InstanceState {}

impl PartialOrd for InstanceState {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for InstanceState {
  /// The order of severity is:
  /// 1. Unknown
  /// 2. Valid
  /// 3. Suspect
  /// 4. Invalid
  fn cmp(&self, other: &Self) -> Ordering {
    use InstanceState::*;
    match (self, other) {
      (Unknown, Unknown) | (Valid(_), Valid(_)) | (Suspect(_), Suspect(_)) | (Invalid(_), Invalid(_)) => Ordering::Equal,
      (Unknown, _) => Ordering::Less,
      (Valid(_), _) => Ordering::Less,
      (_, Valid(_)) => Ordering::Greater,
      (_, Unknown) => Ordering::Greater,
      (Suspect(_), _) => Ordering::Less,
      (_, Suspect(_)) => Ordering::Greater,
    }
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValidInstance {
  /// - ✓ Instance is configured to be ignored by Syncpack
  Ignored,
  /// - ✓ Instance is a local package and its version is valid
  ValidLocal,
  /// - ✓ Instance identical to the version of its locally-developed package
  /// - ✓ Instance matches its semver group
  EqualsLocal,
  /// - ✓ Instance matches the version of its locally-developed package
  /// - ✓ Instance matches its semver group
  /// - ! Considered a loose match we should highlight
  MatchesLocal,
  /// - ✓ Instance identical to highest/lowest semver in its group
  /// - ✓ Instance matches its semver group
  EqualsPreferVersion,
  /// - ✓ Instance has same semver number as highest/lowest semver in its group
  /// - ✓ Instance matches its semver group
  /// - ✓ Range preferred by semver group satisfies the highest/lowest semver
  /// - ! Considered a loose match we should highlight
  MatchesPreferVersion,
  /// - ! No Instances are simple semver
  /// - ✓ Instance identical to every other instance in its version group
  EqualsNonSemverPreferVersion,
  /// - ✓ Instance identical to its pinned version group
  /// - ✓ Instance matches its semver group
  EqualsPin,
  /// - ✓ Instance matches its same range group
  /// - ✓ Instance matches its semver group
  MatchesSameRangeGroup,
  /// - ✓ Instance identical to a matching snapTo instance
  /// - ✓ Instance matches its semver group
  EqualsSnapToVersion,
  /// - ✓ Instance has same semver number as matching snapTo instance
  /// - ✓ Instance matches its semver group
  /// - ✓ Range preferred by semver group satisfies the matching snapTo instance
  /// - ! Considered a loose match we should highlight
  MatchesSnapToVersion,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InvalidInstance {
  Fixable(FixableInstance),
  Unfixable(UnfixableInstance),
  Conflict(SemverGroupAndVersionConflict),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FixableInstance {
  /// - ✘ Instance is in a banned version group
  Banned,
  /// - ✘ Instance mismatches the version of its locally-developed package
  MismatchesLocal,
  /// - ✘ Instance mismatches highest/lowest semver in its group
  MismatchesPreferVersion,
  /// - ✘ Instance mismatches the matching snapTo instance
  MismatchesSnapToVersion,
  /// - ✘ Instance mismatches its pinned version group
  MismatchesPin,
  /// - ✓ Instance has same semver number as highest/lowest semver in its group
  /// - ✘ Instance mismatches its semver group
  /// - ✓ Range preferred by semver group satisfies the highest/lowest semver
  /// - ✓ Fixing the semver range satisfy both groups
  SemverRangeMismatch,
  /// - ✓ Instance has same semver number as its pinned version group
  /// - ✓ Instance matches its semver group
  /// - ! The semver group requires a range which is different to the pinned version
  /// - ! Pinned version wins
  PinMatchOverridesSemverRangeMatch,
  /// - ✓ Instance has same semver number as its pinned version group
  /// - ✘ Instance mismatches its semver group
  /// - ! The semver group requires a range which is different to the pinned version
  /// - ! Pinned version wins
  PinMatchOverridesSemverRangeMismatch,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UnfixableInstance {
  /// - ✘ Instance depends on a local package whose package.json version is not exact semver
  /// - ? We can't know what the version should be
  MismatchesInvalidLocalVersion,
  /// - ✘ Instance mismatches others in its group
  /// - ✘ One or more Instances are not simple semver
  /// - ? We can't know what's right or what isn't
  MismatchesNonSemverPreferVersion,
  /// - ✘ Instance mismatches its same range group
  /// - ? Instance has no semver group
  /// - ? We can't know what range the user wants and have to ask them
  MismatchesSameRangeGroup,
  /// - ✓ Instance is in a snapped to version group
  /// - ✘ An instance of the same dependency was not found in any of the snapped
  ///     to packages
  /// - ✘ This is a misconfiguration resulting in this instance being orphaned
  SnapToVersionNotFound,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SemverGroupAndVersionConflict {
  /// - ✓ Instance has same semver number as highest/lowest semver in its group
  /// - ✓ Instance matches its semver group
  /// - ✘ Range preferred by semver group will not satisfy the highest/lowest semver
  /// - ? We can't know whether the incompatible range matters or not and have to ask
  MatchConflictsWithPrefer,
  /// - ✓ Instance has same semver number as highest/lowest semver in its group
  /// - ✘ Instance mismatches its semver group
  /// - ✘ Range preferred by semver group will not satisfy the highest/lowest semver
  /// - ? We can't know whether the incompatible range matters or not and have to ask
  MismatchConflictsWithPrefer,
  /// - ✓ Instance has same semver number as the matching snapTo instance
  /// - ✓ Instance matches its semver group
  /// - ✘ Range preferred by semver group will not satisfy the matching snapTo instance
  /// - ? We can't know whether the incompatible range matters or not and have to ask
  MatchConflictsWithSnapTo,
  /// - ✓ Instance has same semver number as the matching snapTo instance
  /// - ✘ Instance mismatches its semver group
  /// - ✘ Range preferred by semver group will not satisfy the matching snapTo instance
  /// - ? We can't know whether the incompatible range matters or not and have to ask
  MismatchConflictsWithSnapTo,
  /// - ✓ Instance has same semver number as local instance in its group
  /// - ✓ Instance matches its semver group
  /// - ✘ Range preferred by semver group will not satisfy the local instance
  /// - ? We can't know whether the incompatible range matters or not and have to ask
  MatchConflictsWithLocal,
  /// - ✓ Instance has same semver number as local instance
  /// - ✘ Instance mismatches its semver group
  /// - ✘ Range preferred by semver group will not satisfy the local instance
  /// - ? We can't know whether the incompatible range matters or not and have to ask
  MismatchConflictsWithLocal,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SuspectInstance {
  /// - ✘ Local Instance is in a banned version group
  /// - ✘ Misconfiguration: Syncpack refuses to change local dependency specifiers
  RefuseToBanLocal,
  /// - ✘ Local Instance mismatches its pinned version group
  /// - ✘ Misconfiguration: Syncpack refuses to change local dependency specifiers
  RefuseToPinLocal,
  /// - ✘ Local Instance is in a snapped to version group
  /// - ✘ An Instance of this dependency was found in the snapped to package
  /// - ✘ Misconfiguration: Syncpack refuses to change local dependency specifiers
  RefuseToSnapLocal,
  /// - ! Local Instance has no version property
  /// - ! Not an error on its own unless an instance of it mismatches
  InvalidLocalVersion,
}
