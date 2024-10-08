use colored::*;
use log::info;

use crate::{
  dependency::{Dependency, DependencyState},
  instance::InstanceState,
  version_group::{VersionGroup, VersionGroupVariant},
};

pub fn green_tick() -> ColoredString {
  "✓".green()
}

pub fn red_cross() -> ColoredString {
  "✘".red()
}

pub fn yellow_warning() -> ColoredString {
  "!".yellow()
}

fn dim_right_arrow() -> ColoredString {
  "→".dimmed()
}

pub fn err(msg: &str) -> ColoredString {
  format!("{} {}", red_cross(), msg).red()
}

pub fn warn(msg: &str) -> ColoredString {
  format!("{} {}", yellow_warning(), msg).yellow()
}

pub fn link(url: impl Into<String>, text: impl Into<ColoredString>) -> ColoredString {
  format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url.into(), text.into()).normal()
}

pub fn group_header(group: &VersionGroup) {
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

pub fn dependency_header(dependency: &Dependency) {
  let count = render_count_column(dependency.instances.borrow().len());
  let name = &dependency.name;
  let hint = get_dependency_hint(dependency);
  info!("{count} {name} {hint}");
}

pub fn status_code_link(instance_state: &InstanceState) -> ColoredString {
  let base_url = "https://jamiemason.github.io/syncpack/guide/status-codes/";
  let branch_name = format!("{:?}", instance_state);
  let branch_name_lower_case = branch_name.to_lowercase();
  link(format!("{base_url}#{branch_name_lower_case}"), format!("[{branch_name}]").dimmed())
}

/// Return a right aligned column of a count of instances
/// Example "    38x"
fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

fn get_dependency_hint(dependency: &Dependency) -> ColoredString {
  match *dependency.state.borrow() {
    DependencyState::Valid => green_tick(),
    DependencyState::Invalid => match dependency.variant {
      VersionGroupVariant::Banned => err("banned"),
      VersionGroupVariant::HighestSemver => err("highest semver mismatch"),
      VersionGroupVariant::Ignored => "ignored".dimmed(),
      VersionGroupVariant::LowestSemver => err("lowest semver mismatch"),
      VersionGroupVariant::Pinned => err("pinned version mismatch"),
      VersionGroupVariant::SameRange => err("same range mismatch"),
      VersionGroupVariant::SnappedTo => err("snapped to mismatch"),
    },
    DependencyState::Suspect => "has unsupported name or specifier".yellow(),
  }

  // match dependency.variant {
  //   VersionGroupVariant::Banned => {
  //     panic!("Banned should not have an expected specifier");
  //   }
  //   VersionGroupVariant::HighestSemver => {
  //     let specifier = specifier.unwrap().green();
  //     let label = "is highest semver".dimmed();
  //     format!("{specifier} {label}").normal()
  //   }
  //   VersionGroupVariant::Ignored => "".to_string().dimmed(),
  //   VersionGroupVariant::LowestSemver => {
  //     let specifier = specifier.unwrap().green();
  //     let label = "is lowest semver".dimmed();
  //     format!("{specifier} {label}").normal()
  //   }
  //   VersionGroupVariant::Pinned => {
  //     let label = "is pinned to".dimmed();
  //     let specifier = specifier.unwrap().green();
  //     format!("{label} {specifier}").normal()
  //   }
  //   VersionGroupVariant::SameRange => {
  //     panic!("SameRange should not have an expected specifier");
  //   }
  //   VersionGroupVariant::SnappedTo => {
  //     // @TODO: "is snapped to 0.1.4 from /devDependencies of @foo/numberwang"
  //     let label = "is snapped to".dimmed();
  //     let specifier = specifier.unwrap().green();
  //     format!("{label} {specifier}").normal()
  //   }
  // }
}

/*
fn on_dependency_valid(dependency: &Dependency) {
  let count = render_count_column(dependency.instances.borrow().len());
  let name = &dependency.name;
  let hint = get_expected_hint(dependency);
  info!("{count} {name} {hint}");
}

fn on_dependency_invalid(dependency: &Dependency) {
  let count = render_count_column(dependency.instances.borrow().len());
  let name = &dependency.name;
  let hint = get_expected_hint(dependency);
  info!("{count} {name} {hint}");
}

fn on_dependency_warning(dependency: &Dependency) {
  let count = render_count_column(dependency.instances.borrow().len());
  let name = &dependency.name;
  let hint = "has name or specifiers unsupported by syncpack".dimmed();
  info!("{count} {name} {hint}");
}

fn on_package_format_match(package: &PackageJson) {
  let file_path = package.borrow().get_relative_file_path(&self.config.cwd);
  info!("{} {file_path}", green_tick());
}

fn on_package_format_mismatch(package: &PackageJson) {
  let file_path = package.borrow().get_relative_file_path(&self.config.cwd);
  info!("{} {file_path}", red_cross());
  event.formatting_mismatches.iter().for_each(|mismatch| {
    let property_path = &mismatch.property_path.dimmed();
    let expected = &mismatch.expected;
    match &mismatch.variant {
      FormatMismatchVariant::BugsPropertyIsNotFormatted => {
        let message = "is not in shorthand format".dimmed();
        info!("  {property_path} {message}");
      }
      FormatMismatchVariant::RepositoryPropertyIsNotFormatted => {
        let message = "is not in shorthand format".dimmed();
        info!("  {property_path} {message}");
      }
      FormatMismatchVariant::ExportsPropertyIsNotSorted => {
        let message = "is not sorted".dimmed();
        info!("  {property_path} {message}");
      }
      FormatMismatchVariant::PropertyIsNotSortedAz => {
        let message = "is not sorted alphabetically".dimmed();
        info!("  {property_path} {message}");
      }
      FormatMismatchVariant::PackagePropertiesAreNotSorted => {
        let message = "root properties are not sorted".dimmed();
        info!("  {message}");
      }
    }
  });
  self.is_valid = false;
}

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
