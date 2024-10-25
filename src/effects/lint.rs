use itertools::Itertools;

use crate::{context::Context, effects::ui::Ui, version_group::VersionGroupVariant};

/// Run the lint command side effects
pub fn run(ctx: Context) -> Context {
  // @TODO: move values to config file
  let ui = Ui {
    ctx: &ctx,
    show_ignored: false,
    show_instances: true,
    show_status_codes: true,
    show_packages: false,
    // @TODO: show_valid: false,
    // @TODO: sort_by: "name" | "state" | "count",
  };

  if ctx.config.cli.options.versions {
    ui.print_command_header("SEMVER RANGES AND VERSION MISMATCHES");
    ctx.version_groups.iter().for_each(|group| {
      if !ui.show_ignored && matches!(group.variant, VersionGroupVariant::Ignored) {
        return;
      }
      ui.print_group_header(group);
      group.dependencies.borrow().values().for_each(|dependency| {
        ui.print_dependency(dependency, &group.variant);
        ui.for_each_instance(dependency, |instance| {
          if ui.show_instances {
            ui.print_instance(instance, &group.variant);
          }
        });
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
