use colored::*;
use log::info;

use crate::{context::Context, effects::ui, version_group::VersionGroupVariant};

/// Run the lint command side effects
pub fn run(ctx: Context) -> Context {
  if ctx.config.cli.options.versions {
    info!("{}", "= SEMVER RANGES AND VERSION MISMATCHES".dimmed());
    ctx.version_groups.iter().for_each(|group| {
      ui::group_header(group);
      group.dependencies.borrow().values().for_each(|dependency| {
        ui::dependency_header(dependency);
        match dependency.variant {
          VersionGroupVariant::Banned => {
            dependency.instances.borrow().iter().for_each(|instance| {
              println!("      {}", ui::status_code_link(&instance.state.borrow()));
            });
          }
          VersionGroupVariant::HighestSemver | VersionGroupVariant::LowestSemver => {
            dependency.instances.borrow().iter().for_each(|instance| {
              println!("      {}", ui::status_code_link(&instance.state.borrow()));
            });
          }
          VersionGroupVariant::Ignored => {
            dependency.instances.borrow().iter().for_each(|instance| {
              println!("      {}", ui::status_code_link(&instance.state.borrow()));
            });
          }
          VersionGroupVariant::Pinned => {
            dependency.instances.borrow().iter().for_each(|instance| {
              println!("      {}", ui::status_code_link(&instance.state.borrow()));
            });
          }
          VersionGroupVariant::SameRange => {
            dependency.instances.borrow().iter().for_each(|instance| {
              println!("      {}", ui::status_code_link(&instance.state.borrow()));
            });
          }
          VersionGroupVariant::SnappedTo => {
            dependency.instances.borrow().iter().for_each(|instance| {
              println!("      {}", ui::status_code_link(&instance.state.borrow()));
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
