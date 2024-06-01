use colored::*;
use log::info;
use std::process;

use crate::{
  config::Config,
  dependency::Dependency,
  effects::{Effects, InstanceEvent},
  effects_lint::render_count_column,
  group_selector::GroupSelector,
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
  // Tear Down
  // ===========================================================================

  /// Linting/fixing has completed
  fn on_complete(&self, is_valid: bool) {
    if is_valid {
      info!("\n{} {}", "✓".green(), "valid");
      process::exit(0);
    } else {
      // @TODO: when fixing and unfixable errors happen, explain them to the user
      info!("\n{} {}", "✘".red(), "invalid");
      process::exit(1);
    }
  }

  // ===========================================================================
  // Formatting
  // ===========================================================================

  fn on_formatted_packages(&self, _valid_packages: &Vec<&PackageJson>, _config: &Config) {}
  fn on_unformatted_packages(&self, _invalid_packages: &Vec<&PackageJson>, _config: &Config) {}

  // ===========================================================================
  // Version Groups
  // ===========================================================================

  fn on_group(&self, _selector: &GroupSelector) {}

  // ===========================================================================
  // Instance Groups
  // ===========================================================================

  fn on_ignored_dependency(&self, _dependency: &Dependency) {}
  fn on_banned_dependency(&self, _dependency: &Dependency) {}
  fn on_valid_pinned_dependency(&self, _dependency: &Dependency) {}
  fn on_invalid_pinned_dependency(&self, _dependency: &Dependency) {}
  fn on_valid_same_range_dependency(&self, _dependency: &Dependency) {}

  fn on_invalid_same_range_dependency(&self, dependency: &Dependency) {
    let count = render_count_column(dependency.all.len());
    info!("{} {}", count, dependency.name.red());
  }

  fn on_valid_snap_to_dependency(&self, _dependency: &Dependency) {}
  fn on_invalid_snap_to_dependency(&self, _dependency: &Dependency) {}
  fn on_valid_standard_dependency(&self, _dependency: &Dependency) {}

  fn on_invalid_standard_dependency(&self, dependency: &Dependency) {
    // show name above unsupported mismatches
    if !dependency.non_semver.is_empty() {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name.red());
    }
  }

  // ===========================================================================
  // Instances
  // ===========================================================================

  fn on_banned_instance(&self, event: &mut InstanceEvent) {
    let target_instance_ids = event.target.1.clone();
    target_instance_ids.iter().for_each(|instance_id| {
      if let Some(target_instance) = event.instances_by_id.get_mut(instance_id) {
        if let Some(package) = event
          .packages
          .by_name
          .get_mut(&target_instance.package_name)
        {
          target_instance.remove_from(package);
        }
      };
    });
  }

  fn on_pinned_version_mismatch(&self, event: &mut InstanceEvent) {
    let pinned_specifier = &event.mismatches_with.0;
    set_every_instance_version_to(pinned_specifier.clone(), event);
  }

  fn on_same_range_mismatch(&self, event: &mut InstanceEvent) {
    info!(
      "      {} {} {} {} {}",
      "✘".red(),
      event.mismatches_with.0.red(),
      "falls outside".red(),
      event.target.0.red(),
      "[SameRangeMismatch]".dimmed()
    )
  }

  fn on_snap_to_mismatch(&self, event: &mut InstanceEvent) {
    let snapped_to_specifier = &event.mismatches_with.0;
    set_every_instance_version_to(snapped_to_specifier.clone(), event);
  }

  fn on_local_version_mismatch(&self, event: &mut InstanceEvent) {
    let local_specifier = &event.mismatches_with.0;
    set_every_instance_version_to(local_specifier.clone(), event);
  }

  fn on_unsupported_mismatch(&self, event: &mut InstanceEvent) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    info!(
      "      {} {} {} {} {}",
      icon,
      event.target.0.red(),
      arrow,
      "?".yellow(),
      "[UnsupportedMismatch]".dimmed()
    );
  }

  fn on_lowest_version_mismatch(&self, event: &mut InstanceEvent) {
    let lowest_specifier = &event.mismatches_with.0;
    set_every_instance_version_to(lowest_specifier.clone(), event);
  }

  fn on_highest_version_mismatch(&self, event: &mut InstanceEvent) {
    let highest_specifier = &event.mismatches_with.0;
    set_every_instance_version_to(highest_specifier.clone(), event);
  }
}

fn set_every_instance_version_to(expected: String, event: &mut InstanceEvent) {
  let target_instance_ids = event.target.1.clone();
  target_instance_ids.iter().for_each(|instance_id| {
    if let Some(target_instance) = event.instances_by_id.get_mut(instance_id) {
      if let Some(package) = event
        .packages
        .by_name
        .get_mut(&target_instance.package_name)
      {
        target_instance.set_version(package, expected.clone());
      }
    };
  });
}
