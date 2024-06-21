use colored::*;
use log::info;

use crate::{
  config::Config,
  context::InstancesById,
  dependency::Dependency,
  effects::{Effects, Event},
  instance::Instance,
  packages::Packages,
  version_group::Variant,
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
  fn get_packages(&mut self) -> Packages {
    let packages = self.packages.take().unwrap();
    self.packages = None;
    packages
  }

  fn set_packages(&mut self, packages: Packages) -> () {
    self.packages = Some(packages);
  }

  fn on(&mut self, event: Event, instances_by_id: &mut InstancesById) -> () {
    match &event {
      Event::EnterVersionsAndRanges => {
        info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
      }
      Event::EnterFormat => {
        info!("{}", "= FORMATTING".dimmed());
      }
      Event::GroupVisited(group) => {
        let print_width = 80;
        let header = format!("= {} ", group.label);
        let divider = if header.len() < print_width { "=".repeat(print_width - header.len()) } else { "".to_string() };
        let full_header = format!("{}{}", header, divider);
        info!("{}", full_header.blue());
      }
      Event::DependencyValid(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name);
      }
      Event::DependencyInvalid(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.red());
      }
      Event::DependencyWarning(dependency) => {
        let count = render_count_column(dependency.all.len());
        info!("{} {}", count, dependency.name.yellow());
      }
      Event::LocalInstanceIsPreferred(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = green_tick();
        let hint = "*is local";
        let location_hint = instance.location_hint.dimmed();
        let actual = instance.actual.unwrap().green();
        info!("      {icon} {actual} {hint} {location_hint}");
      }
      Event::InstanceMatchesLocal(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = green_tick();
        let hint = "*matches local";
        let location_hint = instance.location_hint.dimmed();
        let actual = instance.actual.unwrap().green();
        info!("      {icon} {actual} {hint} {location_hint}");
      }
      Event::InstanceMatchesHighestOrLowestSemver(instance_id, variant) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = green_tick();
        let high_low = high_low_hint(variant);
        let hint = format!("is {high_low}").dimmed();
        let location_hint = instance.location_hint.dimmed();
        let actual = instance.actual.unwrap().green();
        info!("      {icon} {actual} {hint} {location_hint}");
      }
      Event::InstanceMatchesButIsUnsupported(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceIsIgnored(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMatchesPinned(instance_id) => {
        // let instance = instances_by_id.get(instance_id).unwrap();
        // let icon = red_cross();
        // let location_hint = instance.location_hint.dimmed();
        // let actual = instance.actual.unwrap().red();
        // let expected = instance.expected.unwrap().green();
        // let arrow = dimmed_arrow();
        // info!("      {icon} {actual} {arrow} {expected} {location_hint}");
        // self.is_valid = false;
      }
      Event::InstanceMatchesSameRangeGroup(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::LocalInstanceMistakenlyBanned(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceIsBanned(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = red_cross();
        let hint = "banned".red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {hint} {location_hint}");
        self.is_valid = false;
      }
      Event::InstanceMatchesHighestOrLowestSemverButMismatchesSemverGroup(instance_id, variant) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = red_cross();
        let high_low = high_low_hint(variant);
        let hint = format!("is {high_low} but mismatches its semver group").dimmed();
        let location_hint = instance.location_hint.dimmed();
        let actual = instance.actual.unwrap().red();
        let arrow = dimmed_arrow();
        let expected = instance.expected.unwrap().green();
        info!("      {icon} {actual} {arrow} {expected} {hint} {location_hint}");
        self.is_valid = false;
      }
      Event::InstanceMatchesLocalButMismatchesSemverGroup(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMismatchesLocal(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMismatchesHighestOrLowestSemver(instance_id, variant) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("      {} {} {}", red_cross(), actual_to_expected(&instance), instance.location_hint.dimmed());
        self.is_valid = false;
      }
      Event::InstanceMismatchesAndIsUnsupported(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::LocalInstanceMistakenlyMismatchesSemverGroup(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMatchesPinnedButMismatchesSemverGroup(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::LocalInstanceMistakenlyMismatchesPinned(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMismatchesPinned(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = red_cross();
        let actual = instance.actual.unwrap().red();
        let arrow = dimmed_arrow();
        let expected = instance.expected.unwrap().green();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {arrow} {expected} {location_hint}");
        self.is_valid = false;
      }
      Event::InstanceMismatchesBothSameRangeAndConflictingSemverGroups(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::InstanceMismatchesSameRangeGroup(instance_id) => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  {:?}", &event);
      }
      Event::PackagesMatchFormatting(valid_packages) => {
        info!("{} {} valid formatting", render_count_column(valid_packages.len()), green_tick());
      }
      Event::PackagesMismatchFormatting(invalid_packages) => {
        info!("{} {}", render_count_column(invalid_packages.len()), "invalid formatting".red());
        invalid_packages.iter().for_each(|package| {
          info!("      {} {}", red_cross(), package.get_relative_file_path(&self.config.cwd).red());
        });
        self.is_valid = false;
      }
      Event::ExitCommand => {
        if self.is_valid {
          info!("\n{} {}", green_tick(), "valid");
        } else {
          info!("\n{} {}", red_cross(), "invalid");
        }
      }
    }

    // match event {
    //   Event::InstanceMatchesStandard(event) => {
    //     if !self.config.cli.options.versions {
    //       return;
    //     }
    //     let icon = green_tick();
    //     let arrow = dimmed_arrow();
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
    //     let icon = red_cross();
    //     info!(
    //       "      {} {} {}",
    //       icon,
    //       event.specifier.unwrap().red(),
    //       "[Banned]".dimmed()
    //     );
    //     self.is_valid = false;
    //   }
    //   Event::InstanceMatchesWithRange(event) => {
    //     let icon = green_tick();
    //     let arrow = dimmed_arrow();
    //     info!(
    //       "      {} {} {}",
    //       icon,
    //       event.specifier.unwrap().green(),
    //       "[Valid]".dimmed(),
    //     );
    //   }
    //   Event::InstanceMismatchesWithRange(event) => {
    //     let icon = red_cross();
    //     let arrow = dimmed_arrow();
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
    //     let icon = red_cross();
    //     let arrow = dimmed_arrow();
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
    //       red_cross(),
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
    //     let icon = red_cross();
    //     let arrow = dimmed_arrow();
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
    //     let arrow = dimmed_arrow();
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
    //     let icon = red_cross();
    //     let arrow = dimmed_arrow();
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
    //     let icon = red_cross();
    //     let arrow = dimmed_arrow();
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
    //     let icon = red_cross();
    //     let arrow = dimmed_arrow();
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
    //     let icon = red_cross();
    //     let arrow = dimmed_arrow();
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
  // info!("{} {} {}", count, dependency.name, &specifier.unwrap().dimmed());
  info!("@TODO print_version_match");
}

fn actual_to_expected(instance: &Instance) -> String {
  let actual = instance.actual.unwrap();
  let expected = instance.expected.unwrap();
  format!("{} {} {}", actual.red(), dimmed_arrow(), expected.dimmed())
}

fn high_low_hint(variant: &Variant) -> &str {
  let is_highest = matches!(variant, Variant::HighestSemver);
  if is_highest {
    "highest semver"
  } else {
    "lowest semver"
  }
}

fn green_tick() -> ColoredString {
  "✓".green()
}

fn red_cross() -> ColoredString {
  "✘".red()
}

fn dimmed_arrow() -> ColoredString {
  "→".dimmed()
}
