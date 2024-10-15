use std::cmp::Ordering;

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
