use {
  crate::{context::Context, version_group::VersionGroupVariant},
  colored::*,
  log::info,
};

/// Run the fix command side effects
pub fn run(ctx: Context) -> Context {
  if ctx.config.cli.options.versions {
    info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
    ctx.version_groups.iter().for_each(|group| {
      group.dependencies.borrow().values().for_each(|dependency| {
        match dependency.variant {
          VersionGroupVariant::Banned => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          VersionGroupVariant::HighestSemver | VersionGroupVariant::LowestSemver => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          VersionGroupVariant::Ignored => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          VersionGroupVariant::Pinned => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          VersionGroupVariant::SameRange => {
            dependency.instances.borrow().iter().for_each(|instance| {
              //
            });
          }
          VersionGroupVariant::SnappedTo => {
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
    ctx.packages.all.iter().for_each(|package| {
      //
    });
  }
  ctx
}

// impl Effects for FixEffects<'_> {
//   fn on(&mut self, event: Event) {
//     match &event {
//       Event::EnterVersionsAndRanges => {}
//       Event::EnterFormat => {}
//       Event::GroupVisited(group) => {
//         let print_width = 80;
//         let label = &group.label;
//         let header = format!("= {label} ");
//         let divider = if header.len() < print_width {
//           "=".repeat(print_width - header.len())
//         } else {
//           "".to_string()
//         };
//         let full_header = format!("{header}{divider}");
//         info!("{}", full_header.blue());
//       }
//       Event::DependencyValid(_) => { /*NOOP*/ }
//       Event::DependencyInvalid(_) => { /*NOOP*/ }
//       Event::DependencyWarning(_) => { /*NOOP*/ }
//       Event::PackageFormatMatch(_) => {
//         // @TODO
//       }
//       Event::PackageFormatMismatch(event) => {
//         let file_path =
// event.package.borrow().get_relative_file_path(&self.config.cwd);
//         event.formatting_mismatches.iter().for_each(|mismatch| {
//           let property_path = &mismatch.property_path;
//           let expected = &mismatch.expected;
//           match &mismatch.variant {
//             FormatMismatchVariant::BugsPropertyIsNotFormatted
//             | FormatMismatchVariant::RepositoryPropertyIsNotFormatted
//             | FormatMismatchVariant::ExportsPropertyIsNotSorted
//             | FormatMismatchVariant::PropertyIsNotSortedAz
//             | FormatMismatchVariant::PackagePropertiesAreNotSorted => {
//               event
//                 .package
//                 .borrow_mut()
//                 .set_prop(mismatch.property_path.as_str(),
// mismatch.expected.clone());             }
//           }
//         });
//       }
//       Event::ExitCommand => {
//         for package in self.packages.by_name.values() {
//           package.borrow().write_to_disk(self.config);
//         }
//         if self.is_valid {
//           let icon = icon_valid();
//           info!("\n{icon} valid");
//         } else {
//           let icon = icon_fixable();
//           info!("\n{icon} invalid");
//         }
//       }
//     }
//   }

//   fn on_instance(&mut self, event: InstanceEvent) {
//     let instance = &event.instance;
//     let dependency = &event.dependency;
//     match &event.variant {
//       InstanceState::Unknown => {
//         panic!("Unknown instance state");
//       }
//       /* Ignored */
//       InstanceState::Ignored => { /*NOOP*/ }
//       /* Matches */
//       InstanceState::ValidLocal
//       | InstanceState::EqualsLocal
//       | InstanceState::MatchesLocal
//       | InstanceState::EqualsPreferVersion
//       | InstanceState::EqualsSnapToVersion
//       | InstanceState::EqualsNonSemverPreferVersion
//       | InstanceState::EqualsPin
//       | InstanceState::MatchesSameRangeGroup => { /*NOOP*/ }
//       /* Warnings */
//       InstanceState::RefuseToBanLocal => {
//         debug!("@TODO: explain RefuseToBanLocal");
//       }
//       InstanceState::RefuseToPinLocal => {
//         debug!("@TODO: explain RefuseToPinLocal");
//       }
//       InstanceState::InvalidLocalVersion => {
//         debug!("@TODO: explain InvalidLocalVersion");
//       }
//       InstanceState::MatchesPreferVersion => {
//         debug!("@TODO: explain MatchesPreferVersion");
//       }
//       InstanceState::MatchesSnapToVersion => {
//         debug!("@TODO: explain MatchesSnapToVersion");
//       }
//       InstanceState::RefuseToSnapLocal => {
//         debug!("@TODO: explain RefuseToSnapLocal");
//       }
//       /* Overrides */
//       InstanceState::PinMatchOverridesSemverRangeMatch => {
//         debug!("@TODO: explain PinMatchOverridesSemverRangeMatch");
//       }
//       InstanceState::PinMatchOverridesSemverRangeMismatch => {
//         debug!("@TODO: explain PinMatchOverridesSemverRangeMismatch");
//       }
//       /* Fixable Mismatches */
//       InstanceState::Banned
//       | InstanceState::MismatchesLocal
//       | InstanceState::MismatchesPreferVersion
//       | InstanceState::MismatchesSnapToVersion
//       | InstanceState::SemverRangeMismatch
//       | InstanceState::MismatchesPin => {
//         instance.package.borrow().copy_expected_specifier(instance);
//       }
//       /* Conflicts */
//       InstanceState::SemverRangeMatchConflictsWithPreferVersion => {
//         debug!("@TODO: explain SemverRangeMatchConflictsWithPreferVersion");
//         self.is_valid = false;
//       }
//       InstanceState::SemverRangeMismatchConflictsWithPreferVersion => {
//         debug!("@TODO: explain
// SemverRangeMismatchConflictsWithPreferVersion");         self.is_valid =
// false;       }
//       InstanceState::SemverRangeMatchConflictsWithSnapToVersion => {
//         debug!("@TODO: explain SemverRangeMatchConflictsWithSnapToVersion");
//         self.is_valid = false;
//       }
//       InstanceState::SemverRangeMismatchConflictsWithSnapToVersion => {
//         debug!("@TODO: explain
// SemverRangeMismatchConflictsWithSnapToVersion");         self.is_valid =
// false;       }
//       InstanceState::SemverRangeMatchConflictsWithLocalVersion => {
//         debug!("@TODO: explain SemverRangeMatchConflictsWithLocalVersion");
//         self.is_valid = false;
//       }
//       InstanceState::SemverRangeMismatchConflictsWithLocalVersion => {
//         debug!("@TODO: explain
// SemverRangeMismatchConflictsWithLocalVersion");         self.is_valid =
// false;       }
//       /* Unfixable Mismatches */
//       InstanceState::MismatchesInvalidLocalVersion => {
//         debug!("@TODO: explain MismatchesInvalidLocalVersion");
//         self.is_valid = false;
//       }
//       InstanceState::MismatchesNonSemverPreferVersion => {
//         debug!("@TODO: explain MismatchesNonSemverPreferVersion");
//         self.is_valid = false;
//       }
//       InstanceState::MismatchesSameRangeGroup => {
//         debug!("@TODO: explain MismatchesSameRangeGroup");
//         self.is_valid = false;
//       }
//       InstanceState::SnapToVersionNotFound => {
//         debug!("@TODO: explain SnapToVersionNotFound");
//         self.is_valid = false;
//       }
//     }
//   }
// }
