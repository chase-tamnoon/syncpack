use std::path::PathBuf;

use colored::*;

use crate::{
  effects::Effects,
  group_selector::GroupSelector,
  instance::Instance,
  instance_group::{InstanceGroup, InstancesBySpecifier},
  package_json::{PackageJson, Packages},
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

  fn on_banned_instance(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    println!("@TODO: Implement on_banned_instance");
  }

  fn on_pinned_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let (_, instances) = specifier;
    set_all_instances_to_expected_version(instance_group, instances, packages);
  }

  fn on_same_range_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let name = &instance_group.name;
    let message = format!("{}", "SameRangeMismatch is not auto-fixable");
    println!("{}", message.dimmed());
  }

  fn on_snap_to_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &Instance,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    println!("@TODO: Implement on_snap_to_mismatch");
  }

  fn on_local_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &Instance,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let (_, instances) = specifier;
    set_all_instances_to_expected_version(instance_group, instances, packages);
  }

  fn on_unsupported_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let name = &instance_group.name;
    let message = format!("{}", "UnsupportedMismatch is not auto-fixable");
    println!("{}", message.dimmed());
  }

  fn on_lowest_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let (_, instances) = specifier;
    set_all_instances_to_expected_version(instance_group, instances, packages);
  }

  fn on_highest_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let (_, instances) = specifier;
    set_all_instances_to_expected_version(instance_group, instances, packages);
  }
}

fn set_all_instances_to_expected_version(
  instance_group: &InstanceGroup,
  instances: &Vec<&Instance>,
  packages: &mut Packages,
) {
  if let Some(expected) = &instance_group.expected_version {
    instances.iter().for_each(|instance| {
      if let Some(package) = packages.by_name.get_mut(&instance.package_name) {
        instance.set_version(package, expected.clone());
      };
    });
  }
}
