use colored::*;
use log::info;
use std::path::PathBuf;

use crate::{
  effects::{Effects, InstanceEvent},
  group_selector::GroupSelector,
  instance_group::InstanceGroup,
  package_json::PackageJson,
};

pub struct LintEffects {}

impl Effects for LintEffects {
  // ===========================================================================
  // Enabled Tasks
  // ===========================================================================

  fn on_begin_format(&self) {
    info!("{}", "= FORMATTING".dimmed());
  }

  fn on_skip_ranges_and_versions(&self) {}

  fn on_begin_ranges_and_versions(&self) {
    info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
  }

  fn on_begin_ranges_only(&self) {
    info!("{}", "= SEMVER RANGES".dimmed());
  }

  fn on_begin_versions_only(&self) {
    info!("{}", "= VERSION MISMATCHES".dimmed());
  }

  // ===========================================================================
  // Formatting
  // ===========================================================================

  fn on_formatted_packages(&self, valid_packages: &Vec<&PackageJson>, _cwd: &PathBuf) {
    if !valid_packages.is_empty() {
      info!(
        "{} {} valid formatting",
        render_count_column(valid_packages.len()),
        "✓".green()
      );
    }
  }

  fn on_unformatted_packages(&self, invalid_packages: &Vec<&PackageJson>, cwd: &PathBuf) {
    if !invalid_packages.is_empty() {
      info!(
        "{} {}",
        render_count_column(invalid_packages.len()),
        "invalid formatting".red()
      );
      invalid_packages.iter().for_each(|package| {
        info!(
          "      {} {}",
          "✘".red(),
          package.get_relative_file_path(&cwd).red()
        );
      });
    }
  }

  // ===========================================================================
  // Version/Semver Groups
  // ===========================================================================

  fn on_group(&self, selector: &GroupSelector) {
    let print_width = 80;
    let header = format!("= {} ", selector.label);
    let divider = if header.len() < print_width {
      "=".repeat(print_width - header.len())
    } else {
      "".to_string()
    };
    let full_header = format!("{}{}", header, divider);
    info!("{}", full_header.blue());
  }

  // ===========================================================================
  // Instance Groups
  // ===========================================================================

  fn on_ignored_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!(
      "{} {} {}",
      count,
      instance_group.name.dimmed(),
      "[Ignored]".dimmed()
    );
  }

  fn on_banned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }

  fn on_invalid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!("{} {}", count, instance_group.name);
  }

  fn on_invalid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!("{} {}", count, instance_group.name);
  }

  fn on_invalid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }

  fn on_invalid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    info!("{} {}", count, instance_group.name.red());
  }

  // ===========================================================================
  // Instances
  // ===========================================================================

  fn on_banned_instance(&self, event: &mut InstanceEvent) {
    let icon = "✘".red();
    info!(
      "      {} {} {}",
      icon,
      event.target.0.red(),
      "[Banned]".dimmed()
    );
  }

  fn on_pinned_version_mismatch(&self, event: &mut InstanceEvent) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = event.instance_group.expected_version.as_ref().unwrap();
    info!(
      "      {} {} {} {} {}",
      icon,
      event.target.0.red(),
      arrow,
      expected.green(),
      "[PinnedMismatch]".dimmed()
    );
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
    let (expected, mismatches_with) = &event.mismatches_with;
    // (there is only one member in this vec)
    mismatches_with.iter().for_each(|snapped_to_instance_id| {
      let icon = "✘".red();
      let arrow = "→".dimmed();
      info!(
        "      {} {} {} {} {}",
        icon,
        event.target.0.red(),
        arrow,
        expected.green(),
        "[SnappedToMismatch]".dimmed()
      );
    });
  }

  fn on_local_version_mismatch(&self, event: &mut InstanceEvent) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = event.instance_group.expected_version.as_ref().unwrap();
    info!(
      "      {} {} {} {} {}",
      icon,
      event.target.0.red(),
      arrow,
      expected.green(),
      "[LocalPackageMismatch]".dimmed()
    );
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
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = event.instance_group.expected_version.as_ref().unwrap();
    info!(
      "      {} {} {} {} {}",
      icon,
      event.target.0.red(),
      arrow,
      expected.green(),
      "[LowestSemverMismatch]".dimmed()
    );
  }

  fn on_highest_version_mismatch(&self, event: &mut InstanceEvent) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = event.instance_group.expected_version.as_ref().unwrap();
    info!(
      "      {} {} {} {} {}",
      icon,
      event.target.0.red(),
      arrow,
      expected.green(),
      "[HighestSemverMismatch]".dimmed()
    );
  }
}

/// Return a right aligned column of a count of instances
/// Example "    38x"
pub fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

fn print_version_match(instance_group: &InstanceGroup) {
  let count = render_count_column(instance_group.all.len());
  let (version, _) = instance_group.by_specifier.iter().next().unwrap();
  info!("{} {} {}", count, instance_group.name, &version.dimmed());
}
