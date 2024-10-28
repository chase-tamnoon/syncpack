use {
  crate::{context::Context, effects::ui::Ui, version_group::VersionGroupVariant},
  itertools::Itertools,
};

/// Run the lint command side effects
pub fn run(ctx: Context) -> Context {
  // @TODO: move values to config file
  let ui = Ui {
    ctx: &ctx,
    // @TODO: show_valid: false,
    // @TODO: sort_by: "name" | "state" | "count",
  };
  let dependency_name_regex = ctx.config.cli.options.dependency_name_regex.as_ref();
  let matches_filter = |name: &str| -> bool { dependency_name_regex.map_or(true, |regex| regex.is_match(name)) };

  if ctx.config.cli.options.inspect_mismatches {
    ui.print_command_header("SEMVER RANGES AND VERSION MISMATCHES");
    ctx.version_groups.iter().for_each(|group| {
      ui.print_group_header(group);
      if group.dependencies.borrow().len() == 0 {
        let label = &group.selector.label;
        ui.print_empty_group();
        return;
      }
      if !ctx.config.cli.options.show_ignored && matches!(group.variant, VersionGroupVariant::Ignored) {
        ui.print_ignored_group();
        return;
      }
      group.dependencies.borrow().values().for_each(|dependency| {
        if matches_filter(&dependency.name) {
          ui.print_dependency(dependency, &group.variant);
          ui.for_each_instance(dependency, |instance| {
            if ctx.config.cli.options.show_instances {
              ui.print_instance(instance, &group.variant);
            }
          });
        }
      });
    });
  }
  if ctx.config.cli.options.inspect_formatting {
    ui.print_command_header("PACKAGE FORMATTING");
    let formatted_packages = ctx
      .packages
      .all
      .iter()
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
