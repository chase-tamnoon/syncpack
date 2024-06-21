use crate::{
  context::InstancesById,
  dependency::Dependency,
  group_selector::GroupSelector,
  instance::{Instance, InstanceId},
  package_json::PackageJson,
  packages::Packages,
  specifier::Specifier,
};

/// Side effects in Syncpack commands are handled by structs which implement
/// this trait. Multiple commands such as `lint`, `fix`, and `json` all depend
/// on the same core logic, but have different side effects.
///
/// This trait allows the core logic to be reused across all commands, while the
/// side effects are handled by the command-specific structs which implement
/// this trait.
pub trait Effects {
  // @TODO: split this into multiple methods for instance events etc
  fn on(&mut self, event: Event, instances_by_id: &mut InstancesById) -> ();
  fn get_packages(&mut self) -> Packages;
  fn set_packages(&mut self, packages: Packages) -> ();
}

#[derive(Debug)]
pub enum Event<'a> {
  /// Syncpack is about to lint/fix versions/ranges, if enabled
  EnterVersionsAndRanges,
  /// Syncpack is about to lint/fix formatting, if enabled
  EnterFormat,
  /// Linting/fixing has completed
  ExitCommand,

  /// Linting/fixing of formatting has completed and these packages were valid
  PackagesMatchFormatting(Vec<&'a PackageJson>),
  /// Linting/fixing of formatting has completed and these packages were
  /// initially invalid. In the case of fixing, they are now valid but were
  /// invalid beforehand.
  PackagesMismatchFormatting(Vec<&'a PackageJson>),

  /// A version/semver group is next to be processed
  GroupVisited(&'a GroupSelector),

  DependencyValid(&'a Dependency),
  DependencyInvalid(&'a Dependency),
  DependencyWarning(&'a Dependency),

  LocalInstanceIsPreferred(InstanceId /*&'a Instance*/),
  InstanceMatchesLocal(InstanceId /*&'a Instance*/),
  InstanceMatchesHighestOrLowestSemver(InstanceId /*&'a Instance*/),
  InstanceMatchesButIsUnsupported(InstanceId /*&'a Instance*/),
  InstanceIsIgnored(InstanceId /*&'a Instance*/),
  InstanceMatchesPinned(InstanceId /*&'a Instance*/),

  /// ✓ Instance matches its same range group
  /// ✓ Instance matches its semver group
  InstanceMatchesSameRangeGroup(InstanceId /*&'a Instance*/),

  /// Misconfiguration: Syncpack refuses to change local dependency specifiers
  LocalInstanceMistakenlyBanned(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  InstanceIsBanned(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  InstanceMatchesHighestOrLowestSemverButMismatchesSemverGroup(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  InstanceMatchesLocalButMismatchesSemverGroup(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  InstanceMismatchesLocal(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  InstanceMismatchesHighestOrLowestSemver(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  InstanceMismatchesAndIsUnsupported(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  /// Misconfiguration: Syncpack refuses to change local dependency specifiers
  LocalInstanceMistakenlyMismatchesSemverGroup(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  InstanceMatchesPinnedButMismatchesSemverGroup(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  /// Misconfiguration: Syncpack refuses to change local dependency specifiers
  LocalInstanceMistakenlyMismatchesPinned(InstanceId /*&'a mut Instance*/),

  InstanceMismatchesPinned(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✘ If semver group is fixed, instance would still mismatch its same range group
  InstanceMismatchesBothSameRangeAndConflictingSemverGroups(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would match its same range group
  InstanceMismatchesBothSameRangeAndCompatibleSemverGroups(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✘ If semver group is fixed, instance would then mismatch its same range group
  InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would still match its same range group
  InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup(InstanceId /*&'a mut Instance, &'a mut Packages*/),

  /// ✘ Instance mismatches its same range group
  /// ✓ Instance matches its semver group
  /// ✘ We can't know what range the user wants and have to ask them
  InstanceMismatchesSameRangeGroup(InstanceId /*&'a mut Instance, &'a mut Packages*/),
}

/// A single instance of a dependency was found, which is valid
#[derive(Debug)]
pub struct MatchEvent<'a> {
  // pub dependency: &'a Dependency,
  pub instance: &'a Instance,
}

/// A single instance of a dependency was found, which is not valid
#[derive(Debug)]
pub struct MismatchEvent<'a> {
  // pub dependency: &'a Dependency,
  pub instance: &'a Instance,
}

// /// A single instance of a dependency was found, which is valid
// #[derive(Debug)]
// pub struct MatchEvent<'a> {
//   /// all instances of this dependency (eg. "react") in this version group
//   pub dependency: &'a Dependency,
//   /// the unique identifier for the instance which was found
//   pub instance_id: InstanceId,
//   /// the version specifier on the instance which was found
//   pub specifier: Specifier,
// }

// /// A single instance of a dependency was found, which is not valid
// #[derive(Debug)]
// pub struct MismatchEvent<'a> {
//   /// all instances of this dependency (eg. "react") in this version group
//   pub dependency: &'a Dependency,
//   /// the unique identifier for the instance which was found
//   pub instance_id: InstanceId,
//   /// the correct version specifier the instance should have had
//   pub expected_specifier: Specifier,
//   /// the incorrect version specifier the instance actually has
//   pub actual_specifier: Specifier,
//   /// other instances which do have the correct version specifier
//   pub matching_instance_ids: Vec<InstanceId>,
//   /// all instances in the workspace
//   pub instances_by_id: &'a mut InstancesById,
//   /// all packages in the workspace
//   pub packages: &'a mut Packages,
// }

/// A single instance of a dependency was found, where or or one of the other
/// instances of this dependency have a version specifier which is not
/// understood by syncpack
#[derive(Debug)]
pub struct UnsupportedMismatchEvent<'a> {
  /// all instances of this dependency (eg. "react") in this version group
  pub dependency: &'a Dependency,
  /// all instances in the workspace
  pub instances_by_id: &'a mut InstancesById,
  /// the unique identifier for the instance which was found
  pub instance_id: InstanceId,
  /// the incorrect version specifier the instance actually has
  pub specifier: Specifier,
}

/// A single instance of a dependency was found, which is not allowed
#[derive(Debug)]
pub struct BannedEvent<'a> {
  /// all instances of this dependency (eg. "react") in this version group
  pub dependency: &'a Dependency,
  /// all instances in the workspace
  pub instances_by_id: &'a mut InstancesById,
  /// all packages in the workspace
  pub packages: &'a mut Packages,
  /// the unique identifier for the instance which was found
  pub instance_id: InstanceId,
  /// the version specifier the banned instance has
  pub specifier: Specifier,
}

/// A single instance of a dependency was found, which is not valid
#[derive(Debug)]
pub struct SameRangeMismatchEvent<'a> {
  /// all instances of this dependency (eg. "react") in this version group
  pub dependency: &'a Dependency,
  /// all instances in the workspace
  pub instances_by_id: &'a mut InstancesById,
  /// all packages in the workspace
  pub packages: &'a mut Packages,
  /// the unique identifier for the instance which was found
  pub instance_id: InstanceId,
  /// the range specifier which does not match every other range
  pub specifier: Specifier,
  /// another range specifier which is not matched by this instance
  pub specifier_outside_range: Specifier,
  /// the instance IDs which have the specifier_outside_range
  pub instance_ids_outside_range: Vec<InstanceId>,
}

/// A single instance of a dependency was found, which is not valid
#[derive(Debug)]
pub struct SnapToMismatchEvent<'a> {
  /// all instances of this dependency (eg. "react") in this version group
  pub dependency: &'a Dependency,
  /// the unique identifier for the instance which was found
  pub instance_id: InstanceId,
  /// the correct version specifier the instance should have had
  pub expected_specifier: Specifier,
  /// the incorrect version specifier the instance actually has
  pub actual_specifier: Specifier,
  /// the instance with the version specifier to be snapped to
  pub snap_to_instance_id: InstanceId,
  /// all instances in the workspace
  pub instances_by_id: &'a mut InstancesById,
  /// all packages in the workspace
  pub packages: &'a mut Packages,
}
