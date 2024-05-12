use std::path::PathBuf;

use crate::{
  effects::Effects,
  group_selector::GroupSelector,
  instance::Instance,
  instance_group::{InstanceGroup, InstancesBySpecifier},
  package_json::PackageJson,
};

pub struct FixEffects {}

impl Effects for FixEffects {
  fn on_begin_format(&self) {}

  fn on_skip_ranges_and_versions(&self) {}

  fn on_begin_ranges_and_versions(&self) {}

  fn on_begin_ranges_only(&self) {}

  fn on_begin_versions_only(&self) {}

  // ===========================================================================
  // Formatting
  // ===========================================================================

  fn on_formatted_packages(&self, valid_packages: &Vec<&PackageJson>, _cwd: &PathBuf) {}

  fn on_unformatted_packages(&self, invalid_packages: &Vec<&PackageJson>, cwd: &PathBuf) {}

  // ===========================================================================
  // Version Groups
  // ===========================================================================

  fn on_group(&self, selector: &GroupSelector) {}

  // ===========================================================================
  // Instance Groups
  // ===========================================================================

  fn on_ignored_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_banned_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_valid_pinned_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_invalid_pinned_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_valid_same_range_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_invalid_same_range_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_valid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_invalid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_valid_standard_instance_group(&self, instance_group: &InstanceGroup) {}

  fn on_invalid_standard_instance_group(&self, instance_group: &InstanceGroup) {}

  // ===========================================================================
  // Instances
  // ===========================================================================

  fn on_banned_instance(&self, specifier: &InstancesBySpecifier, instance_group: &InstanceGroup) {}

  fn on_pinned_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
  ) {
  }

  fn on_same_range_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
  ) {
  }

  fn on_snap_to_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &Instance,
    instance_group: &InstanceGroup,
  ) {
  }

  fn on_local_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &Instance,
    instance_group: &InstanceGroup,
  ) {
  }

  fn on_unsupported_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
  ) {
  }

  fn on_lowest_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
  ) {
  }

  fn on_highest_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
  ) {
  }
}
