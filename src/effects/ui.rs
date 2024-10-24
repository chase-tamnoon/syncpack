use std::{cell::RefCell, rc::Rc};

use colored::*;
use itertools::Itertools;
use log::info;

use crate::{
  context::Context,
  dependency::Dependency,
  instance::Instance,
  instance_state::{
    FixableInstance, InstanceState, InvalidInstance, SemverGroupAndVersionConflict, SuspectInstance, UnfixableInstance, ValidInstance,
  },
  package_json::{FormatMismatch, FormatMismatchVariant, PackageJson},
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
  pub fn print_command_header(&self, msg: &str) {
    info!("{}", format!(" {msg} ").on_blue().black());
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

  pub fn print_dependency(&self, dependency: &Dependency, group_variant: &VersionGroupVariant) {
    let state_links = dependency
      .get_states()
      .iter()
      .map(|state| {
        let state_name = state.get_name();
        // Issues related to whether a specifier is the highest or lowest semver are
        // all the same logic internally, so we have combined enum branches for
        // them, but from an end user point of view though it is clearer to have a
        // specific status code related to what has happened.
        if matches!(group_variant, VersionGroupVariant::HighestSemver) {
          state_name.replace("HighestOrLowestSemver", "HighestSemver")
        } else if matches!(group_variant, VersionGroupVariant::LowestSemver) {
          state_name.replace("HighestOrLowestSemver", "LowestSemver")
        } else {
          state_name
        }
      })
      .sorted()
      .unique()
      .map(|state_name| self.status_code_link(&state_name))
      .join(", ");
    let state_links = format!("({state_links})").dimmed();
    let instances_len = dependency.instances.borrow().len();
    let count = self.count_column(instances_len);
    let name = &dependency.name;
    let expected = dependency
      .expected
      .borrow()
      .clone()
      .map(|expected| expected.unwrap())
      .unwrap_or("".to_string())
      .dimmed();

    match &dependency.get_state() {
      InstanceState::Valid(variant) => {
        let icon = self.ok_icon();
        match variant {
          ValidInstance::IsIgnored => {
            let icon = self.unknown_icon();
            let name = name.dimmed();
            info!("{count} {icon} {name} {state_links}");
          }
          ValidInstance::IsLocalAndValid => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::IsIdenticalToLocal => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::SatisfiesLocal => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::IsHighestOrLowestSemver => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::SatisfiesHighestOrLowestSemver => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::IsNonSemverButIdentical => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::IsIdenticalToPin => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::SatisfiesSameRangeGroup => {
            info!("{count} {icon} {name} {state_links}");
          }
          ValidInstance::IsIdenticalToSnapTarget => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
          ValidInstance::SatisfiesSnapTarget => {
            info!("{count} {icon} {name} {expected} {state_links}");
          }
        }
      }
      InstanceState::Invalid(variant) => {
        let name = name.red();
        match variant {
          InvalidInstance::Fixable(variant) => {
            let icon = self.err_icon();
            match variant {
              FixableInstance::IsBanned => {
                info!("{count} {icon} {name} {state_links}");
              }
              FixableInstance::DiffersToLocal => {
                info!("{count} {icon} {name} {expected} {state_links}");
              }
              FixableInstance::DiffersToHighestOrLowestSemver => {
                info!("{count} {icon} {name} {expected} {state_links}");
              }
              FixableInstance::DiffersToSnapTarget => {
                info!("{count} {icon} {name} {expected} {state_links}");
              }
              FixableInstance::DiffersToPin => {
                info!("{count} {icon} {name} {expected} {state_links}");
              }
              FixableInstance::SemverRangeMismatch => {
                info!("{count} {icon} {name} {expected} {state_links}");
              }
              FixableInstance::PinOverridesSemverRange => {
                info!("{count} {icon} {name} {expected} {state_links}");
              }
              FixableInstance::PinOverridesSemverRangeMismatch => {
                info!("{count} {icon} {name} {expected} {state_links}");
              }
            }
          }
          InvalidInstance::Unfixable(variant) => {
            let icon = self.err_icon();
            match variant {
              UnfixableInstance::DependsOnInvalidLocalPackage => {
                info!("{count} {icon} {name} {state_links}");
              }
              UnfixableInstance::NonSemverMismatch => {
                info!("{count} {icon} {name} {state_links}");
              }
              UnfixableInstance::SameRangeMismatch => {
                info!("{count} {icon} {name} {state_links}");
              }
              UnfixableInstance::DependsOnMissingSnapTarget => {
                info!("{count} {icon} {name} {state_links}");
              }
            }
          }
          InvalidInstance::Conflict(variant) => {
            let icon = self.err_icon();
            match variant {
              SemverGroupAndVersionConflict::MatchConflictsWithHighestOrLowestSemver => {
                info!("{count} {icon} {name} {state_links}");
              }
              SemverGroupAndVersionConflict::MismatchConflictsWithHighestOrLowestSemver => {
                info!("{count} {icon} {name} {state_links}");
              }
              SemverGroupAndVersionConflict::MatchConflictsWithSnapTarget => {
                info!("{count} {icon} {name} {state_links}");
              }
              SemverGroupAndVersionConflict::MismatchConflictsWithSnapTarget => {
                info!("{count} {icon} {name} {state_links}");
              }
              SemverGroupAndVersionConflict::MatchConflictsWithLocal => {
                info!("{count} {icon} {name} {state_links}");
              }
              SemverGroupAndVersionConflict::MismatchConflictsWithLocal => {
                info!("{count} {icon} {name} {state_links}");
              }
            }
          }
        }
      }
      InstanceState::Suspect(variant) => {
        let icon = self.warn_icon();
        match variant {
          SuspectInstance::RefuseToBanLocal => {
            info!("{count} {icon} {name} {state_links}");
          }
          SuspectInstance::RefuseToPinLocal => {
            info!("{count} {icon} {name} {state_links}");
          }
          SuspectInstance::RefuseToSnapLocal => {
            info!("{count} {icon} {name} {state_links}");
          }
          SuspectInstance::InvalidLocalVersion => {
            info!("{count} {icon} {name} {state_links}");
          }
        }
      }
      InstanceState::Unknown => {
        panic!("Unknown");
      }
    }
  }

  pub fn for_each_instance(&self, dependency: &Dependency, f: impl Fn(&Rc<Instance>)) {
    dependency
      .instances
      .borrow()
      .iter()
      // .sorted_unstable_by_key(|instance| (instance.actual_specifier.unwrap(), &instance.name, &instance.dependency_type.path))
      // .rev()
      .for_each(f);
  }

  pub fn print_instance(&self, instance: &Instance, group_variant: &VersionGroupVariant) {
    let state = instance.state.borrow().clone();
    let state_name = state.get_name();
    // Issues related to whether a specifier is the highest or lowest semver are
    // all the same logic internally, so we have combined enum branches for
    // them, but from an end user point of view though it is clearer to have a
    // specific status code related to what has happened.
    let state_name = if matches!(group_variant, VersionGroupVariant::HighestSemver) {
      state_name.replace("HighestOrLowestSemver", "HighestSemver").normal()
    } else if matches!(group_variant, VersionGroupVariant::LowestSemver) {
      state_name.replace("HighestOrLowestSemver", "LowestSemver").normal()
    } else {
      state_name.normal()
    };
    let state_link = self.status_code_link(&state_name);
    let state_link = format!("({state_link})").dimmed();
    let actual = instance.actual_specifier.unwrap();
    let location = self.instance_location(instance).dimmed();
    match &state {
      InstanceState::Valid(variant) => {
        let icon = self.ok_icon();
        let actual = if matches!(variant, ValidInstance::IsIgnored) {
          actual.dimmed()
        } else {
          actual.green()
        };
        match variant {
          ValidInstance::IsIgnored => {
            let icon = self.unknown_icon();
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::IsLocalAndValid => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::IsIdenticalToLocal => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::SatisfiesLocal => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::IsHighestOrLowestSemver => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::SatisfiesHighestOrLowestSemver => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::IsNonSemverButIdentical => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::IsIdenticalToPin => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::SatisfiesSameRangeGroup => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::IsIdenticalToSnapTarget => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          ValidInstance::SatisfiesSnapTarget => {
            info!("      {icon} {actual} {location} {state_link}");
          }
        }
      }
      InstanceState::Invalid(variant) => {
        let icon = self.err_icon();
        let actual = actual.red();
        match variant {
          InvalidInstance::Fixable(variant) => match variant {
            FixableInstance::IsBanned => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            FixableInstance::DiffersToLocal => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            FixableInstance::DiffersToHighestOrLowestSemver => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            FixableInstance::DiffersToSnapTarget => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            FixableInstance::DiffersToPin => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            FixableInstance::SemverRangeMismatch => {
              let arrow = self.dim_right_arrow();
              let expected = instance.get_specifier_with_preferred_semver_range();
              let expected = expected.unwrap().unwrap();
              let expected = expected.green();
              info!("      {icon} {actual} {arrow} {expected} {location} {state_link}");
            }
            FixableInstance::PinOverridesSemverRange => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            FixableInstance::PinOverridesSemverRangeMismatch => {
              info!("      {icon} {actual} {location} {state_link}");
            }
          },
          InvalidInstance::Unfixable(variant) => match variant {
            UnfixableInstance::DependsOnInvalidLocalPackage => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            UnfixableInstance::NonSemverMismatch => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            UnfixableInstance::SameRangeMismatch => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            UnfixableInstance::DependsOnMissingSnapTarget => {
              info!("      {icon} {actual} {location} {state_link}");
            }
          },
          InvalidInstance::Conflict(variant) => match variant {
            SemverGroupAndVersionConflict::MatchConflictsWithHighestOrLowestSemver => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            SemverGroupAndVersionConflict::MismatchConflictsWithHighestOrLowestSemver => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            SemverGroupAndVersionConflict::MatchConflictsWithSnapTarget => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            SemverGroupAndVersionConflict::MismatchConflictsWithSnapTarget => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            SemverGroupAndVersionConflict::MatchConflictsWithLocal => {
              info!("      {icon} {actual} {location} {state_link}");
            }
            SemverGroupAndVersionConflict::MismatchConflictsWithLocal => {
              info!("      {icon} {actual} {location} {state_link}");
            }
          },
        }
      }
      InstanceState::Suspect(variant) => {
        let icon = self.warn_icon();
        match variant {
          SuspectInstance::RefuseToBanLocal => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          SuspectInstance::RefuseToPinLocal => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          SuspectInstance::RefuseToSnapLocal => {
            info!("      {icon} {actual} {location} {state_link}");
          }
          SuspectInstance::InvalidLocalVersion => {
            info!("      {icon} {actual} {location} {state_link}");
          }
        }
      }
      InstanceState::Unknown => {
        panic!("Unknown");
      }
    }
  }

  fn ok_icon(&self) -> ColoredString {
    "✓".green()
  }

  fn err_icon(&self) -> ColoredString {
    "✘".red()
  }

  fn warn_icon(&self) -> ColoredString {
    "!".yellow()
  }

  fn unknown_icon(&self) -> ColoredString {
    "?".dimmed()
  }

  fn dim_right_arrow(&self) -> ColoredString {
    "→".dimmed()
  }

  /// Return a right-aligned column of a count of instances
  /// Example "    38x"
  fn count_column(&self, count: usize) -> ColoredString {
    format!("{: >4}x", count).dimmed()
  }

  /// Return a location hint for an instance
  fn instance_location(&self, instance: &Instance) -> ColoredString {
    let path_to_prop = instance.dependency_type.path.replace("/", ".");
    let file_link = self.package_json_link(&instance.package.borrow());
    format!("in {file_link} at {path_to_prop}").normal()
  }

  /// Issues related to whether a specifier is the highest or lowest semver are
  /// all the same logic internally, so we have combined enum branches for them.
  ///
  /// From an end user point of view though it is clearer to have a specific
  /// status code related to what has happened.
  fn to_public_status_code(group_variant: &VersionGroupVariant, code: &str) -> ColoredString {
    if matches!(group_variant, VersionGroupVariant::HighestSemver) {
      code.replace("HighestOrLowestSemver", "HighestSemver").normal()
    } else if matches!(group_variant, VersionGroupVariant::LowestSemver) {
      code.replace("HighestOrLowestSemver", "LowestSemver").normal()
    } else {
      code.normal()
    }
  }

  pub fn print_formatted_packages(&self, packages: Vec<&Rc<RefCell<PackageJson>>>) {
    if !packages.is_empty() {
      let icon = self.ok_icon();
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
  fn print_formatted_package(&self, package: &PackageJson) {
    if package.formatting_mismatches.borrow().is_empty() {
      let icon = "-".dimmed();
      let file_link = self.package_json_link(package);
      info!("      {icon} {file_link}");
    }
  }

  /// Print every package.json which has the given formatting mismatch
  pub fn print_formatting_mismatches(&self, variant: &FormatMismatchVariant, mismatches: &[Rc<FormatMismatch>]) {
    let count = self.count_column(mismatches.len());
    let icon = self.err_icon();
    let status_code = format!("{:?}", variant);
    let link = self.status_code_link(&status_code).red();
    info!("{count} {icon} {link}");
    if self.show_packages {
      mismatches
        .iter()
        .sorted_by(|a, b| a.package.borrow().get_name_unsafe().cmp(&b.package.borrow().get_name_unsafe()))
        .for_each(|mismatch| {
          let icon = "-".dimmed();
          let package = mismatch.package.borrow();
          let property_path = self.format_path(&mismatch.property_path).dimmed();
          let file_link = self.package_json_link(&package);
          let in_ = "in".dimmed();
          let at = "at".dimmed();
          let msg = format!("      {icon} {in_} {file_link} {at} {property_path}");
          info!("{msg}");
        });
    }
  }

  /// Render a clickable link to a package.json file
  fn package_json_link(&self, package: &PackageJson) -> ColoredString {
    let name = package.get_name_unsafe();
    let file_path = package.file_path.to_str().unwrap();
    let plain_link = self.link(format!("file:{file_path}"), name);
    format!("{plain_link}").normal()
  }

  fn status_code_link(&self, pascal_case: &str) -> ColoredString {
    if !self.show_status_codes {
      return "".normal();
    }
    let base_url = "https://jamiemason.github.io/syncpack/guide/status-codes/";
    let lower_case = pascal_case.to_lowercase();
    let plain_link = self.link(format!("{base_url}#{lower_case}"), pascal_case);
    format!("{plain_link}").normal()
  }

  /// Render a clickable link
  fn link(&self, url: impl Into<String>, text: impl Into<ColoredString>) -> ColoredString {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url.into(), text.into()).normal()
  }

  /// Convert eg. "/dependencies/react" to ".dependencies.react"
  fn format_path(&self, path: &str) -> ColoredString {
    let path = path.replace("/", ".");
    path.normal()
  }

  /*
  fn err(&self, msg: &str) -> ColoredString {
    format!("{} {}", self.red_cross(), msg).red()
  }

  fn warn(&self, msg: &str) -> ColoredString {
    format!("{} {}", self.yellow_warning(), msg).yellow()
  }


  fn print_dependency_header(&self, dependency: &Dependency) {
    let state = dependency.get_state();
    let count = self.count_column(dependency.instances.borrow().len());
    let status_code = self.get_dependency_status_code(dependency);
    if matches!(state, InstanceState::Valid(ValidInstance::IsIgnored)) {
      let icon = "?".dimmed();
      let name = &dependency.name;
      return info!("{count} {icon} {name} {status_code}");
    }
    let name = if matches!(state, InstanceState::Invalid(_)) {
      dependency.name.red()
    } else {
      dependency.name.normal()
    };
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
    let has_issue = matches!(
      state,
      InstanceState::Invalid(_) | InstanceState::Suspect(_) | InstanceState::Valid(ValidInstance::IsIgnored)
    );
    let will_be_shown_by_every_instance = self.show_instances;
    if has_issue && !will_be_shown_by_every_instance {
      self.instance_state_link2(&state)
    } else {
      "".normal()
    }
  }

  fn dependency_state_link(&self, dependency: &Dependency) -> ColoredString {
    self.status_code_link(&dependency.get_state().get_name())
  }

  fn instance_state_link(&self, instance: &Instance) -> ColoredString {
    self.status_code_link(&instance.state.borrow().get_name())
  }

  fn instance_state_link2(&self, instance_state: &InstanceState) -> ColoredString {
    self.status_code_link(&instance_state.get_name())
  }


  fn state_icon(&self, state: &InstanceState) -> ColoredString {
    match state {
      InstanceState::Valid(variant) => self.green_tick(),
      InstanceState::Invalid(_) => self.red_cross(),
      InstanceState::Suspect(_) => self.yellow_warning(),
      InstanceState::Unknown => panic!("Unknown state"),
    }
  }


  fn print_instances(&self, instances: &[Rc<Instance>]) {
    if self.show_instances {
      instances
        .iter()
        .sorted_unstable_by_key(|instance| (instance.actual_specifier.unwrap(), &instance.name, &instance.dependency_type.path))
        .rev()
        .for_each(|instance| self.print_instance(instance))
    }
  }

  fn specifier_with_icon(&self, instance: &Instance) -> ColoredString {
    let state = instance.state.borrow().clone();
    let specifier = instance.actual_specifier.unwrap();
    match &state {
      InstanceState::Valid(variant) => {
        let icon = self.green_tick();
        format!("{icon} {specifier}").green()
      }
      InstanceState::Invalid(_) => {
        let icon = self.red_cross();
        format!("{icon} {specifier}").red()
      }
      InstanceState::Suspect(_) => {
        let icon = self.yellow_warning();
        format!("{icon} {specifier}").yellow()
      }
      InstanceState::Unknown => "".normal(),
    }
  }
  */

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
