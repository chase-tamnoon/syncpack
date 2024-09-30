use std::rc::Rc;

use serde_json::Value;

use crate::{dependency::Dependency, group_selector::GroupSelector, instance::Instance, specifier::Specifier};

pub mod fix;
pub mod lint;

/// Side effects in Syncpack commands are handled by structs which implement
/// this trait. Multiple commands such as `lint`, `fix`, and `json` all depend
/// on the same core logic, but have different side effects.
///
/// This trait allows the core logic to be reused across all commands, while the
/// side effects are handled by the command-specific structs which implement
/// this trait.
pub trait Effects {
  fn on(&mut self, event: Event);
  fn on_instance(&mut self, event: InstanceEvent);
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
  PackageFormatMatch(String),
  /// Linting/fixing of formatting of a package.json file has completed and the
  /// package was initially invalid. In the case of fixing, they are now valid
  /// but were invalid beforehand
  PackageFormatMismatch(PackageFormatEvent),
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
  pub instance: Rc<Instance>,
  pub variant: InstanceEventVariant,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FormatEventVariant {
  /// ✓ `rcFile.formatBugs` is enabled
  /// ✘ The `bugs` property is not formatted
  BugsPropertyIsNotFormatted,
  /// ✓ `rcFile.formatRepository` is enabled
  /// ✘ The `repository` property is not formatted
  RepositoryPropertyIsNotFormatted,
  /// ✓ `rcFile.sortAz` is enabled
  /// ✘ This property is not sorted alphabetically
  PropertyIsNotSortedAz,
  /// ✓ `rcFile.sortPackages` is enabled
  /// ✘ This package.json's properties are not sorted
  PackagePropertiesAreNotSorted,
  /// ✓ `rcFile.sortExports` is enabled
  /// ✘ The `exports` property is not sorted
  ExportsPropertyIsNotSorted,
}

#[derive(Debug)]
pub struct PackageFormatEvent {
  /// The name of the package.json file with formatting issues
  pub package_name: String,
  /// Each formatting issue in this file
  pub formatting_mismatches: Vec<FormatEvent>,
}

#[derive(Debug)]
pub struct FormatEvent {
  /// The formatted value
  pub expected: Value,
  /// The name of the package.json file being linted
  pub package_name: String,
  /// The path to the property that was linted
  pub property_path: String,
  /// The broken linting rule
  pub variant: FormatEventVariant,
}
