use crate::{
  config::Config,
  context::RunState,
  context::{self, Context},
  effects::Effects,
  format::{self, InMemoryFormattingStatus},
  packages::Packages,
};

pub fn lint(
  config: &Config,
  packages: &mut Packages,
  run_effect: fn(Effects) -> (),
  state: &mut RunState,
) {
  run_effect(Effects::PackagesLoaded(&config, &packages, state));

  let cli = &config.cli;
  let Context {
    mut instances_by_id,
    version_groups,
  } = context::get(&config, &packages);

  run_effect(Effects::EnterVersionsAndRanges(&config));

  if cli.options.ranges || cli.options.versions {
    version_groups.iter().for_each(|group| {
      group.visit(&mut instances_by_id, packages, run_effect, state);
    });
  }

  run_effect(Effects::EnterFormat(&config));

  if cli.options.format {
    let InMemoryFormattingStatus {
      was_valid,
      was_invalid,
    } = format::fix(&config, packages);
    if !was_valid.is_empty() {
      run_effect(Effects::PackagesMatchFormatting(&was_valid, &config));
    }
    if !was_invalid.is_empty() {
      run_effect(Effects::PackagesMismatchFormatting(
        &was_invalid,
        &config,
        state,
      ));
    }
  }

  run_effect(Effects::ExitCommand(state));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn run_effect_when_packages_loaded() {
    let config = Config::new();
    let mut packages = Packages::new();
    let mut state = RunState { is_valid: false };

    fn effects(effect: Effects) -> () {
      if let Effects::PackagesLoaded(config, packages, state) = effect {
        state.is_valid = true;
      }
    }

    lint(&config, &mut packages, effects, &mut state);
    assert_eq!(state.is_valid, true);
  }
}
