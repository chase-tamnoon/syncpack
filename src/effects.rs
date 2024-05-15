use std::path::PathBuf;

use crate::{
  group_selector::GroupSelector,
  instance_group::{InstanceGroup, InstancesBySpecifier},
  package_json::{PackageJson, Packages},
};

pub struct InstanceEvent<'a> {
  pub instance_group: &'a InstanceGroup,
  /// when same range mismatch: the range which was not satisfied by `invalid_range` (there may be others which this range does not match, they will be reported separately)
  /// when snapped to mismatch: the snapped to instance which should be matched
  /// when local mismatch: the local instance which should be matched
  pub mismatches_with: InstancesBySpecifier,
  /// all packages in the workspace
  pub packages: &'a mut Packages,
  /// when same range mismatch: the range which was found not to satisfy another
  /// when snapped to mismatch: the specifier which does not match the snapped to instance
  /// when local mismatch: the specifier which does not match the local instance
  pub target: InstancesBySpecifier,
}

pub trait Effects {
  // ===========================================================================
  // Enabled Tasks
  // ===========================================================================

  /// Syncpack is about to lint or fix formatting
  fn on_begin_format(&self);

  /// Syncpack will not lint or fix semver ranges or versions
  fn on_skip_ranges_and_versions(&self);

  /// Syncpack is about to lint or fix both semver ranges and versions
  fn on_begin_ranges_and_versions(&self);

  /// Syncpack is about to lint or fix semver ranges only
  fn on_begin_ranges_only(&self);

  /// Syncpack is about to lint or fix version mismatches only
  fn on_begin_versions_only(&self);

  // ===========================================================================
  // Formatting
  // ===========================================================================

  /// Linting/fixing of formatting has completed and these packages were valid
  fn on_formatted_packages(&self, valid_packages: &Vec<&PackageJson>, _cwd: &PathBuf);

  /// Linting/fixing of formatting has completed and these packages were
  /// initially invalid. In the case of fixing, they are now valid but were
  /// invalid beforehand.
  fn on_unformatted_packages(&self, invalid_packages: &Vec<&PackageJson>, cwd: &PathBuf);

  // ===========================================================================
  // Version/Semver Groups
  // ===========================================================================

  /// A version/semver group is next to be processed
  fn on_group(&self, selector: &GroupSelector);

  // ===========================================================================
  // Instance Groups
  // ===========================================================================

  /// An instance group in an ignored version group has been found
  fn on_ignored_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a banned version group has been found
  fn on_banned_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a pinned version group has been found where all
  /// instances are valid
  fn on_valid_pinned_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a pinned version group has been found which has one
  /// or more instances with versions that are not the same as the `.pinVersion`
  fn on_invalid_pinned_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a same range version group has been found where all
  /// instances are valid
  fn on_valid_same_range_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a same range version group has been found which has
  /// one or more instances with versions that are not a semver range which
  /// satisfies all of the other semver ranges in the group
  fn on_invalid_same_range_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a snapped to version group has been found where all
  /// instances are valid
  fn on_valid_snap_to_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a snapped to version group has been found which has
  /// one or more instances with versions that are not the same as those used
  /// by the packages named in the `.snapTo` config array
  fn on_invalid_snap_to_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a standard version group has been found where all
  /// instances are valid
  fn on_valid_standard_instance_group(&self, instance_group: &InstanceGroup);

  /// An instance group in a standard version group has been found which has
  /// one or more instances with versions that are not the same as the others
  fn on_invalid_standard_instance_group(&self, instance_group: &InstanceGroup);

  // ===========================================================================
  // Instances
  // ===========================================================================

  /// An instance in a banned version group has been found
  fn on_banned_instance(&self, event: InstanceEvent);

  /// An instance in a pinned version group has been found whose version is not
  /// the same as the `.pinVersion`
  fn on_pinned_version_mismatch(&self, event: InstanceEvent);

  /// An instance in a same range version group has been found which has a
  /// version which is not a semver range which satisfies all of the other
  /// semver ranges in the group
  fn on_same_range_mismatch(&self, event: InstanceEvent);

  /// An instance in a snapped to version group has been found which has a
  /// version that is not the same as those used by the packages named in the
  /// `.snapTo` config array
  fn on_snap_to_mismatch(&self, event: InstanceEvent);

  /// An instance in a standard version group has been found which is a
  /// dependency developed in this repo, its version does not match the
  /// `.version` property of the package.json file for this package in the repo
  fn on_local_version_mismatch(&self, event: InstanceEvent);

  /// An instance in a standard version group has been found which has a version
  /// which is not identical to the others, but not all of the instances have
  /// valid or supported version specifiers, so it's impossible to know which
  /// should be preferred
  fn on_unsupported_mismatch(&self, event: InstanceEvent);

  /// An instance in a standard version group has been found which has a semver
  /// version which is higher than the lowest semver version in the group, and
  /// `.preferVersion` is set to `lowestSemver`
  fn on_lowest_version_mismatch(&self, event: InstanceEvent);

  /// An instance in a standard version group has been found which has a semver
  /// version which is lower than the highest semver version in the group, and
  /// `.preferVersion` is set to `highestSemver`
  fn on_highest_version_mismatch(&self, event: InstanceEvent);
}
