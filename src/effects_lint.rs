use colored::*;
use log::info;
use std::process;

use crate::{dependency::Dependency, effects::Effects};

pub fn lint_effects(effect: Effects) -> () {
  match effect {
    Effects::PackagesLoaded(config, packages) => {
      if packages.all_names.is_empty() {
        info!("\n{} {}", "✘".red(), "No packages found");
        process::exit(1);
      }
    }

    Effects::EnterVersionsAndRanges(config) => {
      match (config.cli.options.ranges, config.cli.options.versions) {
        (true, true) => {
          info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
        }
        (true, false) => {
          info!("{}", "= SEMVER RANGES".dimmed());
        }
        (false, true) => {
          info!("{}", "= VERSION MISMATCHES".dimmed());
        }
        (false, false) => {}
      };
    }
    Effects::EnterFormat(config) => {
      if config.cli.options.format {
        info!("{}", "= FORMATTING".dimmed());
      }
    }
    Effects::ExitCommand(state) => {
      if state.is_valid {
        info!("\n{} {}", "✓".green(), "valid");
        process::exit(0);
      } else {
        info!("\n{} {}", "✘".red(), "invalid");
        process::exit(1);
      }
    }

    Effects::PackagesMatchFormatting(valid_packages, _config) => {
      info!(
        "{} {} valid formatting",
        render_count_column(valid_packages.len()),
        "✓".green()
      );
    }
    Effects::PackagesMismatchFormatting(invalid_packages, config, state) => {
      info!(
        "{} {}",
        render_count_column(invalid_packages.len()),
        "invalid formatting".red()
      );
      invalid_packages.iter().for_each(|package| {
        info!(
          "      {} {}",
          "✘".red(),
          package.get_relative_file_path(&config.cwd).red()
        );
      });
      state.is_valid = false;
    }

    Effects::GroupVisited(selector) => {
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

    Effects::DependencyIgnored(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!(
        "{} {} {}",
        count,
        dependency.name.dimmed(),
        "[Ignored]".dimmed()
      );
    }
    Effects::DependencyBanned(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name.red());
    }
    Effects::DependencyMatchesPinnedVersion(dependency) => {
      print_version_match(dependency);
    }
    Effects::DependencyMismatchesPinnedVersion(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name.red());
    }
    Effects::DependencyMatchesRange(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name);
    }
    Effects::DependencyMismatchesRange(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name.red());
    }
    Effects::DependencyMatchesSnapTo(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name);
    }
    Effects::DependencyMismatchesSnapTo(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name.red());
    }
    Effects::DependencyMatchesStandard(dependency) => {
      print_version_match(dependency);
    }
    Effects::DependencyMismatchesStandard(dependency) => {
      let count = render_count_column(dependency.all.len());
      info!("{} {}", count, dependency.name.red());
    }

    Effects::InstanceBanned(event, state) => {
      let icon = "✘".red();
      info!(
        "      {} {} {}",
        icon,
        event.target.0.red(),
        "[Banned]".dimmed()
      );
      state.is_valid = false;
    }
    Effects::InstanceMismatchesPinnedVersion(event, state) => {
      let icon = "✘".red();
      let arrow = "→".dimmed();
      let expected = event.dependency.expected_version.as_ref().unwrap();
      info!(
        "      {} {} {} {} {}",
        icon,
        event.target.0.red(),
        arrow,
        expected.green(),
        "[PinnedMismatch]".dimmed()
      );
      state.is_valid = false;
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
    Effects::InstanceMismatchesSnapTo(event, state) => {
      let (expected, _) = &event.mismatches_with;
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
      state.is_valid = false;
    }
    Effects::InstanceMismatchesLocalVersion(event, state) => {
      let icon = "✘".red();
      let arrow = "→".dimmed();
      let expected = event.dependency.expected_version.as_ref().unwrap();
      info!(
        "      {} {} {} {} {}",
        icon,
        event.target.0.red(),
        arrow,
        expected.green(),
        "[LocalPackageMismatch]".dimmed()
      );
      state.is_valid = false;
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
    Effects::InstanceMismatchesLowestVersion(event, state) => {
      let icon = "✘".red();
      let arrow = "→".dimmed();
      let expected = event.dependency.expected_version.as_ref().unwrap();
      info!(
        "      {} {} {} {} {}",
        icon,
        event.target.0.red(),
        arrow,
        expected.green(),
        "[LowestSemverMismatch]".dimmed()
      );
      state.is_valid = false;
    }
    Effects::InstanceMismatchesHighestVersion(event, state) => {
      let icon = "✘".red();
      let arrow = "→".dimmed();
      let expected = event.dependency.expected_version.as_ref().unwrap();
      info!(
        "      {} {} {} {} {}",
        icon,
        event.target.0.red(),
        arrow,
        expected.green(),
        "[HighestSemverMismatch]".dimmed()
      );
      state.is_valid = false;
    }
  };
}

/// Return a right aligned column of a count of instances
/// Example "    38x"
pub fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

fn print_version_match(dependency: &Dependency) {
  let count = render_count_column(dependency.all.len());
  let (version, _) = dependency.by_specifier.iter().next().unwrap();
  info!("{} {} {}", count, dependency.name, &version.dimmed());
}
