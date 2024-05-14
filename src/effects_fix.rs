use std::path::PathBuf;

use colored::*;

use crate::{
  effects::Effects,
  effects_lint::render_count_column,
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

  fn on_group(&self, _selector: &GroupSelector) {}

  // ===========================================================================
  // Instance Groups
  // ===========================================================================

  fn on_ignored_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_banned_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_valid_pinned_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_invalid_pinned_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_valid_same_range_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_invalid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_snap_to_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_invalid_snap_to_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_valid_standard_instance_group(&self, _instance_group: &InstanceGroup) {}

  fn on_invalid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    // show name above unsupported mismatches
    if !instance_group.non_semver.is_empty() {
      let count = render_count_column(instance_group.all.len());
      println!("{} {}", count, instance_group.name.red());
    }
  }

  // ===========================================================================
  // Instances
  // ===========================================================================

  fn on_banned_instance(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let (_, instances) = specifier;
    instances.iter().for_each(|instance| {
      if let Some(package) = packages.by_name.get_mut(&instance.package_name) {
        instance.remove_from(package);
      };
    });
  }

  fn on_pinned_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    if let Some(expected) = &instance_group.expected_version {
      let (_, instances) = specifier;
      set_every_instance_version_to(expected, instances, packages);
    }
  }

  fn on_same_range_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &InstancesBySpecifier,
    _instance_group: &InstanceGroup,
    _packages: &mut Packages,
  ) {
    println!(
      "      {} {} {} {} {}",
      "✘".red(),
      mismatches_with.0.red(),
      "falls outside".red(),
      specifier.0.red(),
      "[SameRangeMismatch]".dimmed()
    )
  }

  fn on_snap_to_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &Instance,
    _instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let expected = &mismatches_with.specifier;
    let (_, instances) = specifier;
    set_every_instance_version_to(expected, instances, packages);
  }

  fn on_local_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &Instance,
    _instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    let (_, instances) = specifier;
    let expected = &mismatches_with.specifier;
    set_every_instance_version_to(expected, instances, packages);
  }

  fn on_unsupported_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    _instance_group: &InstanceGroup,
    _packages: &mut Packages,
  ) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    println!(
      "      {} {} {} {} {}",
      icon,
      specifier.0.red(),
      arrow,
      "?".yellow(),
      "[UnsupportedMismatch]".dimmed()
    );
  }

  fn on_lowest_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    if let Some(expected) = &instance_group.expected_version {
      let (_, instances) = specifier;
      set_every_instance_version_to(expected, instances, packages);
    }
  }

  fn on_highest_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    packages: &mut Packages,
  ) {
    if let Some(expected) = &instance_group.expected_version {
      let (_, instances) = specifier;
      set_every_instance_version_to(expected, instances, packages);
    }
  }
}

fn set_every_instance_version_to(
  expected: &String,
  instances: &Vec<&Instance>,
  packages: &mut Packages,
) {
  instances.iter().for_each(|instance| {
    if let Some(package) = packages.by_name.get_mut(&instance.package_name) {
      instance.set_version(package, expected.clone());
    };
  });
}
