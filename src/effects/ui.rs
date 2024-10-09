use std::{cell::RefCell, rc::Rc};

use colored::*;
use itertools::Itertools;
use log::info;

use crate::{
  context::Context,
  dependency::Dependency,
  instance::{Instance, InstanceState},
  package_json::{FormatMismatch, FormatMismatchVariant, PackageJson},
  specifier::Specifier,
  version_group::{VersionGroup, VersionGroupVariant},
};

#[derive(Debug)]
pub struct Ui<'a> {
  pub ctx: &'a Context,
  /// Whether to output ignored dependencies regardless
  pub show_ignored: bool,
  /// Whether to list every affected instance of a dependency when listing
  /// version or semver range
  /// mismatches
  pub show_instances: bool,
  /// Whether to show the name of the status code for each dependency and
  /// instance, such as `HighestSemverMismatch`
  pub show_status_codes: bool,
  /// Whether to list every affected package.json file when listing formatting
  /// mismatches
  pub show_packages: bool,
}

impl<'a> Ui<'a> {
  pub fn green_tick(&self) -> ColoredString {
    "✓".green()
  }

  pub fn red_cross(&self) -> ColoredString {
    "✘".red()
  }

  pub fn yellow_warning(&self) -> ColoredString {
    "!".yellow()
  }

  fn dim_right_arrow(&self) -> ColoredString {
    "→".dimmed()
  }

  pub fn err(&self, msg: &str) -> ColoredString {
    format!("{} {}", self.red_cross(), msg).red()
  }

  pub fn warn(&self, msg: &str) -> ColoredString {
    format!("{} {}", self.yellow_warning(), msg).yellow()
  }

