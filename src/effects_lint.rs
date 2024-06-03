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
      Event::DependencyMatchesRange(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name);
      }
      Event::DependencyMismatchesRange(dependency) => {
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

      Event::InstanceMatchesStandard(_) => {
        //
      }
      Event::InstanceBanned(event) => {
        let icon = "✘".red();
        info!(
          "      {} {} {}",
          icon,
          event.target.0.red(),
          "[Banned]".dimmed()
        );
        self.is_valid = false;
      }
      Event::InstanceMismatchesPinnedVersion(event) => {
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
        self.is_valid = false;
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
        self.is_valid = false;
      }
      Event::InstanceMismatchesLocalVersion(event) => {
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
        self.is_valid = false;
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
        self.is_valid = false;
      }
      Event::InstanceMismatchesHighestVersion(event) => {
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
  let (version, _) = dependency.by_specifier.iter().next().unwrap();
  info!("{} {} {}", count, dependency.name, &version.dimmed());
}
