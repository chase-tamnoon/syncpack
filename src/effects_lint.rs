use colored::*;
use log::info;

use crate::{
  config::Config,
  dependency::Dependency,
  effects::{Effects, Event},
  packages::Packages,
};

/// The implementation of the `lint` command's side effects
pub struct LintEffects<'a> {
  pub config: &'a Config,
  pub is_valid: bool,
  pub packages: Option<Packages>,
}

impl<'a> LintEffects<'a> {
  pub fn new(config: &'a Config) -> Self {
    Self {
      config,
      is_valid: true,
      packages: None,
    }
  }
}

impl Effects for LintEffects<'_> {
  fn set_packages(&mut self, packages: Packages) -> () {
    self.packages = Some(packages);
  }

  fn on(&mut self, event: Event) -> () {
    println!("packages.is_some(): {}", self.packages.is_some());
    println!("{:?}", event);

    // match event {
    //   Event::PackagesLoaded(packages) => {
    //     if packages.all_names.is_empty() {
    //       info!("\n{} {}", "✘".red(), "No packages found");
    //       self.is_valid = false;
    //     }
    //   }

    //   Event::EnterVersionsAndRanges => {
    //     if self.config.cli.options.versions {
    //       info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
    //     };
    //   }
    //   Event::EnterFormat => {
    //     if self.config.cli.options.format {
    //       info!("{}", "= FORMATTING".dimmed());
    //     }
    //   }
    //   Event::ExitCommand => {
    //     if self.is_valid {
    //       info!("\n{} {}", "✓".green(), "valid");
    //     } else {
    //       info!("\n{} {}", "✘".red(), "invalid");
    //     }
    //   }

    //   Event::PackagesMatchFormatting(valid_packages) => {
    //     info!(
    //       "{} {} valid formatting",
    //       render_count_column(valid_packages.len()),
    //       "✓".green()
    //     );
    //   }
    //   Event::PackagesMismatchFormatting(invalid_packages) => {
    //     info!(
    //       "{} {}",
    //       render_count_column(invalid_packages.len()),
    //       "invalid formatting".red()
    //     );
    //     invalid_packages.iter().for_each(|package| {
    //       info!(
    //         "      {} {}",
    //         "✘".red(),
    //         package.get_relative_file_path(&self.config.cwd).red()
    //       );
    //     });
    //     self.is_valid = false;
    //   }

    //   Event::GroupVisited(selector) => {
    //     let print_width = 80;
    //     let header = format!("= {} ", selector.label);
    //     let divider = if header.len() < print_width {
    //       "=".repeat(print_width - header.len())
    //     } else {
    //       "".to_string()
    //     };
    //     let full_header = format!("{}{}", header, divider);
    //     info!("{}", full_header.blue());
    //   }

    //   Event::DependencyIgnored(dependency) => {
    //     let count = render_count_column(dependency.all.len());
    //     info!(
    //       "{} {} {}",
    //       count,
    //       dependency.name.dimmed(),
    //       "[Ignored]".dimmed()
    //     );
    //   }
    //   Event::DependencyMatchesWithRange(dependency) => {
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name);
    //   }
    //   Event::DependencyMismatchesWithRange(dependency) => {
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name.red());
    //   }
    //   Event::DependencyBanned(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name.red());
    //   }
    //   Event::DependencyMatchesPinnedVersion(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     print_version_match(dependency);
    //   }
    //   Event::DependencyMismatchesPinnedVersion(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name.red());
    //   }
    //   Event::DependencyMatchesSameRange(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name);
    //   }
    //   Event::DependencyMismatchesSameRange(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name.red());
    //   }
    //   Event::DependencyMatchesSnapTo(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name);
    //   }
    //   Event::DependencyMismatchesSnapTo(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name.red());
    //   }
    //   Event::DependencyMatchesStandard(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     print_version_match(dependency);
    //   }
    //   Event::DependencyMismatchesStandard(dependency) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let count = render_count_column(dependency.all.len());
    //     info!("{} {}", count, dependency.name.red());
    //   }

    //   Event::InstanceMatchesStandard(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✓".green();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {}",
    //       icon,
    //       event.specifier.unwrap().green(),
    //       "[Valid]".dimmed()
    //     );
    //   }
    //   Event::InstanceBanned(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✘".red();
    //     info!(
    //       "      {} {} {}",
    //       icon,
    //       event.specifier.unwrap().red(),
    //       "[Banned]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMatchesWithRange(event) => {
    //     let icon = "✓".green();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {}",
    //       icon,
    //       event.specifier.unwrap().green(),
    //       "[Valid]".dimmed(),
    //     );
    //   }
    //   Event::InstanceMismatchesWithRange(event) => {
    //     let icon = "✘".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.actual_specifier.unwrap().red(),
    //       arrow,
    //       event.expected_specifier.unwrap().green(),
    //       "[SemverRangeMismatch]".dimmed(),
    //     );
    //     self.is_valid = false;
    //     let instance_id = &event.instance_id;
    //     let instance = event.instances_by_id.get_mut(instance_id).unwrap();
    //     instance.expected = event.expected_specifier.clone();
    //   }
    //   Event::InstanceMismatchesPinnedVersion(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✘".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.actual_specifier.unwrap().red(),
    //       arrow,
    //       event.expected_specifier.unwrap().green(),
    //       "[PinnedMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMismatchesSameRange(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     info!(
    //       "      {} {} {} {} {}",
    //       "✘".red(),
    //       event.specifier_outside_range.unwrap().red(),
    //       "falls outside".red(),
    //       event.specifier.unwrap().red(),
    //       "[SameRangeMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMismatchesSnapTo(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✘".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.actual_specifier.unwrap().red(),
    //       arrow,
    //       event.expected_specifier.unwrap().green(),
    //       "[SnappedToMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMismatchCorruptsLocalVersion(event) => {
    //     let icon = "!".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.actual_specifier.unwrap().green(),
    //       arrow,
    //       event.expected_specifier.unwrap().red(),
    //       "[RejectedLocalMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMismatchesLocalVersion(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✘".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.actual_specifier.unwrap().red(),
    //       arrow,
    //       event.expected_specifier.unwrap().green(),
    //       "[LocalPackageMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceUnsupportedMismatch(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✘".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.specifier.unwrap().red(),
    //       arrow,
    //       "?".yellow(),
    //       "[UnsupportedMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMismatchesLowestVersion(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✘".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.actual_specifier.unwrap().red(),
    //       arrow,
    //       event.expected_specifier.unwrap().green(),
    //       "[LowestSemverMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMismatchesHighestVersion(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = "✘".red();
    //     let arrow = "→".dimmed();
    //     info!(
    //       "      {} {} {} {} {}",
    //       icon,
    //       event.actual_specifier.unwrap().red(),
    //       arrow,
    //       event.expected_specifier.unwrap().green(),
    //       "[HighestSemverMismatch]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    // };
  }
}

/// Return a right aligned column of a count of instances
/// Example "    38x"
pub fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

fn print_version_match(dependency: &Dependency) {
  // let count = render_count_column(dependency.all.len());
  // let (specifier, _) = dependency.by_initial_specifier.iter().next().unwrap();
  // info!(
  //   "{} {} {}",
  //   count,
  //   dependency.name,
  //   &specifier.unwrap().dimmed()
  // );
}
