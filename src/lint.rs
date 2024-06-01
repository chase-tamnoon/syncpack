use crate::{
  config::Config,
  context::{self, Context},
  effects::Effects,
  format::{self, InMemoryFormattingStatus},
  packages::Packages,
};

pub fn lint(config: &Config, packages: &mut Packages, run_effect: fn(Effects) -> ()) {
  let cli_options = &config.cli.options;
  let Context {
    mut instances_by_id,
    mut state,
    version_groups,
  } = context::get(&config, &packages);

  run_effect(Effects::EnterVersionsAndRanges(&config));

  if cli_options.ranges || cli_options.versions {
    version_groups.iter().for_each(|group| {
      group.visit(&mut instances_by_id, packages, run_effect, &mut state);
    });
  }

  run_effect(Effects::EnterFormat(&config));

  if cli_options.format {
    let InMemoryFormattingStatus {
      was_valid: valid,
      was_invalid: invalid,
    } = format::fix(&config, packages);
    if !valid.is_empty() {
      run_effect(Effects::PackagesMatchFormatting(&valid, &config));
    }
    if !invalid.is_empty() {
      run_effect(Effects::PackagesMismatchFormatting(
        &invalid, &config, &mut state,
      ));
    }
  }

  run_effect(Effects::ExitCommand(&mut state));
}
