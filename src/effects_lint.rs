use std::path::PathBuf;

use colored::*;

use crate::{
  effects::Effects,
  group_selector::GroupSelector,
  instance::Instance,
  instance_group::{InstanceGroup, InstancesBySpecifier},
  package_json::{PackageJson, Packages},
};

pub struct LintEffects {}

impl Effects for LintEffects {
  // ===========================================================================
  // Enabled Tasks
  // ===========================================================================

  fn on_begin_format(&self) {
    println!("{}", "= FORMATTING".yellow());
  }

  fn on_skip_ranges_and_versions(&self) {}

  fn on_begin_ranges_and_versions(&self) {
    println!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".yellow());
  }

  fn on_begin_ranges_only(&self) {
    println!("{}", "= SEMVER RANGES".yellow());
  }

  fn on_begin_versions_only(&self) {
    println!("{}", "= VERSION MISMATCHES".yellow());
  }

  // ===========================================================================
  // Formatting
  // ===========================================================================

  fn on_formatted_packages(&self, valid_packages: &Vec<&PackageJson>, _cwd: &PathBuf) {
    if !valid_packages.is_empty() {
      println!(
        "{} {} valid formatting",
        render_count_column(valid_packages.len()),
        "✓".green()
      );
    }
  }

  fn on_unformatted_packages(&self, invalid_packages: &Vec<&PackageJson>, cwd: &PathBuf) {
    if !invalid_packages.is_empty() {
      println!(
        "{} {}",
        render_count_column(invalid_packages.len()),
        "invalid formatting".red()
      );
      invalid_packages.iter().for_each(|package| {
        println!(
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
    println!("{}", full_header.blue());
  }

  // ===========================================================================
  // Instance Groups
  // ===========================================================================

  fn on_ignored_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!(
      "{} {} {}",
      count,
      instance_group.name.dimmed(),
      "[Ignored]".dimmed()
    );
  }

  fn on_banned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }

  fn on_invalid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name);
  }

  fn on_invalid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name);
  }

  fn on_invalid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  fn on_valid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }

  fn on_invalid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  // ===========================================================================
  // Instances
  // ===========================================================================

  fn on_banned_instance(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    _packages: &mut Packages,
  ) {
    let icon = "✘".red();
    println!(
      "      {} {} {}",
      icon,
      specifier.0.red(),
      "[Banned]".dimmed()
    );
  }

  fn on_pinned_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    _packages: &mut Packages,
  ) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      specifier.0.red(),
      arrow,
      expected.green(),
      "[PinnedMismatch]".dimmed()
    );
  }

  fn on_same_range_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
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
    instance_group: &InstanceGroup,
    _packages: &mut Packages,
  ) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    println!(
      "      {} {} {} {} {}",
      icon,
      specifier.0.red(),
      arrow,
      mismatches_with.specifier.green(),
      "[SnappedToMismatch]".dimmed()
    );
  }

  fn on_local_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    mismatches_with: &Instance,
    instance_group: &InstanceGroup,
    _packages: &mut Packages,
  ) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      specifier.0.red(),
      arrow,
      expected.green(),
      "[LocalPackageMismatch]".dimmed()
    );
  }

  fn on_unsupported_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
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
    _packages: &mut Packages,
  ) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      specifier.0.red(),
      arrow,
      expected.green(),
      "[LowestSemverMismatch]".dimmed()
    );
  }

  fn on_highest_version_mismatch(
    &self,
    specifier: &InstancesBySpecifier,
    instance_group: &InstanceGroup,
    _packages: &mut Packages,
  ) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      specifier.0.red(),
      arrow,
      expected.green(),
      "[HighestSemverMismatch]".dimmed()
    );
  }
}

/// Return a right aligned column of a count of instances
/// Example "    38x"
fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

fn print_version_match(instance_group: &InstanceGroup<'_>) {
  let count = render_count_column(instance_group.all.len());
  let (version, _) = instance_group.by_specifier.iter().next().unwrap();
  println!("{} {} {}", count, instance_group.name, &version.dimmed());
}
