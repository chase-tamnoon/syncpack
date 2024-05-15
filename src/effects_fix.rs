use std::path::PathBuf;

use colored::*;

use crate::{
  effects::{Effects, InstanceEvent},
  effects_lint::render_count_column,
  group_selector::GroupSelector,
  instance::Instance,
  instance_group::InstanceGroup,
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

  fn on_banned_instance(&self, event: InstanceEvent) {
    let (_, target_instances) = event.target;
    target_instances.iter().for_each(|instance| {
      if let Some(package) = event.packages.by_name.get_mut(&instance.package_name) {
        instance.remove_from(package);
      };
    });
  }

  fn on_pinned_version_mismatch(&self, event: InstanceEvent) {
    if let Some(expected) = &event.instance_group.expected_version {
      let (_, target_instances) = event.target;
      set_every_instance_version_to(expected, &target_instances, event.packages);
    }
  }

  fn on_same_range_mismatch(&self, event: InstanceEvent) {
    println!(
      "      {} {} {} {} {}",
      "✘".red(),
      event.mismatches_with.0.red(),
      "falls outside".red(),
      event.target.0.red(),
      "[SameRangeMismatch]".dimmed()
    )
  }

  fn on_snap_to_mismatch(&self, event: InstanceEvent) {
    let (_, target_instances) = event.target;
    let (_, mismatches_with) = &event.mismatches_with;
    // (there is only one member in this vec)
    mismatches_with.iter().for_each(|snapped_to_instance| {
      let expected = &snapped_to_instance.specifier;
      set_every_instance_version_to(&expected, &target_instances, event.packages);
    });
  }

  fn on_local_version_mismatch(&self, event: InstanceEvent) {
    let (_, target_instances) = event.target;
    let (_, mismatches_with) = &event.mismatches_with;
    // (there is only one member in this vec)
    mismatches_with.iter().for_each(|local_instance| {
      let expected = &local_instance.specifier;
      set_every_instance_version_to(&expected, &target_instances, event.packages);
    });
  }

  fn on_unsupported_mismatch(&self, event: InstanceEvent) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    println!(
      "      {} {} {} {} {}",
      icon,
      event.target.0.red(),
      arrow,
      "?".yellow(),
      "[UnsupportedMismatch]".dimmed()
    );
  }

  fn on_lowest_version_mismatch(&self, event: InstanceEvent) {
    if let Some(expected) = &event.instance_group.expected_version {
      let (_, target_instances) = event.target;
      set_every_instance_version_to(expected, &target_instances, event.packages);
    }
  }

  fn on_highest_version_mismatch(&self, event: InstanceEvent) {
    if let Some(expected) = &event.instance_group.expected_version {
      let (_, target_instances) = event.target;
      set_every_instance_version_to(expected, &target_instances, event.packages);
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
