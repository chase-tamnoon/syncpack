use colored::*;
use log::info;

use crate::{
  context::Context,
  version_group::{Variant, VersionGroup},
};

/// Run the lint command side effects
pub fn run(ctx: Context) -> Context {
  if ctx.config.cli.options.versions {
    info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
    ctx.version_groups.iter().for_each(|group| {
      on_group_visited(group);
      group.dependencies.borrow().values().for_each(|dependency| {
        match dependency.variant {
          Variant::Banned => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          Variant::HighestSemver | Variant::LowestSemver => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          Variant::Ignored => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          Variant::Pinned => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          Variant::SameRange => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          Variant::SnappedTo => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
        }
      });
    });
  }
  if ctx.config.cli.options.format {
    info!("{}", "= FORMATTING".dimmed());
    ctx.packages.by_name.values().for_each(|package| {
      //
    });
  }
  ctx
}

fn on_group_visited(group: &VersionGroup) {
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

// fn on_dependency_valid(dependency: &Dependency) {
//   let count = render_count_column(dependency.instances.borrow().len());
//   let name = &dependency.name;
//   let hint = get_expected_hint(dependency);
//   info!("{count} {name} {hint}");
// }

// fn on_dependency_invalid(dependency: &Dependency) {
//   let count = render_count_column(dependency.instances.borrow().len());
//   let name = &dependency.name;
//   let hint = get_expected_hint(dependency);
//   info!("{count} {name} {hint}");
// }

// fn on_dependency_warning(dependency: &Dependency) {
//   let count = render_count_column(dependency.instances.borrow().len());
//   let name = &dependency.name;
//   let hint = "has name or specifiers unsupported by syncpack".dimmed();
//   info!("{count} {name} {hint}");
// }

// fn on_package_format_match(package: &PackageJson) {
//   let file_path = package.borrow().get_relative_file_path(&self.config.cwd);
//   info!("{} {file_path}", icon_valid());
// }

// fn on_package_format_mismatch(package: &PackageJson) {
//   let file_path = package.borrow().get_relative_file_path(&self.config.cwd);
//   info!("{} {file_path}", icon_fixable());
//   event.formatting_mismatches.iter().for_each(|mismatch| {
//     let property_path = &mismatch.property_path.dimmed();
//     let expected = &mismatch.expected;
//     match &mismatch.variant {
//       FormatMismatchVariant::BugsPropertyIsNotFormatted => {
//         let message = "is not in shorthand format".dimmed();
//         info!("  {property_path} {message}");
//       }
//       FormatMismatchVariant::RepositoryPropertyIsNotFormatted => {
//         let message = "is not in shorthand format".dimmed();
//         info!("  {property_path} {message}");
//       }
//       FormatMismatchVariant::ExportsPropertyIsNotSorted => {
//         let message = "is not sorted".dimmed();
//         info!("  {property_path} {message}");
//       }
//       FormatMismatchVariant::PropertyIsNotSortedAz => {
//         let message = "is not sorted alphabetically".dimmed();
//         info!("  {property_path} {message}");
//       }
//       FormatMismatchVariant::PackagePropertiesAreNotSorted => {
//         let message = "root properties are not sorted".dimmed();
//         info!("  {message}");
//       }
//     }
//   });
//   self.is_valid = false;
// }

// fn on_exit_command() {
//   if self.is_valid {
//     info!("\n{} {}", icon_valid(), "valid");
//   } else {
//     info!("\n{} {}", icon_fixable(), "invalid");
//   }
// }

// fn on_instance(&mut self, event: InstanceEvent) {
//   let instance = &event.instance;
//   let dependency = &event.dependency;
//   match &event.variant {
//     InstanceState::Unknown => {
//       info!("@TODO: InstanceState::Unknown '{}'", instance.id);
//     }
//     /* Ignored */
//     InstanceState::Ignored => { /*NOOP*/ }
//     /* Matches */
//     InstanceState::ValidLocal
//     | InstanceState::EqualsLocal
//     | InstanceState::MatchesLocal
//     | InstanceState::EqualsPreferVersion
//     | InstanceState::EqualsSnapToVersion
//     | InstanceState::EqualsNonSemverPreferVersion
//     | InstanceState::EqualsPin
//     | InstanceState::MatchesSameRangeGroup => {
//       // return /*SKIP*/;
//       let icon = icon_valid();
//       let actual = instance.actual_specifier.unwrap().green();
//       let location_hint = instance.location_hint.dimmed();
//       info!("      {icon} {actual} {location_hint}");
//     }
//     /* Warnings */
//     InstanceState::RefuseToBanLocal => {
//       info!("@TODO: explain RefuseToBanLocal");
//     }
//     InstanceState::RefuseToPinLocal => {
//       info!("@TODO: explain RefuseToPinLocal");
//     }
//     InstanceState::RefuseToSnapLocal => {
//       info!("@TODO: explain RefuseToSnapLocal");
//     }
//     InstanceState::InvalidLocalVersion => {
//       info!("@TODO: explain InvalidLocalVersion");
//     }
//     InstanceState::MatchesPreferVersion => {
//       // return /*SKIP*/;
//       let icon = icon_fixable();
//       let actual = instance.actual_specifier.unwrap().red();
//       let high_low = high_low_hint(&dependency.variant);
//       let opposite = if matches!(dependency.variant, Variant::HighestSemver) {
//         "lower"
//       } else {
//         "higher"
//       };
//       let hint =
//         format!("is {high_low} but mismatches its semver group, fixing its semver group would cause its version to be {opposite}").dimmed();
//       let location_hint = instance.location_hint.dimmed();
//       info!("      {icon} {actual} {hint} {location_hint}");
//       self.is_valid = false;
//     }
//     InstanceState::MatchesSnapToVersion => {
//       info!("@TODO: explain MatchesSnapToVersion");
//     }
//     /* Overrides */
//     InstanceState::PinMatchOverridesSemverRangeMatch => {
//       info!("@TODO: explain PinMatchOverridesSemverRangeMatch");
//     }
//     InstanceState::PinMatchOverridesSemverRangeMismatch => {
//       info!("@TODO: explain PinMatchOverridesSemverRangeMismatch");
//     }
//     /* Fixable Mismatches */
//     InstanceState::Banned => {
//       // return /*SKIP*/;
//       let icon = icon_fixable();
//       let hint = "banned".red();
//       let location_hint = instance.location_hint.dimmed();
//       info!("      {icon} {hint} {location_hint}");
//       self.is_valid = false;
//     }
//     InstanceState::MismatchesLocal => {
//       info!("@TODO: explain MismatchesLocal");
//     }
//     InstanceState::MismatchesPreferVersion => {
//       // return /*SKIP*/;
//       let icon = icon_fixable();
//       let actual = instance.actual_specifier.unwrap().red();
//       let location_hint = instance.location_hint.dimmed();
//       info!("      {icon} {actual} {location_hint}");
//       self.is_valid = false;
//     }
//     InstanceState::MismatchesSnapToVersion => {
//       info!("@TODO: explain MismatchesSnapToVersion");
//     }
//     InstanceState::MismatchesPin => {
//       // return /*SKIP*/;
//       let icon = icon_fixable();
//       let actual = instance.actual_specifier.unwrap().red();
//       let location_hint = instance.location_hint.dimmed();
//       info!("      {icon} {actual} {location_hint}");
//       self.is_valid = false;
//     }
//     InstanceState::SemverRangeMismatch => {
//       info!("@TODO: explain SemverRangeMismatch");
//     }
//     /* Conflicts */
//     InstanceState::SemverRangeMatchConflictsWithPreferVersion => {
//       info!("@TODO: explain SemverRangeMatchConflictsWithPreferVersion");
//     }
//     InstanceState::SemverRangeMismatchConflictsWithPreferVersion => {
//       info!("@TODO: explain SemverRangeMismatchConflictsWithPreferVersion");
//     }
//     InstanceState::SemverRangeMatchConflictsWithSnapToVersion => {
//       info!("@TODO: explain SemverRangeMatchConflictsWithSnapToVersion");
//     }
//     InstanceState::SemverRangeMismatchConflictsWithSnapToVersion => {
//       info!("@TODO: explain SemverRangeMismatchConflictsWithSnapToVersion");
//     }
//     InstanceState::SemverRangeMatchConflictsWithLocalVersion => {
//       info!("@TODO: explain SemverRangeMatchConflictsWithLocalVersion");
//     }
//     InstanceState::SemverRangeMismatchConflictsWithLocalVersion => {
//       info!("@TODO: explain SemverRangeMismatchConflictsWithLocalVersion");
//     }
//     /* Unfixable Mismatches */
//     InstanceState::MismatchesInvalidLocalVersion => {
//       info!("@TODO: explain MismatchesInvalidLocalVersion");
//     }
//     InstanceState::MismatchesNonSemverPreferVersion => {
//       // return /*SKIP*/;
//       let icon = icon_unfixable();
//       let actual = instance.actual_specifier.unwrap().red();
//       let location_hint = instance.location_hint.dimmed();
//       info!("      {icon} {actual} {location_hint}");
//       self.is_valid = false;
//     }
//     InstanceState::MismatchesSameRangeGroup => {
//       info!("@TODO: explain MismatchesSameRangeGroup");
//     }
//     InstanceState::SnapToVersionNotFound => {
//       info!("@TODO: explain SnapToVersionNotFound");
//     }
//   }
// }

// /// Return a right aligned column of a count of instances
// /// Example "    38x"
// pub fn render_count_column(count: usize) -> ColoredString {
//   format!("{: >4}x", count).dimmed()
// }

// fn high_low_hint(variant: &Variant) -> &str {
//   let is_highest = matches!(variant, Variant::HighestSemver);
//   if is_highest {
//     "highest semver"
//   } else {
//     "lowest semver"
//   }
// }

// pub fn icon_valid() -> ColoredString {
//   "✓".green()
// }

// pub fn icon_fixable() -> ColoredString {
//   "✘".red()
// }

// fn icon_unfixable() -> ColoredString {
//   "✘".red()
// }

// fn icon_arrow() -> ColoredString {
//   "→".dimmed()
// }

// // @TODO: write a .resolution enum on Dependency in visit_packages instead
// fn get_expected_hint(dependency: &Dependency) -> ColoredString {
//   match dependency.expected.borrow().clone() {
//     Some(specifier) => match dependency.variant {
//       Variant::Banned => {
//         panic!("Banned should not have an expected specifier");
//       }
//       Variant::HighestSemver => {
//         let specifier = specifier.unwrap().green();
//         let label = "is highest semver".dimmed();
//         format!("{specifier} {label}").normal()
//       }
//       Variant::Ignored => "".to_string().dimmed(),
//       Variant::LowestSemver => {
//         let specifier = specifier.unwrap().green();
//         let label = "is lowest semver".dimmed();
//         format!("{specifier} {label}").normal()
//       }
//       Variant::Pinned => {
//         let label = "is pinned to".dimmed();
//         let specifier = specifier.unwrap().green();
//         format!("{label} {specifier}").normal()
//       }
//       Variant::SameRange => {
//         panic!("SameRange should not have an expected specifier");
//       }
//       Variant::SnappedTo => {
//         // @TODO: "is snapped to 0.1.4 from /devDependencies of @foo/numberwang"
//         let label = "is snapped to".dimmed();
//         let specifier = specifier.unwrap().green();
//         format!("{label} {specifier}").normal()
//       }
//     },
//     None => match dependency.variant {
//       Variant::Banned => "is banned".dimmed(),
//       Variant::SameRange => "requires all ranges to satisfy each other".dimmed(),
//       Variant::HighestSemver | Variant::LowestSemver => "has non semver mismatches syncpack cannot fix".dimmed(),
//       _ => {
//         panic!("{} ({:?}) should have an expected specifier", dependency.name, dependency.variant);
//       }
//     },
//   }
// }
