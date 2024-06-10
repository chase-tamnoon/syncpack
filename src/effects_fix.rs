use colored::*;
use log::info;

use crate::{
  dependency::InstancesById,
  effects::{Effects, Event},
  effects_lint::render_count_column,
  packages::Packages,
  specifier::Specifier,
};

/// The implementation of the `fix` command's side effects
pub struct FixEffects {
  pub is_valid: bool,
}

impl FixEffects {
  pub fn new() -> Self {
    Self { is_valid: true }
  }
}

impl Effects for FixEffects {
  fn on(&mut self, event: Event) -> () {
    match event {
      Event::PackagesLoaded(_, _) => {
        // @TODO
        // lint_effects(event);
      }

      Event::EnterVersionsAndRanges(_) => {
        // @TODO
        // lint_effects(event);
      }
      Event::EnterFormat(_) => {
        // @TODO
        // lint_effects(event);
      }
      Event::ExitCommand => {
        if self.is_valid {
          info!("\n{} {}", "✓".green(), "complete");
        } else {
          // @TODO: when fixing and unfixable errors happen, explain them to the user
          info!("\n{} {}", "✘".red(), "some issues were not autofixable");
        }
      }

      Event::PackagesMatchFormatting(valid_packages, config) => {}
      Event::PackagesMismatchFormatting(invalid_packages, config) => {
        info!(
          "{} {}",
          render_count_column(invalid_packages.len()),
          "fixed formatting".green()
        );
        invalid_packages.iter().for_each(|package| {
          info!(
            "      {} {}",
            "✓".green(),
            package.get_relative_file_path(&config.cwd)
          );
        });
      }

      Event::GroupVisited(selector) => {
        // @TODO
        // lint_effects(event);
      }

      Event::DependencyIgnored(dependency) => {}
      Event::DependencyBanned(dependency) => {}
      Event::DependencyMatchesWithRange(dependency) => {}
      Event::DependencyMismatchesWithRange(_) => {
        println!("@TODO: fix Event::DependencyMismatchesWithRange");
      }
      Event::DependencyMatchesPinnedVersion(dependency) => {}
      Event::DependencyMismatchesPinnedVersion(dependency) => {}
      Event::DependencyMatchesSameRange(dependency) => {}
      Event::DependencyMismatchesSameRange(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
      Event::DependencyMatchesSnapTo(dependency) => {}
      Event::DependencyMismatchesSnapTo(dependency) => {}
      Event::DependencyMatchesStandard(dependency) => {}
      Event::DependencyMismatchesStandard(dependency) => {
        // show name above unsupported mismatches
        if !dependency.non_semver.is_empty() {
          let count = render_count_column(dependency.all.len());
          info!("{} {}", count, dependency.name.red());
        }
      }

      Event::InstanceMatchesStandard(_) => {
        //
      }
      Event::InstanceBanned(event) => {
        let target_instance = event.instances_by_id.get_mut(&event.instance_id).unwrap();
        let package = event
          .packages
          .by_name
          .get_mut(&target_instance.package_name)
          .unwrap();
        target_instance.remove_from(package);
      }
      Event::InstanceMismatchesSemverRange(event) => {
        set_instance_version_to(
          event.instances_by_id,
          event.packages,
          &event.instance_id,
          &event.expected_specifier,
        );
      }
      Event::InstanceMismatchesPinnedVersion(event) => {
        set_instance_version_to(
          event.instances_by_id,
          event.packages,
          &event.instance_id,
          &event.expected_specifier,
        );
      }
      Event::InstanceMismatchesRange(event) => {
        info!(
          "      {} {} {} {} {}",
          "✘".red(),
          event.specifier_outside_range.unwrap().red(),
          "falls outside".red(),
          event.specifier.unwrap().red(),
          "[SameRangeMismatch]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesSnapTo(event) => {
        set_instance_version_to(
          event.instances_by_id,
          event.packages,
          &event.instance_id,
          &event.expected_specifier,
        );
      }
      Event::InstanceMismatchCorruptsLocalVersion(event) => {
        let icon = "!".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.actual_specifier.unwrap().green(),
          arrow,
          event.expected_specifier.unwrap().red(),
          "[RejectedLocalMismatch]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesLocalVersion(event) => {
        set_instance_version_to(
          event.instances_by_id,
          event.packages,
          &event.instance_id,
          &event.expected_specifier,
        );
      }
      Event::InstanceUnsupportedMismatch(event) => {
        let icon = "✘".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.specifier.unwrap().red(),
          arrow,
          "?".yellow(),
          "[UnsupportedMismatch]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesLowestVersion(event) => {
        set_instance_version_to(
          event.instances_by_id,
          event.packages,
          &event.instance_id,
          &event.expected_specifier,
        );
      }
      Event::InstanceMismatchesHighestVersion(event) => {
        set_instance_version_to(
          event.instances_by_id,
          event.packages,
          &event.instance_id,
          &event.expected_specifier,
        );
      }
    };
  }
}

fn set_instance_version_to(
  instances_by_id: &mut InstancesById,
  packages: &mut Packages,
  instance_id: &String,
  expected_specifier: &Specifier,
) {
  let target_instance = instances_by_id.get_mut(instance_id).unwrap();
  let package = packages
    .by_name
    .get_mut(&target_instance.package_name)
    .unwrap();
  target_instance.set_specifier(package, expected_specifier);
}
