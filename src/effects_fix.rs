use colored::*;
use log::info;

use crate::{
  effects::{Effects, Event, MismatchEvent},
  effects_lint::render_count_column,
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
      Event::DependencyMatchesPinnedVersion(dependency) => {}
      Event::DependencyMismatchesPinnedVersion(dependency) => {}
      Event::DependencyMatchesRange(dependency) => {}
      Event::DependencyMismatchesRange(dependency) => {
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
      Event::InstanceMismatchesPinnedVersion(event) => {
        let pinned_specifier = &event.mismatches_with.0;
        set_every_instance_version_to(pinned_specifier.clone(), event);
      }
      Event::InstanceMismatchesRange(event) => {
        info!(
          "      {} {} {} {} {}",
          "✘".red(),
          event.mismatches_with.0.red(),
          "falls outside".red(),
          event.target.0.red(),
          "[SameRangeMismatch]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesSnapTo(event) => {
        let snapped_to_specifier = &event.mismatches_with.0;
        set_every_instance_version_to(snapped_to_specifier.clone(), event);
      }
      Event::InstanceMismatchesLocalVersion(event) => {
        let local_specifier = &event.mismatches_with.0;
        set_every_instance_version_to(local_specifier.clone(), event);
      }
      Event::InstanceUnsupportedMismatch(event) => {
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
        self.is_valid = false;
      }
      Event::InstanceMismatchesLowestVersion(event) => {
        let lowest_specifier = &event.mismatches_with.0;
        set_every_instance_version_to(lowest_specifier.clone(), event);
      }
      Event::InstanceMismatchesHighestVersion(event) => {
        let highest_specifier = &event.mismatches_with.0;
        set_every_instance_version_to(highest_specifier.clone(), event);
      }
    };
  }
}

fn set_every_instance_version_to(expected: String, event: &mut MismatchEvent) {
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
