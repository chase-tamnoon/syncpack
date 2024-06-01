use crate::{
  config::Config,
  context::{self, Context},
  effects::{Effects, Event},
  format::{self, InMemoryFormattingStatus},
  packages::Packages,
};

pub fn lint(config: &Config, packages: &mut Packages, effects: &mut impl Effects) {
  effects.on_event(Event::PackagesLoaded(&config, &packages));

  let cli = &config.cli;
  let Context {
    mut instances_by_id,
    version_groups,
  } = context::get(&config, &packages);

  effects.on_event(Event::EnterVersionsAndRanges(&config));

  if cli.options.ranges || cli.options.versions {
    version_groups.iter().for_each(|group| {
      group.visit(&mut instances_by_id, packages, effects);
    });
  }

  effects.on_event(Event::EnterFormat(&config));

  if cli.options.format {
    let InMemoryFormattingStatus {
      was_valid,
      was_invalid,
    } = format::fix(&config, packages);
    if !was_valid.is_empty() {
      effects.on_event(Event::PackagesMatchFormatting(&was_valid, &config));
    }
    if !was_invalid.is_empty() {
      effects.on_event(Event::PackagesMismatchFormatting(&was_invalid, &config));
    }
  }

  effects.on_event(Event::ExitCommand);
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TestEffects {
    pub is_valid: bool,
  }

  impl TestEffects {
    pub fn new() -> Self {
      TestEffects { is_valid: false }
    }
  }

  impl Effects for TestEffects {
    fn on_event(&mut self, event: Event) -> () {
      if let Event::PackagesLoaded(config, packages) = event {
        self.is_valid = true;
      }
    }
  }

  #[test]
  fn run_effect_when_packages_loaded() {
    let config = Config::new();
    let mut packages = Packages::new();
    let mut effects = TestEffects::new();

    lint(&config, &mut packages, &mut effects);
    assert_eq!(effects.is_valid, true);
  }
}
