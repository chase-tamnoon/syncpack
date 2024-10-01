use colored::*;
use log::info;

use crate::{
  config::Config,
  effects::{
    lint::{icon_fixable, icon_valid},
    Effects, Event, InstanceEvent, InstanceEventVariant,
  },
  packages::Packages,
};

use super::FormatEventVariant;

/// The implementation of the `fix` command's side effects
pub struct FixEffects<'a> {
  pub config: &'a Config,
  pub is_valid: bool,
  pub packages: &'a Packages,
}

impl<'a> FixEffects<'a> {
  pub fn new(config: &'a Config, packages: &'a Packages) -> Self {
    Self {
      config,
      is_valid: true,
      packages,
    }
  }
}

impl Effects for FixEffects<'_> {
  fn on(&mut self, event: Event) {
    match &event {
      Event::EnterVersionsAndRanges => {
        info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
      }
      Event::EnterFormat => {
        info!("{}", "= FORMATTING".dimmed());
      }
      Event::GroupVisited(group) => {
        let print_width = 80;
        let label = &group.label;
        let header = format!("= {label} ");
        let divider = if header.len() < print_width {
          "=".repeat(print_width - header.len())
        } else {
          "".to_string()
        };
        let full_header = format!("{header}{divider}");
        info!("{}", full_header.blue());
      }
      Event::DependencyValid(dependency, expected) => { /*NOOP*/ }
      Event::DependencyInvalid(dependency, expected) => { /*NOOP*/ }
      Event::DependencyWarning(dependency, expected) => { /*NOOP*/ }
      Event::PackageFormatMatch(_) => {
        // @TODO
      }
      Event::PackageFormatMismatch(event) => {
        let file_path = event.package.borrow().get_relative_file_path(&self.config.cwd);
        event.formatting_mismatches.iter().for_each(|mismatch| {
          let property_path = &mismatch.property_path;
          let expected = &mismatch.expected;
          match &mismatch.variant {
            FormatEventVariant::BugsPropertyIsNotFormatted
            | FormatEventVariant::RepositoryPropertyIsNotFormatted
            | FormatEventVariant::ExportsPropertyIsNotSorted
            | FormatEventVariant::PropertyIsNotSortedAz
            | FormatEventVariant::PackagePropertiesAreNotSorted => {
              event
                .package
                .borrow_mut()
                .set_prop(mismatch.property_path.as_str(), mismatch.expected.clone());
            }
          }
        });
      }
      Event::ExitCommand => {
        for package in self.packages.by_name.values() {
          package.borrow().write_to_disk(self.config);
        }
        if self.is_valid {
          let icon = icon_valid();
          info!("\n{icon} valid");
        } else {
          let icon = icon_fixable();
          info!("\n{icon} invalid");
        }
      }
    }
  }

  fn on_instance(&mut self, event: InstanceEvent) {
    let instance = &event.instance;
    let dependency = &event.dependency;
    match &event.variant {
      /* Ignored */
      InstanceEventVariant::InstanceIsIgnored => { /*NOOP*/ }
      /* Matches */
      InstanceEventVariant::LocalInstanceIsValid
      | InstanceEventVariant::InstanceMatchesLocal
      | InstanceEventVariant::InstanceMatchesHighestOrLowestSemver
      | InstanceEventVariant::InstanceMatchesButIsUnsupported
      | InstanceEventVariant::InstanceMatchesPinned
      | InstanceEventVariant::InstanceMatchesSameRangeGroup => { /*NOOP*/ }
      /* Warnings */
      InstanceEventVariant::LocalInstanceMistakenlyBanned => {
        println!("@TODO: explain LocalInstanceMistakenlyBanned");
      }
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup => {
        println!("@TODO: explain LocalInstanceMistakenlyMismatchesSemverGroup");
      }
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned => {
        println!("@TODO: explain LocalInstanceMistakenlyMismatchesPinned");
      }
      InstanceEventVariant::LocalInstanceWithMissingVersion => {
        println!("@TODO: explain LocalInstanceWithMissingVersion");
      }
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup => {
        println!("@TODO: explain InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup");
      }
      /* Fixable Mismatches */
      InstanceEventVariant::InstanceIsBanned
      | InstanceEventVariant::InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed
      | InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup
      | InstanceEventVariant::InstanceMismatchesLocal
      | InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver
      | InstanceEventVariant::InstanceMismatchesPinned => {
        instance.package.borrow().apply_instance_specifier(instance);
      }
      /* Unfixable Mismatches */
      InstanceEventVariant::InstanceMismatchesLocalWithMissingVersion => {
        println!("@TODO: explain InstanceMismatchesLocalWithMissingVersion");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMismatchesAndIsUnsupported => {
        println!("@TODO: explain InstanceMismatchesAndIsUnsupported");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup => {
        println!("@TODO: explain InstanceMatchesPinnedButMismatchesSemverGroup");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups => {
        println!("@TODO: explain InstanceMismatchesBothSameRangeAndConflictingSemverGroups");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups => {
        println!("@TODO: explain InstanceMismatchesBothSameRangeAndCompatibleSemverGroups");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup => {
        println!("@TODO: explain InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup => {
        println!("@TODO: explain InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMismatchesSameRangeGroup => {
        println!("@TODO: explain InstanceMismatchesSameRangeGroup");
        self.is_valid = false;
      }
    }
  }
}
