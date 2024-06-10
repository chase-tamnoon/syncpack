use colored::*;
use log::info;

use crate::{
  dependency::Dependency,
  effects::{Effects, Event},
};

/// The implementation of the `lint` command's side effects
pub struct LintEffects {
  pub is_valid: bool,
}

impl LintEffects {
  pub fn new() -> Self {
    Self { is_valid: true }
  }
}

impl Effects for LintEffects {
  fn on(&mut self, event: Event) -> () {
    match event {
      Event::PackagesLoaded(config, packages) => {
        if packages.all_names.is_empty() {
          info!("\n{} {}", "✘".red(), "No packages found");
          self.is_valid = false;
        }
      }

      Event::EnterVersionsAndRanges(config) => {
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
      Event::EnterFormat(config) => {
        if config.cli.options.format {
          info!("{}", "= FORMATTING".dimmed());
        }
      }
      Event::ExitCommand => {
        if self.is_valid {
          info!("\n{} {}", "✓".green(), "valid");
        } else {
          info!("\n{} {}", "✘".red(), "invalid");
        }
      }

      Event::PackagesMatchFormatting(valid_packages, _config) => {
        info!(
          "{} {} valid formatting",
          render_count_column(valid_packages.len()),
          "✓".green()
        );
      }
      Event::PackagesMismatchFormatting(invalid_packages, config) => {
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
        self.is_valid = false;
      }

      Event::GroupVisited(selector) => {
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

      Event::DependencyIgnored(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!(
          "{} {} {}",
          count,
          dependency.name.dimmed(),
          "[Ignored]".dimmed()
        );
      }
      Event::DependencyMatchesWithRange(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name);
      }
      Event::DependencyMismatchesWithRange(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
      Event::DependencyBanned(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
      Event::DependencyMatchesPinnedVersion(dependency) => {
        print_version_match(dependency);
      }
      Event::DependencyMismatchesPinnedVersion(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
      Event::DependencyMatchesSameRange(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name);
      }
      Event::DependencyMismatchesSameRange(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
      Event::DependencyMatchesSnapTo(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name);
      }
      Event::DependencyMismatchesSnapTo(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
      Event::DependencyMatchesStandard(dependency) => {
        print_version_match(dependency);
      }
      Event::DependencyMismatchesStandard(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }

      Event::InstanceMatchesStandard(event) => {
        let icon = "✓".green();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {}",
          icon,
          event.specifier.unwrap().green(),
          "[Valid]".dimmed()
        );
      }
      Event::InstanceBanned(event) => {
        let icon = "✘".red();
        info!(
          "      {} {} {}",
          icon,
          event.specifier.unwrap().red(),
          "[Banned]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesSemverRange(event) => {
        let icon = "✘".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.actual_specifier.unwrap().red(),
          arrow,
          event.expected_specifier.unwrap().green(),
          "[SemverRangeMismatch]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesPinnedVersion(event) => {
        let icon = "✘".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.actual_specifier.unwrap().red(),
          arrow,
          event.expected_specifier.unwrap().green(),
          "[PinnedMismatch]".dimmed()
        );
        self.is_valid = false;
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
        let icon = "✘".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.actual_specifier.unwrap().red(),
          arrow,
          event.expected_specifier.unwrap().green(),
          "[SnappedToMismatch]".dimmed()
        );
        self.is_valid = false;
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
        let icon = "✘".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.actual_specifier.unwrap().red(),
          arrow,
          event.expected_specifier.unwrap().green(),
          "[LocalPackageMismatch]".dimmed()
        );
        self.is_valid = false;
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
        let icon = "✘".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.actual_specifier.unwrap().red(),
          arrow,
          event.expected_specifier.unwrap().green(),
          "[LowestSemverMismatch]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesHighestVersion(event) => {
        let icon = "✘".red();
        let arrow = "→".dimmed();
        info!(
          "      {} {} {} {} {}",
          icon,
          event.actual_specifier.unwrap().red(),
          arrow,
          event.expected_specifier.unwrap().green(),
          "[HighestSemverMismatch]".dimmed()
        );
        self.is_valid = false;
      }
    };
  }
}

/// Return a right aligned column of a count of instances
/// Example "    38x"
pub fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

fn print_version_match(dependency: &Dependency) {
  let count = render_count_column(dependency.all.len());
  let (specifier, _) = dependency.by_specifier.iter().next().unwrap();
  info!(
    "{} {} {}",
    count,
    dependency.name,
    &specifier.unwrap().dimmed()
  );
}
