use colored::*;
use log::info;

use crate::{
  effects::{Effects, InstanceEvent},
  effects_lint::{lint_effects, render_count_column},
};

pub fn fix_effects(effect: Effects) -> () {
  match effect {
    Effects::PackagesLoaded(_, _, _) => {
      lint_effects(effect);
    }

    Effects::EnterVersionsAndRanges(_) => {
      lint_effects(effect);
    }
    Effects::EnterFormat(_) => {
      lint_effects(effect);
    }
    Effects::ExitCommand(state) => {
      if state.is_valid {
        info!("\n{} {}", "✓".green(), "complete");
      } else {
        // @TODO: when fixing and unfixable errors happen, explain them to the user
        info!("\n{} {}", "✘".red(), "some issues were not autofixable");
      }
    }

    Effects::PackagesMatchFormatting(valid_packages, config) => {}
    Effects::PackagesMismatchFormatting(invalid_packages, config, _state) => {
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

    Effects::GroupVisited(selector) => {
      lint_effects(effect);
    }

    Effects::DependencyIgnored(dependency) => {}
    Effects::DependencyBanned(dependency) => {}
    Effects::DependencyMatchesPinnedVersion(dependency) => {}
    Effects::DependencyMismatchesPinnedVersion(dependency) => {}
    Effects::DependencyMatchesRange(dependency) => {}
    Effects::DependencyMismatchesRange(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name.red());
    }
    Effects::DependencyMatchesSnapTo(dependency) => {}
    Effects::DependencyMismatchesSnapTo(dependency) => {}
    Effects::DependencyMatchesStandard(dependency) => {}
    Effects::DependencyMismatchesStandard(dependency) => {
      // show name above unsupported mismatches
      if !dependency.non_semver.is_empty() {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
    }

    Effects::InstanceBanned(event, _state) => {
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
    Effects::InstanceMismatchesPinnedVersion(event, _state) => {
      let pinned_specifier = &event.mismatches_with.0;
      set_every_instance_version_to(pinned_specifier.clone(), event);
    }
    Effects::InstanceMismatchesRange(event, state) => {
      info!(
        "      {} {} {} {} {}",
        "✘".red(),
        event.mismatches_with.0.red(),
        "falls outside".red(),
        event.target.0.red(),
        "[SameRangeMismatch]".dimmed()
      );
      state.is_valid = false;
    }
    Effects::InstanceMismatchesSnapTo(event, _state) => {
      let snapped_to_specifier = &event.mismatches_with.0;
      set_every_instance_version_to(snapped_to_specifier.clone(), event);
    }
    Effects::InstanceMismatchesLocalVersion(event, _state) => {
      let local_specifier = &event.mismatches_with.0;
      set_every_instance_version_to(local_specifier.clone(), event);
    }
    Effects::InstanceUnsupportedMismatch(event, state) => {
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
      state.is_valid = false;
    }
    Effects::InstanceMismatchesLowestVersion(event, _state) => {
      let lowest_specifier = &event.mismatches_with.0;
      set_every_instance_version_to(lowest_specifier.clone(), event);
    }
    Effects::InstanceMismatchesHighestVersion(event, _state) => {
      let highest_specifier = &event.mismatches_with.0;
      set_every_instance_version_to(highest_specifier.clone(), event);
    }
  };
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
