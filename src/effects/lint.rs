use itertools::Itertools;

use crate::{context::Context, effects::ui::Ui};

/// Run the lint command side effects
pub fn run(ctx: Context) -> Context {
  // @TODO: move values to config file
  let ui = Ui {
    ctx: &ctx,
    show_ignored: true,
    show_instances: false,
    show_status_codes: true,
    show_packages: false,
    // @TODO: show_valid: false,
    // @TODO: sort_by: "name" | "state" | "count",
  };

  if ctx.config.cli.options.versions {
    ui.print_command_header("SEMVER RANGES AND VERSION MISMATCHES");
    ctx.version_groups.iter().for_each(|group| {
      ui.print_group_header(group);
      group.dependencies.borrow().values().for_each(|dependency| {
        dependency.sort_instances();
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
