use std::{cell::RefCell, rc::Rc};

use serde_json::Value;

use crate::{dependency::Dependency, group_selector::GroupSelector, instance::Instance, package_json::PackageJson};

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
  DependencyValid(&'a Dependency),
  DependencyInvalid(&'a Dependency),
  DependencyWarning(&'a Dependency),
  /// Syncpack is about to lint/fix formatting, if enabled
  EnterFormat,
  /// Linting/fixing of formatting of a package.json file has completed and the
  /// package was already valid
  PackageFormatMatch(Rc<RefCell<PackageJson>>),
  /// Linting/fixing of formatting of a package.json file has completed and the
  /// package was initially invalid. In the case of fixing, they are now valid
  /// but were invalid beforehand
  PackageFormatMismatch(FormatMismatchEvent),
  /// Linting/fixing has completed
  ExitCommand,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InstanceState {
  /// Instance has not yet
  Unknown,
  /* = Matches ============================================================== */
  /// ✓ Instance is configured to be ignored by Syncpack
  MatchesIgnored,
  /// ✓ Instance is a local package and its version is valid
  LocalWithValidVersion,
  /// ✓ Instance matches the version of its locally-developed package
  /// ✓ Instance matches its semver group
  MatchesLocal,
  /// ✓ Instance matches highest/lowest semver in its group
  /// ✓ Instance matches its semver group
  MatchesPreferVersion,
  /// ! Specifier is not supported by Syncpack
  /// ✓ Instance matches every other instance in its its version group
  MatchesButUnsupported,
  /// ✓ Instance matches its pinned version group
  /// ✓ Instance matches its semver group
  MatchesPin,
  /// ✓ Instance matches its same range group
  /// ✓ Instance matches its semver group
  MatchesSameRangeGroup,
  /* = Warnings ============================================================= */
  /// ✘ Local Instance is in a banned version group
  /// ✘ Misconfiguration: Syncpack refuses to change local dependency specifiers
  RefuseToBanLocal,
  /// ✘ Local Instance mismatches its same range version group
  /// ✘ Misconfiguration: Syncpack refuses to change local dependency specifiers
  RefuseToChangeLocalSemverRange,
  /// ✘ Local Instance mismatches its pinned version group
  /// ✘ Misconfiguration: Syncpack refuses to change local dependency specifiers
  RefuseToPinLocal,
  /// ! Local Instance has no version property
  /// ! Not an error on its own unless an instance of it mismatches
  MissingLocalVersion,
  /* = Fixable ============================================================== */
  /// ✘ Instance is in a banned version group
  Banned,
  /// ✘ Instance matches the version of its locally-developed package
  MismatchesLocal,
  /// ✘ Instance mismatches highest/lowest semver in its group
  MismatchesPreferVersion,
  /// ✘ Instance mismatches its pinned version group
  MismatchesPin,
  /// ✘ Instance mismatches highest/lowest semver in its group
  /// ✘ Instance mismatches its semver group
  /// ✓ Fixing the semver range will make this instance the highest/lowest
  ///   semver in its group
  SemverRangeMismatchWillFixPreferVersion,
  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would match its same range group
  SemverRangeMismatchWillFixSameRangeGroup,
  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✓ If semver group is fixed, instance would still match its same range group
  SemverRangeMismatchWillMatchSameRangeGroup,
  /* = Conflict ============================================================= */
  /// ✓ Instance matches its pinned version group
  /// ✘ Instance mismatches its semver group
  /// ? If we fix the semver group it will mismatch the pinned version
  PinMatchConflictsWithSemverGroup,
  /// ✓ Instance matches the version of its locally-developed package
  /// ✘ Instance mismatches its semver group
  /// ? If we fix the semver group it will mismatch the local version
  LocalMatchConflictsWithSemverGroup,
  /// ✓ Instance matches highest/lowest semver in its group
  /// ✘ Instance mismatches its semver group
  /// ? If we fix the semver group it will mismatch highest/lowest semver in
  ///   its group
  PreferVersionMatchConflictsWithSemverGroup,
  /// ✓ Instance matches its same range group
  /// ✘ Instance mismatches its semver group
  /// ? If semver group is fixed, instance would then mismatch its same range group
  SameRangeMatchConflictsWithSemverGroup,
  /* = Unfixable ============================================================ */
  /// ✘ Instance depends on a local package whose package.json is missing a version
  /// ? We can't know what the version should be
  MismatchesMissingLocalVersion,
  /// ✘ Instance mismatches others in its group
  /// ✘ One or more Instances have unsupported specifiers
  /// ? We can't know what's right or what isn't
  MismatchesUnsupported,
  /// ✘ Instance mismatches its same range group
  /// ✘ Instance mismatches its semver group
  /// ✘ If semver group is fixed, instance would still mismatch its same range group
  /// ? We can't know what range the user wants and have to ask them
  SemverRangeMismatchWontFixSameRangeGroup,
  /// ✘ Instance mismatches its same range group
  /// ? Instance has no semver group
  /// ? We can't know what range the user wants and have to ask them
  MismatchesSameRangeGroup,
}

#[derive(Debug)]
pub struct InstanceEvent<'a> {
  pub dependency: &'a Dependency,
  pub instance: Rc<Instance>,
  pub variant: InstanceState,
}

#[derive(Debug)]
pub struct FormatMismatchEvent {
  /// The name of the package.json file with formatting issues
  pub package: Rc<RefCell<PackageJson>>,
  /// Each formatting issue in this file
  pub formatting_mismatches: Vec<FormatMismatch>,
}

#[derive(Debug)]
pub struct FormatMismatch {
  /// The formatted value
  pub expected: Value,
  /// The name of the package.json file being linted
  pub package: Rc<RefCell<PackageJson>>,
  /// The path to the property that was linted
  pub property_path: String,
  /// The broken linting rule
  pub variant: FormatMismatchVariant,
}

#[derive(Debug)]
pub enum FormatMismatchVariant {
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
