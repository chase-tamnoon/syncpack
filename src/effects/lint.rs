use colored::*;
use log::info;

use crate::{
  config::Config,
  context::InstancesById,
  dependency::Dependency,
  effects::{Effects, Event, InstanceEvent, InstanceEventVariant},
  packages::Packages,
  specifier::Specifier,
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

  fn set_packages(&mut self, packages: Packages) {
    self.packages = Some(packages);
  }

  fn on(&mut self, event: Event, instances_by_id: &mut InstancesById) {
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
      Event::DependencyValid(dependency, expected) => {
        let count = render_count_column(dependency.all.len());
        let name = &dependency.name;
        let hint = get_expected_hint(dependency, expected);
        info!("{count} {name} {hint}");
      }
      Event::DependencyInvalid(dependency, expected) => {
        let count = render_count_column(dependency.all.len());
        let name = &dependency.name;
        let hint = get_expected_hint(dependency, expected);
        info!("{count} {name} {hint}");
      }
      Event::DependencyWarning(dependency, expected) => {
        let count = render_count_column(dependency.all.len());
        let name = &dependency.name;
        let hint = "has name or specifiers unsupported by syncpack".dimmed();
        info!("{count} {name} {hint}");
      }
      Event::FormatMatch(_) => {
        // @TODO
      }
      Event::FormatMismatch(_) => {
        // @TODO
      }
      Event::ExitCommand => {
        if self.is_valid {
          info!("\n{} {}", icon_valid(), "valid");
        } else {
          info!("\n{} {}", icon_fixable(), "invalid");
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
      | InstanceEventVariant::InstanceMatchesSameRangeGroup => {
        // return /*SKIP*/;
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = icon_valid();
        let actual = instance.actual.unwrap().green();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
      }
      /* Warnings */
      InstanceEventVariant::LocalInstanceMistakenlyBanned => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  LocalInstanceMistakenlyBanned");
      }
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesSemverGroup => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  LocalInstanceMistakenlyMismatchesSemverGroup");
      }
      InstanceEventVariant::LocalInstanceMistakenlyMismatchesPinned => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  LocalInstanceMistakenlyMismatchesPinned");
      }
      InstanceEventVariant::InstanceMatchesHighestOrLowestSemverButMismatchesConflictingSemverGroup => {
        // return /*SKIP*/;
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = icon_fixable();
        let actual = instance.actual.unwrap().red();
        let high_low = high_low_hint(&dependency.variant);
        let opposite = if matches!(dependency.variant, Variant::HighestSemver) { "lower" } else { "higher" };
        let hint = format!("is {high_low} but mismatches its semver group, fixing its semver group would cause its version to be {opposite}").dimmed();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {hint} {location_hint}");
        self.is_valid = false;
      }
      /* Fixable Mismatches */
      InstanceEventVariant::InstanceIsBanned => {
        // return /*SKIP*/;
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = icon_fixable();
        let hint = "banned".red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {hint} {location_hint}");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceIsHighestOrLowestSemverOnceSemverGroupIsFixed => {
        // return /*SKIP*/;
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = icon_fixable();
        let actual = instance.actual.unwrap().red();
        let arrow = icon_arrow();
        let expected = instance.expected.unwrap().green();
        let high_low = high_low_hint(&dependency.variant);
        let hint = format!("mismatches its semver group but will be {high_low} once fixed").dimmed();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {arrow} {expected} {hint} {location_hint}");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMatchesLocalButMismatchesSemverGroup => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMatchesLocalButMismatchesSemverGroup");
      }
      InstanceEventVariant::InstanceMismatchesLocal => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMismatchesLocal");
      }
      InstanceEventVariant::InstanceMismatchesHighestOrLowestSemver => {
        // return /*SKIP*/;
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = icon_fixable();
        let actual = instance.actual.unwrap().red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMismatchesPinned => {
        // return /*SKIP*/;
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = icon_fixable();
        let actual = instance.actual.unwrap().red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
        self.is_valid = false;
      }
      /* Unfixable Mismatches */
      InstanceEventVariant::InstanceMismatchesLocalWithMissingVersion => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMismatchesLocalWithMissingVersion");
      }
      InstanceEventVariant::InstanceMismatchesAndIsUnsupported => {
        // return /*SKIP*/;
        let instance = instances_by_id.get(instance_id).unwrap();
        let icon = icon_unfixable();
        let actual = instance.actual.unwrap().red();
        let location_hint = instance.location_hint.dimmed();
        info!("      {icon} {actual} {location_hint}");
        self.is_valid = false;
      }
      InstanceEventVariant::InstanceMatchesPinnedButMismatchesSemverGroup => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMatchesPinnedButMismatchesSemverGroup");
      }
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndConflictingSemverGroups => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMismatchesBothSameRangeAndConflictingSemverGroups");
      }
      InstanceEventVariant::InstanceMismatchesBothSameRangeAndCompatibleSemverGroups => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMismatchesBothSameRangeAndCompatibleSemverGroups");
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMatchesSameRangeGroupButMismatchesConflictingSemverGroup");
      }
      InstanceEventVariant::InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMatchesSameRangeGroupButMismatchesCompatibleSemverGroup");
      }
      InstanceEventVariant::InstanceMismatchesSameRangeGroup => {
        let instance = instances_by_id.get(instance_id).unwrap();
        info!("  InstanceMismatchesSameRangeGroup");
      }
    }
  }
}

