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
  use super::*;
  use crate::{
    effects_mock::MockEffects,
    expect::{expect, ExpectedMismatch},
  };
  use serde_json::json;

  #[test]
  fn run_effect_when_packages_loaded() {
    let config = Config::new();
    let mut packages = Packages::new();
    let mut effects = MockEffects::new();

    lint(&config, &mut packages, &mut effects);
    assert_eq!(effects.events.packages_loaded.len(), 1);
  }

  #[test]
  fn highest_version_mismatch_in_same_file() {
    let config = Config::new();
    let mut effects = MockEffects::new();
    let mut packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "1.0.0",
      },
      "devDependencies": {
        "wat": "2.0.0"
      }
    })]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_highest_version_mismatches(vec![ExpectedMismatch {
      dependency_name: "wat",
      mismatch_id: "wat in /dependencies of package-a",
      mismatching_version: "1.0.0",
      expected_version: "2.0.0",
    }])
  }

  #[test]
  fn many_highest_version_mismatches_in_same_file() {
    let config = Config::new();
    let mut effects = MockEffects::new();
    let mut packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "0.1.0",
      },
      "devDependencies": {
        "wat": "0.3.0"
      },
      "peerDependencies": {
        "wat": "0.2.0"
      }
    })]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_highest_version_mismatches(vec![
      ExpectedMismatch {
        dependency_name: "wat",
        mismatch_id: "wat in /dependencies of package-a",
        mismatching_version: "0.1.0",
        expected_version: "0.3.0",
      },
      ExpectedMismatch {
        dependency_name: "wat",
        mismatch_id: "wat in /peerDependencies of package-a",
        mismatching_version: "0.2.0",
        expected_version: "0.3.0",
      },
    ])
  }

  #[test]
  fn highest_version_mismatch_in_multiple_files() {
    let config = Config::new();
    let mut effects = MockEffects::new();
    let mut packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "dependencies": {
          "wat": "1.0.0",
        }
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "wat": "2.0.0",
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_highest_version_mismatches(vec![ExpectedMismatch {
      dependency_name: "wat",
      mismatch_id: "wat in /dependencies of package-a",
      mismatching_version: "1.0.0",
      expected_version: "2.0.0",
    }]);
  }

  #[test]
  fn local_version_mismatch_in_multiple_files() {
    let config = Config::new();
    let mut effects = MockEffects::new();
    let mut packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0",
      }),
      json!({
        "name": "package-b",
        "devDependencies": {
          "package-a": "1.1.0",
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_local_version_mismatches(vec![ExpectedMismatch {
      dependency_name: "package-a",
      mismatch_id: "package-a in /devDependencies of package-b",
      mismatching_version: "1.1.0",
      expected_version: "1.0.0",
    }]);
  }
}
