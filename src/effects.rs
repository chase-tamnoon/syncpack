use crate::{
  context::Context,
  dependency::{Dependency, InstancesById},
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
  fn on(&mut self, event: Event) -> ();
  // fn set_context(&mut self, ctx: Context) -> ();
  // fn borrow_context(&self) -> &Context;
}

#[derive(Debug)]
pub enum Event<'a, 'b> {
  /// All package.json files have been read from the workspace
  PackagesLoaded(&'a Packages),

  /// Syncpack is about to lint/fix versions/ranges, if enabled
  EnterVersionsAndRanges,
  /// Syncpack is about to lint/fix formatting, if enabled
  EnterFormat,
  /// Linting/fixing has completed
  ExitCommand,

  /// Linting/fixing of formatting has completed and these packages were valid
  PackagesMatchFormatting(&'b Vec<&'a PackageJson>),
  /// Linting/fixing of formatting has completed and these packages were
  /// initially invalid. In the case of fixing, they are now valid but were
  /// invalid beforehand.
  PackagesMismatchFormatting(&'b Vec<&'a PackageJson>),

  /// A version/semver group is next to be processed
  GroupVisited(&'a GroupSelector),

  /// A dependency in an ignored version group has been found
  DependencyIgnored(&'a Dependency),
  /// A dependency in a banned version group has been found
  DependencyBanned(&'a Dependency),
  /// A dependency in a WithRange semver group has been found where all
  /// instances are valid
  DependencyMatchesWithRange(&'a Dependency),
  /// A dependency in a WithRange semver group has been found where one or more
  /// instances with semver versions with a semver range that are not the same
  /// as the `.range`
  DependencyMismatchesWithRange(&'a Dependency),
  /// A dependency in a pinned version group has been found where all
  /// instances are valid
  DependencyMatchesPinnedVersion(&'a Dependency),
  /// A dependency in a pinned version group has been found which has one
  /// or more instances with versions that are not the same as the `.pinVersion`
  DependencyMismatchesPinnedVersion(&'a Dependency),
  /// A dependency in a same range version group has been found where all
  /// instances are valid
  DependencyMatchesSameRange(&'a Dependency),
  /// A dependency in a same range version group has been found which has
  /// one or more instances with versions that are not a semver range which
  /// satisfies all of the other semver ranges in the group
  DependencyMismatchesSameRange(&'a Dependency),
  /// A dependency in a snapped to version group has been found where all
  /// instances are valid
  DependencyMatchesSnapTo(&'a Dependency),
  /// A dependency in a snapped to version group has been found which has
  /// one or more instances with versions that are not the same as those used
  /// by the packages named in the `.snapTo` config array
  DependencyMismatchesSnapTo(&'a Dependency),
  /// A dependency in a standard version group has been found where all
  /// instances are valid
  DependencyMatchesStandard(&'a Dependency),
  /// A dependency in a standard version group has been found which has
  /// one or more instances with versions that are not the same as the others
  DependencyMismatchesStandard(&'a Dependency),

  // /// A valid instance in a standard version group has been found
  // InstanceMatchesStandard(&'a MatchEvent<'a>),
  // /// An instance in a banned version group has been found
  // InstanceBanned(&'a mut BannedEvent<'a>),
  // /// An instance matches its SemverGroup's version range
  // InstanceMatchesWithRange(&'a MatchEvent<'a>),
  // /// An instance does not match its SemverGroup's version range
  // InstanceMismatchesWithRange(&'a mut MismatchEvent<'a>),
  // /// An instance in a pinned version group has been found whose version is not
  // /// the same as the `.pinVersion`
  // InstanceMismatchesPinnedVersion(&'a mut MismatchEvent<'a>),
  // /// An instance in a same range version group has been found which has a
  // /// version which is not a semver range which satisfies all of the other
  // /// semver ranges in the group
  // InstanceMismatchesSameRange(&'a mut SameRangeMismatchEvent<'a>),
  // /// An instance in a snapped to version group has been found which has a
  // /// version that is not the same as those used by the packages named in the
  // /// `.snapTo` config array
  // InstanceMismatchesSnapTo(&'a mut SnapToMismatchEvent<'a>),
  // /// An instance in a standard version group has been found which is a
  // /// dependency developed in this repo, its version does not match the
  // /// `.version` property of the package.json file for this package in the repo
  // InstanceMismatchesLocalVersion(&'a mut MismatchEvent<'a>),
  // /// A misconfiguration by the user has resulted in the .version property of a
  // /// local package.json to be written to. Syncpack should refuse to do this.
  // InstanceMismatchCorruptsLocalVersion(&'a mut MismatchEvent<'a>),
  // /// An instance in a standard version group has been found which has a version
  // /// which is not identical to the others, but not all of the instances have
  // /// valid or supported version specifiers, so it's impossible to know which
  // /// should be preferred
  // InstanceUnsupportedMismatch(&'a mut UnsupportedMismatchEvent<'a>),
  // /// An instance in a standard version group has been found which has a semver
  // /// version which is higher than the lowest semver version in the group, and
  // /// `.preferVersion` is set to `lowestSemver`
  // InstanceMismatchesLowestVersion(&'a mut MismatchEvent<'a>),
  // /// An instance in a standard version group has been found which has a semver
  // /// version which is lower than the highest semver version in the group, and
  // /// `.preferVersion` is set to `highestSemver`
  // InstanceMismatchesHighestVersion(&'a mut MismatchEvent<'a>),
  /**/
  LocalInstanceIsPreferred(&Instance),
  InstanceMatchesLocal(&Instance),
  InstanceMatchesHighestOrLowestSemver(&Instance),
  InstanceMatchesButIsUnsupported(&Instance),
  InstanceIsIgnored(&Instance),
  InstanceMatchesPinned(&Instance),
  InstanceMatchesSameRangeGroup(&Instance),

  LocalInstanceMistakenlyBanned(&'a mut Instance, &'a mut Packages),
  InstanceIsBanned(&'a mut Instance, &'a mut Packages),
  InstanceMatchesHighestOrLowestSemverButMismatchesSemverGroup(&'a mut Instance, &'a mut Packages),
  InstanceMatchesLocalButMismatchesSemverGroup(&'a mut Instance, &'a mut Packages),
  InstanceMismatchesLocal(&'a mut Instance, &'a mut Packages),
  InstanceMismatchesHighestOrLowestSemver(&'a mut Instance, &'a mut Packages),
  InstanceMismatchesAndIsUnsupported(&'a mut Instance, &'a mut Packages),
  LocalInstanceMistakenlyMismatchesSemverGroup(&'a mut Instance, &'a mut Packages),
  InstanceMatchesPinnedButMismatchesSemverGroup(&'a mut Instance, &'a mut Packages),
  LocalInstanceMistakenlyMismatchesPinned(&'a mut Instance, &'a mut Packages),
  InstanceMismatchesPinned(&'a mut Instance, &'a mut Packages),
  InstanceMismatchesBothSameRangeAndConflictingSemverGroups(&'a mut Instance, &'a mut Packages),
  InstanceMismatchesBothSameRangeAndCompatibleSemverGroups(&'a mut Instance, &'a mut Packages),
  InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup(&'a mut Instance, &'a mut Packages),
  InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup(&'a mut Instance, &'a mut Packages),
  InstanceMismatchesSameRangeGroup(&'a mut Instance, &'a mut Packages),
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
