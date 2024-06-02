use crate::{
  config::Config,
  context::Context,
  effects::{Effects, Event},
  format::{self, InMemoryFormattingStatus},
  packages::Packages,
};

pub fn lint(config: &Config, packages: &mut Packages, effects: &mut impl Effects) {
  effects.on(Event::PackagesLoaded(&config, &packages));

  let cli = &config.cli;
  let Context {
    mut instances_by_id,
    version_groups,
  } = Context::create(&config, &packages);

  effects.on(Event::EnterVersionsAndRanges(&config));

  if cli.options.ranges || cli.options.versions {
    version_groups.iter().for_each(|group| {
      group.visit(&mut instances_by_id, packages, effects);
    });
  }

  effects.on(Event::EnterFormat(&config));

  if cli.options.format {
    let InMemoryFormattingStatus {
      was_valid,
      was_invalid,
    } = format::fix(&config, packages);
    if !was_valid.is_empty() {
      effects.on(Event::PackagesMatchFormatting(&was_valid, &config));
    }
    if !was_invalid.is_empty() {
      effects.on(Event::PackagesMismatchFormatting(&was_invalid, &config));
    }
  }

  effects.on(Event::ExitCommand);
}

#[cfg(test)]
mod tests {
  use crate::effects_mock::MockEffects;

  use super::*;

  #[test]
  fn run_effect_when_packages_loaded() {
    let config = Config::new();
    let mut packages = Packages::new();
    let mut effects = MockEffects::new();

    lint(&config, &mut packages, &mut effects);
    assert_eq!(effects.packages_loaded, 1);
  }
}