  pub fn link(&self, url: impl Into<String>, text: impl Into<ColoredString>) -> ColoredString {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url.into(), text.into()).normal()
  }

  pub fn print_group_header(&self, group: &VersionGroup) {
    let print_width = 80;
    let label = &group.selector.label;
    let header = format!("= {label} ");
    let divider = if header.len() < print_width {
      "=".repeat(print_width - header.len())
    } else {
      "".to_string()
    };
    let full_header = format!("{header}{divider}");
    info!("{}", full_header.blue());
  }

  pub fn print_dependency_header(&self, dependency: &Dependency) {
    if !self.show_ignored && matches!(dependency.variant, VersionGroupVariant::Ignored) {
      return;
    }
    let state = dependency.get_state();
    let name = if matches!(state, InstanceState::Invalid(_)) {
      dependency.name.red()
    } else {
      dependency.name.normal()
    };
    let count = self.count_column(dependency.instances.borrow().len());
    let status_code = self.get_dependency_status_code(dependency);
    let unique_specifiers = dependency.get_unique_specifiers();
    let icon_will_be_shown_by_every_instance = self.show_instances;
    let icon = if icon_will_be_shown_by_every_instance {
      " ".normal()
    } else {
      let icon = self.state_icon(&state);
      format!(" {icon} ").normal()
    };
    if unique_specifiers.len() == 1 {
      let colon = ":".dimmed();
      let specifier = self.get_dependency_specifier(dependency, &unique_specifiers);
      info!("{count}{icon}{name}{specifier} {status_code}");
    } else {
      info!("{count}{icon}{name} {status_code}");
    }
  }

  fn get_dependency_specifier(&self, dependency: &Dependency, unique_specifiers: &[Specifier]) -> ColoredString {
    let will_be_shown_by_every_instance = self.show_instances;
    if will_be_shown_by_every_instance {
      return "".normal();
    }
    let state = dependency.get_state();
    let colon = ":".dimmed();
    let specifier = unique_specifiers.first().unwrap().unwrap();
    let specifier = if matches!(state, InstanceState::Invalid(_)) {
      specifier.red()
    } else {
      specifier.dimmed()
    };
    format!("{colon} {specifier}").normal()
  }

  fn get_dependency_status_code(&self, dependency: &Dependency) -> ColoredString {
    let state = dependency.get_state();
    let has_issue = matches!(state, InstanceState::Invalid(_) | InstanceState::Suspect(_));
    let will_be_shown_by_every_instance = self.show_instances;
    if has_issue && !will_be_shown_by_every_instance {
      self.instance_state_link(&state)
    } else {
      "".normal()
    }
  }

  /// Return a right aligned column of a count of instances
  /// Example "    38x"
  fn count_column(&self, count: usize) -> ColoredString {
    format!("{: >4}x", count).dimmed()
  }

  pub fn instance_state_link(&self, instance_state: &InstanceState) -> ColoredString {
    if !self.show_status_codes {
      return "".normal();
    }
    let base_url = "https://jamiemason.github.io/syncpack/guide/status-codes/";
    let branch_name = instance_state.get_name();
    let branch_name_lower_case = branch_name.to_lowercase();
    let plain_link = self.link(format!("{base_url}#{branch_name_lower_case}"), branch_name);
    format!("({plain_link})").dimmed()
  }

  pub fn format_mismatch_variant_link(&self, state: &FormatMismatchVariant) -> ColoredString {
    let base_url = "https://jamiemason.github.io/syncpack/guide/status-codes/";
    let branch_name = format!("{:?}", state);
    let branch_name_lower_case = branch_name.to_lowercase();
    let plain_link = self.link(format!("{base_url}#{branch_name_lower_case}"), branch_name);
    format!("{plain_link}").normal()
  }

  pub fn package_json_link(&self, package: &PackageJson) -> ColoredString {
    let name = package.get_name_unsafe();
    let file_path = package.file_path.to_str().unwrap();
    let plain_link = self.link(format!("file:{file_path}"), name);
    format!("{plain_link}").normal()
  }

  pub fn state_icon(&self, state: &InstanceState) -> ColoredString {
    match state {
      InstanceState::Valid(_) => self.green_tick(),
      InstanceState::Invalid(_) => self.red_cross(),
      InstanceState::Suspect(_) => self.yellow_warning(),
      InstanceState::Unknown => panic!("Unknown state"),
    }
  }

  pub fn print_formatted_packages(&self, packages: Vec<&Rc<RefCell<PackageJson>>>) {
    if !packages.is_empty() {
      let icon = self.green_tick();
      let count = self.count_column(packages.len());
      let status = "Valid".green();
      info!("{count} {icon} {status}");
      if self.show_packages {
        packages.iter().for_each(|package| {
          self.print_formatted_package(&package.borrow());
        });
      }
    }
  }

  /// Print a package.json which is correctly formatted
  pub fn print_formatted_package(&self, package: &PackageJson) {
    if package.formatting_mismatches.borrow().is_empty() {
      let icon = "-".dimmed();
      let file_link = self.package_json_link(package).blue();
      info!("      {icon} {file_link}");
    }
  }

  /// Print every package.json which has the given formatting mismatch
  pub fn print_formatting_mismatches(&self, variant: &FormatMismatchVariant, mismatches: &[Rc<FormatMismatch>]) {
    let count = self.count_column(mismatches.len());
    let icon = self.red_cross();
    let link = self.format_mismatch_variant_link(variant).red();
    info!("{count} {icon} {link}");
    if self.show_packages {
      mismatches
        .iter()
        .sorted_by(|a, b| b.package.borrow().get_name_unsafe().cmp(&a.package.borrow().get_name_unsafe()))
        .for_each(|mismatch| {
          let icon = "-".dimmed();
          let package = mismatch.package.borrow();
          let property_path = self.format_path(&mismatch.property_path).dimmed();
          let file_link = self.package_json_link(&package).blue();
          let in_ = "in".dimmed();
          let at = "at".dimmed();
          let msg = format!("      {icon} {in_} {file_link} {at} {property_path}");
          info!("{msg}");
        });
    }
  }

  /// Convert a Rust property path to a JS one
  /// eg. "/dependencies/react" -> "dependencies.react"
  fn format_path(&self, path: &str) -> ColoredString {
    let path = path.replace("/", ".");
    path.normal()
  }

  pub fn print_instances(&self, instances: &[Rc<Instance>]) {
    if self.show_instances {
      instances
        .iter()
        .sorted_unstable_by_key(|instance| (instance.actual_specifier.unwrap(), &instance.name, &instance.dependency_type.path))
        .rev()
        .for_each(|instance| self.print_instance(instance))
    }
  }

  fn print_instance(&self, instance: &Instance) {
    let state = instance.state.borrow();
    let state_icon = self.state_icon(&state);
    let specifier = instance.actual_specifier.unwrap();
    let specifier = match *state {
      InstanceState::Valid(_) => specifier.green(),
      InstanceState::Invalid(_) => specifier.red(),
      InstanceState::Suspect(_) => specifier.yellow(),
      InstanceState::Unknown => "".normal(),
    };
    let package = instance.package.borrow();
    let property_path = instance.dependency_type.path.replace("/", ".").dimmed();
    let file_link = self.package_json_link(&package).blue();
    let in_ = "in".dimmed();
    let state_link = self.instance_state_link(&state);
    let tail = format!("at {property_path} {state_link}").dimmed();
    info!("      {state_icon} {specifier} {in_} {file_link} {tail}");
  }

  /*
  fn on_exit_command() {
    if self.is_valid {
      info!("\n{} {}", green_tick(), "valid");
    } else {
      info!("\n{} {}", red_cross(), "invalid");
    }
  }

  fn on_instance(&mut self, event: InstanceEvent) {
    let instance = &event.instance;
    let dependency = &event.dependency;
    match &event.variant {
      InstanceState::Unknown => {
        info!("@TODO: InstanceState::Unknown '{}'", instance.id);
      }
      /* Ignored */
      InstanceState::Ignored => { /*NOOP*/ }
      /* Matches */
      InstanceState::ValidLocal
      | InstanceState::EqualsLocal
      | InstanceState::MatchesLocal
      | InstanceState::EqualsPreferVersion
      | InstanceState::EqualsSnapToVersion
      | InstanceState::EqualsNonSemverPreferVersion
      | InstanceState::EqualsPin
      | InstanceState::MatchesSameRangeGroup => {
        let icon = green_tick();
        let actual = instance.actual_specifier.unwrap().green();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
      }
      /* Warnings */
      InstanceState::RefuseToBanLocal => {
        info!("@TODO: explain RefuseToBanLocal");
      }
      InstanceState::RefuseToPinLocal => {
        info!("@TODO: explain RefuseToPinLocal");
      }
      InstanceState::RefuseToSnapLocal => {
        info!("@TODO: explain RefuseToSnapLocal");
      }
      InstanceState::InvalidLocalVersion => {
        info!("@TODO: explain InvalidLocalVersion");
      }
      InstanceState::MatchesPreferVersion => {
        // return /*SKIP*/;
        let icon = red_cross();
        let actual = instance.actual_specifier.unwrap().red();
        let high_low = high_low_hint(&dependency.variant);
        let opposite = if matches!(dependency.variant, Variant::HighestSemver) {
          "lower"
        } else {
          "higher"
        };
        let hint =
          format!("is {high_low} but mismatches its semver group, fixing its semver group would cause its version to be {opposite}").dimmed();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {hint} {location_hint}");
        self.is_valid = false;
      }
      InstanceState::MatchesSnapToVersion => {
        info!("@TODO: explain MatchesSnapToVersion");
      }
      /* Overrides */
      InstanceState::PinMatchOverridesSemverRangeMatch => {
        info!("@TODO: explain PinMatchOverridesSemverRangeMatch");
      }
      InstanceState::PinMatchOverridesSemverRangeMismatch => {
        info!("@TODO: explain PinMatchOverridesSemverRangeMismatch");
      }
      /* Fixable Mismatches */
      InstanceState::Banned => {
        // return /*SKIP*/;
        let icon = red_cross();
        let hint = "banned".red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {hint} {location_hint}");
        self.is_valid = false;
      }
      InstanceState::MismatchesLocal => {
        info!("@TODO: explain MismatchesLocal");
      }
      InstanceState::MismatchesPreferVersion => {
        // return /*SKIP*/;
        let icon = red_cross();
        let actual = instance.actual_specifier.unwrap().red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
        self.is_valid = false;
      }
      InstanceState::MismatchesSnapToVersion => {
        info!("@TODO: explain MismatchesSnapToVersion");
      }
      InstanceState::MismatchesPin => {
        // return /*SKIP*/;
        let icon = red_cross();
        let actual = instance.actual_specifier.unwrap().red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
        self.is_valid = false;
      }
      InstanceState::SemverRangeMismatch => {
        info!("@TODO: explain SemverRangeMismatch");
      }
      /* Conflicts */
      InstanceState::SemverRangeMatchConflictsWithPreferVersion => {
        info!("@TODO: explain SemverRangeMatchConflictsWithPreferVersion");
      }
      InstanceState::SemverRangeMismatchConflictsWithPreferVersion => {
        info!("@TODO: explain SemverRangeMismatchConflictsWithPreferVersion");
      }
      InstanceState::SemverRangeMatchConflictsWithSnapToVersion => {
        info!("@TODO: explain SemverRangeMatchConflictsWithSnapToVersion");
      }
      InstanceState::SemverRangeMismatchConflictsWithSnapToVersion => {
        info!("@TODO: explain SemverRangeMismatchConflictsWithSnapToVersion");
      }
      InstanceState::SemverRangeMatchConflictsWithLocalVersion => {
        info!("@TODO: explain SemverRangeMatchConflictsWithLocalVersion");
      }
      InstanceState::SemverRangeMismatchConflictsWithLocalVersion => {
        info!("@TODO: explain SemverRangeMismatchConflictsWithLocalVersion");
      }
      /* Unfixable Mismatches */
      InstanceState::MismatchesInvalidLocalVersion => {
        info!("@TODO: explain MismatchesInvalidLocalVersion");
      }
      InstanceState::MismatchesNonSemverPreferVersion => {
        // return /*SKIP*/;
        let icon = red_cross();
        let actual = instance.actual_specifier.unwrap().red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
        self.is_valid = false;
      }
      InstanceState::MismatchesSameRangeGroup => {
        info!("@TODO: explain MismatchesSameRangeGroup");
      }
      InstanceState::SnapToVersionNotFound => {
        info!("@TODO: explain SnapToVersionNotFound");
      }
    }
  }

  fn high_low_hint(variant: &Variant) -> &str {
    let is_highest = matches!(variant, Variant::HighestSemver);
    if is_highest {
      "highest semver"
    } else {
      "lowest semver"
    }
  }


  */
}
