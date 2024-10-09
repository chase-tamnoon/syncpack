use itertools::Itertools;

use crate::{context::Context, effects::ui::Ui, version_group::VersionGroupVariant};

/// Run the lint command side effects
pub fn run(ctx: Context) -> Context {
  // @TODO: move values to config file
  let ui = Ui {
    ctx: &ctx,
    show_ignored: true,
    show_instances: false,
    show_status_codes: true,
    show_packages: false,
  };

  if ctx.config.cli.options.versions {
    ui.print_command_header("SEMVER RANGES AND VERSION MISMATCHES");
    ctx.version_groups.iter().for_each(|group| {
      ui.print_group_header(group);
      group.dependencies.borrow().values().for_each(|dependency| {
        dependency.sort_instances();
        match dependency.variant {
          VersionGroupVariant::Banned => {
            ui.print_dependency_header(dependency);
            ui.print_instances(&dependency.instances.borrow());
          }
          VersionGroupVariant::HighestSemver | VersionGroupVariant::LowestSemver => {
            ui.print_dependency_header(dependency);
            ui.print_instances(&dependency.instances.borrow());
          }
          VersionGroupVariant::Ignored => {
            if ui.show_ignored {
              ui.print_dependency_header(dependency);
              ui.print_instances(&dependency.instances.borrow());
            }
          }
          VersionGroupVariant::Pinned => {
            ui.print_dependency_header(dependency);
            ui.print_instances(&dependency.instances.borrow());
          }
          VersionGroupVariant::SameRange => {
            ui.print_dependency_header(dependency);
            ui.print_instances(&dependency.instances.borrow());
          }
          VersionGroupVariant::SnappedTo => {
            ui.print_dependency_header(dependency);
            ui.print_instances(&dependency.instances.borrow());
          }
        }
      });
    });
  }
  if ctx.config.cli.options.format {
    ui.print_command_header("FORMATTING");
    let formatted_packages = ctx
      .packages
      .by_name
      .values()
      .filter(|package| package.borrow().formatting_mismatches.borrow().is_empty())
      .sorted_by(|a, b| b.borrow().get_name_unsafe().cmp(&a.borrow().get_name_unsafe()))
      .collect_vec();
    ui.print_formatted_packages(formatted_packages);
    ctx
      .formatting_mismatches_by_variant
      .borrow()
      .iter()
      .for_each(|(variant, mismatches)| {
        ui.print_formatting_mismatches(variant, mismatches);
      });
  }
  ctx
}
