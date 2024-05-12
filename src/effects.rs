use colored::*;

use crate::{group_selector::GroupSelector, instance_group::InstanceGroup};

pub struct Effects {}

impl Effects {
  ///
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
  ///
  pub fn on_ignored_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!(
      "{} {} {}",
      count,
      instance_group.name.dimmed(),
      "[Ignored]".dimmed()
    );
  }
  ///
  pub fn on_banned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }
  // @TODO: take instance: Instance
  ///
  pub fn on_banned_instance(&self, actual_specifier: &String, _instance_group: &InstanceGroup) {
    let icon = "✘".red();
    println!(
      "      {} {} {}",
      icon,
      actual_specifier.red(),
      "[Banned]".dimmed()
    );
  }
  ///
  pub fn on_invalid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }
  ///
  pub fn on_valid_pinned_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }
  ///
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
  ///
  pub fn on_valid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name);
  }
  ///
  pub fn on_invalid_same_range_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }
  ///
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
  ///
  pub fn on_valid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name);
  }
  ///
  pub fn on_invalid_snap_to_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }
  ///
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
  ///
  pub fn on_valid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    print_version_match(instance_group);
  }
  ///
  pub fn on_invalid_standard_instance_group(&self, instance_group: &InstanceGroup) {
    let count = render_count_column(instance_group.all.len());
    println!("{} {}", count, instance_group.name.red());
  }
  ///
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
  ///
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
  ///
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
  ///
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