/// Return a right aligned column of a count of instances
/// Example "    38x"
pub fn render_count_column(count: usize) -> ColoredString {
  format!("{: >4}x", count).dimmed()
}

fn high_low_hint(variant: &Variant) -> &str {
  let is_highest = matches!(variant, Variant::HighestSemver);
  if is_highest {
    "highest semver"
  } else {
    "lowest semver"
  }
}

pub fn icon_valid() -> ColoredString {
  "✓".green()
}

pub fn icon_fixable() -> ColoredString {
  "✘".red()
}

fn icon_unfixable() -> ColoredString {
  "✘".red()
}

fn icon_arrow() -> ColoredString {
  "→".dimmed()
}

// @TODO: write a .resolution enum on Dependency in visit_packages instead
fn get_expected_hint(dependency: &Dependency, expected: &Option<Specifier>) -> ColoredString {
  match expected {
    Some(specifier) => match dependency.variant {
      Variant::Banned => {
        panic!("Banned should not have an expected specifier");
      }
      Variant::HighestSemver => {
        let specifier = specifier.unwrap().green();
        let label = "is highest semver".dimmed();
        format!("{specifier} {label}").normal()
      }
      Variant::Ignored => "".to_string().dimmed(),
      Variant::LowestSemver => {
        let specifier = specifier.unwrap().green();
        let label = "is lowest semver".dimmed();
        format!("{specifier} {label}").normal()
      }
      Variant::Pinned => {
        let label = "is pinned to".dimmed();
        let specifier = specifier.unwrap().green();
        format!("{label} {specifier}").normal()
      }
      Variant::SameRange => {
        panic!("SameRange should not have an expected specifier");
      }
      Variant::SnappedTo => {
        // @TODO: "is snapped to 0.1.4 from /devDependencies of @foo/numberwang"
        let label = "is snapped to".dimmed();
        let specifier = specifier.unwrap().green();
        format!("{label} {specifier}").normal()
      }
    },
    None => match dependency.variant {
      Variant::Banned => "is banned".dimmed(),
      Variant::SameRange => "requires all ranges to satisfy each other".dimmed(),
      Variant::HighestSemver | Variant::LowestSemver => {
        "has non semver mismatches syncpack cannot fix".dimmed()
      }
      _ => {
        panic!(
          "{} ({:?}) should have an expected specifier",
          dependency.name, dependency.variant
        );
      }
    },
  }
}
