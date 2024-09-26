use crate::{
  context::InstancesById, dependency::Dependency, group_selector::GroupSelector, instance::InstanceId,
  package_json::PackageJson, packages::Packages, specifier::Specifier,
};

pub mod fix;
pub mod lint;
pub mod mock;

/// Side effects in Syncpack commands are handled by structs which implement
/// this trait. Multiple commands such as `lint`, `fix`, and `json` all depend
/// on the same core logic, but have different side effects.
///
/// This trait allows the core logic to be reused across all commands, while the
/// side effects are handled by the command-specific structs which implement
/// this trait.
pub trait Effects {
  fn on(&mut self, event: Event, instances_by_id: &mut InstancesById);
  fn on_instance(&mut self, event: InstanceEvent, instances_by_id: &mut InstancesById);
  fn get_packages(&mut self) -> Packages;
  fn set_packages(&mut self, packages: Packages);
}

#[derive(Debug)]
pub enum Event<'a> {
  /// Syncpack is about to lint/fix versions/ranges, if enabled
  EnterVersionsAndRanges,
  /// A version/semver group is next to be processed
  GroupVisited(&'a GroupSelector),
  DependencyValid(&'a Dependency, Option<Specifier>),
  DependencyInvalid(&'a Dependency, Option<Specifier>),
  DependencyWarning(&'a Dependency, Option<Specifier>),
  /// Syncpack is about to lint/fix formatting, if enabled
  EnterFormat,
  /// Linting/fixing of formatting of a package.json file has completed and the
  /// package was already valid
  FormatMatch(&'a FormatEvent<'a>),
  /// Linting/fixing of formatting of a package.json file has completed and the
  /// package was initially invalid. In the case of fixing, they are now valid
  /// but were invalid beforehand
  FormatMismatch(&'a FormatEvent<'a>),
  /// Linting/fixing has completed
  ExitCommand,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InstanceEventVariant {
  /* = Ignored ============================================================== */
  InstanceIsIgnored,
  /* = Matches ============================================================== */
  LocalInstanceIsPreferred,
  InstanceMatchesLocal,
  InstanceMatchesHighestOrLowestSemver,
  InstanceMatchesButIsUnsupported,
  InstanceMatchesPinned,
  /// ✓ Instance matches its same range group
  /// ✓ Instance matches its semver group
  InstanceMatchesSameRangeGroup,
  /* = Warnings ============================================================= */
  /// Misconfiguration: Syncpack refuses to change local dependency specifiers
  LocalInstanceMistakenlyBanned,
  /// Misconfiguration: Syncpack refuses to change local dependency specifiers
  LocalInstanceMistakenlyMismatchesSemverGroup,
  /// Misconfiguration: Syncpack refuses to change local dependency specifiers
  LocalInstanceMistakenlyMismatchesPinned,
  /* = Fixable with config to resolve conflict ============================== */
  InstanceMatchesPinnedButMismatchesSemverGroup,
  InstanceMatchesLocalButMismatchesSemverGroup,
  /// Instance has the highest actual version but does not match its semver
  /// group, if we fix the semver group it will no longer match the highest
  /// expected version
  InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup,
  /* = Fixable ============================================================== */
  InstanceIsBanned,
  InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed,
  InstanceMismatchesLocal,
  InstanceMismatchesHighestOrLowestSemver,
  InstanceMismatchesPinned,
  /* = Unfixable ============================================================ */
  /// A local package is depended on but its package.json is missing a version
  InstanceMismatchesLocalWithMissingVersion,
  InstanceMismatchesAndIsUnsupported,
  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✘ If semver group is fixed, instance would still mismatch its same range group
  InstanceMismatchesBothSameRangeAndConflictingSemverGroups,
  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would match its same range group
  InstanceMismatchesBothSameRangeAndCompatibleSemverGroups,
  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✘ If semver group is fixed, instance would then mismatch its same range group
  InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup,
  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would still match its same range group
  InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup,
  /// ✘ Instance mismatches its same range group
  /// ✓ Instance matches its semver group
  /// ✘ We can't know what range the user wants and have to ask them
  InstanceMismatchesSameRangeGroup,
}

#[derive(Debug)]
pub struct InstanceEvent<'a> {
  pub dependency: &'a Dependency,
  pub instance_id: InstanceId,
  pub variant: InstanceEventVariant,
}

#[derive(Debug)]
pub struct FormatEvent<'a> {
  /// The package.json file being linted
  pub package_json: &'a PackageJson,
  /// Whether `rcfile.format_bugs` is enabled and matches
  pub format_bugs_is_valid: Option<bool>,
  /// Whether `rcfile.format_repository` is enabled and matches
  pub format_repository_is_valid: Option<bool>,
  /// Whether `rcfile.sort_az` is enabled and matches
  pub sort_az_is_valid: Option<bool>,
  /// Whether `rcfile.sort_first` is enabled and matches
  pub sort_first_is_valid: Option<bool>,
  /// Whether `rcfile.sort_exports` is enabled and matches
  pub sort_exports_is_valid: Option<bool>,
}
