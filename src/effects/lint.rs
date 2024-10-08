use colored::*;
use log::info;

use crate::{context::Context, effects::ui::Ui, version_group::VersionGroupVariant};

/// Run the lint command side effects
pub fn run(ctx: Context) -> Context {
  // @TODO: move values to config file
  let ui = Ui {
    show_ignored: true,
    show_instances: true,
    show_status_codes: true,
  };

  if ctx.config.cli.options.versions {
    info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
    ctx.version_groups.iter().for_each(|group| {
      ui.print_group_header(group);
      group.dependencies.borrow().values().for_each(|dependency| {
        dependency.sort_instances();
        match dependency.variant {
          VersionGroupVariant::Banned => {
            ui.print_dependency_header(dependency);
            dependency.instances.borrow().iter().for_each(|instance| {
              ui.print_instance_link(instance);
            });
          }
          VersionGroupVariant::HighestSemver | VersionGroupVariant::LowestSemver => {
            ui.print_dependency_header(dependency);
            dependency.instances.borrow().iter().for_each(|instance| {
              ui.print_instance_link(instance);
            });
          }
          VersionGroupVariant::Ignored => {
            ui.print_dependency_header(dependency);
            dependency.instances.borrow().iter().for_each(|instance| {
              ui.print_instance_link(instance);
            });
          }
          VersionGroupVariant::Pinned => {
            ui.print_dependency_header(dependency);
            dependency.instances.borrow().iter().for_each(|instance| {
              ui.print_instance_link(instance);
            });
          }
          VersionGroupVariant::SameRange => {
            ui.print_dependency_header(dependency);
            dependency.instances.borrow().iter().for_each(|instance| {
              ui.print_instance_link(instance);
            });
          }
          VersionGroupVariant::SnappedTo => {
            ui.print_dependency_header(dependency);
            dependency.instances.borrow().iter().for_each(|instance| {
              ui.print_instance_link(instance);
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
