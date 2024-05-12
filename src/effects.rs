use std::path::PathBuf;

use colored::*;

use crate::{
  group_selector::GroupSelector, instance_group::InstanceGroup, package_json::PackageJson,
};

pub struct Effects {}

impl Effects {
  // ===========================================================================
  // Enabled Tasks
  // ===========================================================================

  /// Syncpack is about to lint or fix formatting
  pub fn on_begin_format(&self) {
    println!("{}", "= FORMATTING".yellow());
  }

  /// Syncpack will not lint or fix semver ranges or versions
  pub fn on_skip_ranges_and_versions(&self) {}

  /// Syncpack is about to lint or fix both semver ranges and versions
  pub fn on_begin_ranges_and_versions(&self) {
    println!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".yellow());
  }

  /// Syncpack is about to lint or fix semver ranges only
  pub fn on_begin_ranges_only(&self) {
    println!("{}", "= SEMVER RANGES".yellow());
  }

  /// Syncpack is about to lint or fix version mismatches only
  pub fn on_begin_versions_only(&self) {
    println!("{}", "= VERSION MISMATCHES".yellow());
  }

  // ===========================================================================
  // Formatting
  // ===========================================================================

  /// Linting/fixing of formatting has completed and these packages were valid
  pub fn on_formatted_packages(&self, valid_packages: &Vec<&PackageJson>, _cwd: &PathBuf) {
    if !valid_packages.is_empty() {
      println!(
        "{} {} valid formatting",
        render_count_column(valid_packages.len()),
        "✓".green()
      );
    }
  }

  /// Linting/fixing of formatting has completed and these packages were
  /// initially invalid. In the case of fixing, they are now valid but were
  /// invalid beforehand.
  pub fn on_unformatted_packages(&self, invalid_packages: &Vec<&PackageJson>, cwd: &PathBuf) {
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
  // Version Groups
  // ===========================================================================

  /// A version/semver group is next to be processed
  pub fn on_group(&self, selector: &GroupSelector) {
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

  /// An instance group in an ignored version group has been found
  pub fn on_ignored_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!(
      "{} {} {}",
      count,
      instance_group.name.dimmed(),
      "[Ignored]".dimmed()
    );
  }

  /// An instance group in a banned version group has been found
  pub fn on_banned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  /// An instance in a banned version group has been found
  pub fn on_banned_instance(&self, actual_specifier: &String, _instance_group: &InstanceGroup) {
    // @TODO: take instance: Instance
    let icon = "✘".red();
    println!(
      "      {} {} {}",
      icon,
      actual_specifier.red(),
      "[Banned]".dimmed()
    );
  }

  /// An instance group in a pinned version group has been found where all
  /// instances are valid
  pub fn on_valid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }

  /// An instance group in a pinned version group has been found which has one
  /// or more instances with versions that are not the same as the `.pinVersion`
  pub fn on_invalid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  /// An instance in a pinned version group has been found whose version is not
  /// the same as the `.pinVersion`
  pub fn on_pinned_version_mismatch(
    &self,
    actual_specifier: &String,
    instance_group: &InstanceGroup,
  ) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      actual_specifier.red(),
      arrow,
      expected.green(),
      "[PinnedMismatch]".dimmed()
    );
  }

  /// An instance group in a same range version group has been found where all
  /// instances are valid
  pub fn on_valid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name);
  }

  /// An instance group in a same range version group has been found which has
  /// one or more instances with versions that are not a semver range which
  /// satisfies all of the other semver ranges in the group
  pub fn on_invalid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  /// An instance in a same range version group has been found which has a
  /// version which is not a semver range which satisfies all of the other
  /// semver ranges in the group
  pub fn on_same_range_mismatch(
    &self,
    mismatching_ranges: &(String, String),
    _instance_group: &InstanceGroup,
  ) {
    let (a, b) = mismatching_ranges;
    println!(
      "      {} {} {} {} {}",
      "✘".red(),
      b.red(),
      "falls outside".red(),
      a.red(),
      "[SameRangeMismatch]".dimmed()
    )
  }

  /// An instance group in a snapped to version group has been found where all
  /// instances are valid
  pub fn on_valid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name);
  }

  /// An instance group in a snapped to version group has been found which has
  /// one or more instances with versions that are not the same as those used
  /// by the packages named in the `.snapTo` config array
  pub fn on_invalid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  /// An instance in a snapped to version group has been found which has a
  /// version that is not the same as those used by the packages named in the
  /// `.snapTo` config array
  pub fn on_snap_to_mismatch(
    &self,
    mismatching_versions: &(String, String),
    _instance_group: &InstanceGroup,
  ) {
    let (actual_specifier, expected_specifier) = mismatching_versions;
    let icon = "✘".red();
    let arrow = "→".dimmed();
    println!(
      "      {} {} {} {} {}",
      icon,
      actual_specifier.red(),
      arrow,
      expected_specifier.green(),
      "[SnappedToMismatch]".dimmed()
    );
  }

  /// An instance group in a standard version group has been found where all
  /// instances are valid
  pub fn on_valid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }

  /// An instance group in a standard version group has been found which has
  /// one or more instances with versions that are not the same as the others
  pub fn on_invalid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }

  /// An instance in a standard version group has been found which is a
  /// dependency developed in this repo, its version does not match the
  /// `.version` property of the package.json file for this package in the repo
  pub fn on_local_version_mismatch(&self, instance_group: &InstanceGroup, actual: &String) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      actual.red(),
      arrow,
      expected.green(),
      "[LocalPackageMismatch]".dimmed()
    );
  }

  /// An instance in a standard version group has been found which has a version
  /// which is not identical to the others, but not all of the instances have
  /// valid or supported version specifiers, so it's impossible to know which
  /// should be preferred
  pub fn on_unsupported_mismatch(&self, actual: &String, _instance_group: &InstanceGroup) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    println!(
      "      {} {} {} {} {}",
      icon,
      actual.red(),
      arrow,
      "?".yellow(),
      "[UnsupportedMismatch]".dimmed()
    );
  }

  /// An instance in a standard version group has been found which has a semver
  /// version which is higher than the lowest semver version in the group, and
  /// `.preferVersion` is set to `lowestSemver`
  pub fn on_lowest_version_mismatch(&self, actual: &String, instance_group: &InstanceGroup) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      actual.red(),
      arrow,
      expected.green(),
      "[LowestSemverMismatch]".dimmed()
    );
  }

  /// An instance in a standard version group has been found which has a semver
  /// version which is lower than the highest semver version in the group, and
  /// `.preferVersion` is set to `highestSemver`
  pub fn on_highest_version_mismatch(&self, actual: &String, instance_group: &InstanceGroup) {
    let icon = "✘".red();
    let arrow = "→".dimmed();
    let expected = instance_group.expected_version.as_ref().unwrap();
    println!(
      "      {} {} {} {} {}",
      icon,
      actual.red(),
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
  let version = instance_group.unique_specifiers.iter().next().unwrap();
  println!("{} {} {}", count, instance_group.name, &version.dimmed());
}
