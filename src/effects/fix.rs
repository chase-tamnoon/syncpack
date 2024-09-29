use colored::*;
use log::info;

use crate::{
  config::Config,
  context::InstancesById,
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
  pub packages: Option<Packages>,
}

impl<'a> FixEffects<'a> {
  pub fn new(config: &'a Config) -> Self {
    Self {
      config,
      is_valid: true,
      packages: None,
    }
  }
}

impl Effects for FixEffects<'_> {
  fn get_packages(&mut self) -> Packages {
    let packages = self.packages.take().unwrap();
    self.packages = None;
    packages
  }

  fn set_packages(&mut self, packages: Packages) {
    self.packages = Some(packages);
  }

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
        let packages = self.packages.as_mut().unwrap();
        let package = packages.by_name.get_mut(&event.package_name).unwrap();
        let file_path = package.get_relative_file_path(&self.config.cwd);

        event.formatting_mismatches.iter().for_each(|mismatch| {
          let property_path = &mismatch.property_path;
          let expected = &mismatch.expected;
          match &mismatch.variant {
            FormatEventVariant::BugsPropertyIsNotFormatted => {
              package.set_prop(mismatch.property_path.as_str(), mismatch.expected.clone());
            }
            FormatEventVariant::RepositoryPropertyIsNotFormatted => {
              package.set_prop(mismatch.property_path.as_str(), mismatch.expected.clone());
            }
            FormatEventVariant::ExportsPropertyIsNotSorted => {
              package.set_prop(mismatch.property_path.as_str(), mismatch.expected.clone());
            }
            FormatEventVariant::PropertyIsNotSortedAz => {
              package.set_prop(mismatch.property_path.as_str(), mismatch.expected.clone());
            }
            FormatEventVariant::PackagePropertiesAreNotSorted => {
              package.set_prop(mismatch.property_path.as_str(), mismatch.expected.clone());
            }
          }
        });
      }
      Event::ExitCommand => {
        let mut packages = self.get_packages();
        for package in packages.by_name.values_mut() {
          package.write_to_disk(self.config);
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

  fn on_instance(&mut self, event: InstanceEvent, instances_by_id: &mut InstancesById) {
    let instance_id = &event.instance_id;
    let dependency = &event.dependency;
    match &event.variant {
      /* Ignored */
      InstanceEventVariant::InstanceIsIgnored => { /*NOOP*/ }
      /* Matches */
      InstanceEventVariant::LocalInstanceIsPreferred
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
        let instance = instances_by_id.get_mut(instance_id).unwrap();
        let packages = self.packages.as_mut().unwrap();
        let package = packages.by_name.get_mut(&instance.package_name).unwrap();
        instance.set_specifier(package, &instance.expected.clone());
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
