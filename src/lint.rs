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
    expect::{expect, ExpectedMatchEvent, ExpectedMismatchEvent},
  };
  use serde_json::json;

  #[test]
  fn run_effect_when_packages_loaded() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::new();

    lint(&config, &mut packages, &mut effects);
    assert_eq!(effects.events.packages_loaded.len(), 1);
  }

  #[test]
  fn highest_version_mismatch_in_same_file() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "1.0.0"
      },
      "devDependencies": {
        "wat": "2.0.0"
      }
    })]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_highest_version_mismatches(vec![ExpectedMismatchEvent {
      dependency_name: "wat",
      instance_id: "wat in /dependencies of package-a",
      actual_specifier: "1.0.0",
      expected_specifier: "2.0.0",
    }])
  }

  #[test]
  fn many_highest_version_mismatches_in_same_file() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "0.1.0"
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
      ExpectedMismatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-a",
        actual_specifier: "0.1.0",
        expected_specifier: "0.3.0",
      },
      ExpectedMismatchEvent {
        dependency_name: "wat",
        instance_id: "wat in /peerDependencies of package-a",
        actual_specifier: "0.2.0",
        expected_specifier: "0.3.0",
      },
    ])
  }

  #[test]
  fn highest_version_mismatch_in_multiple_files() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "dependencies": {
          "wat": "1.0.0"
        }
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "wat": "2.0.0"
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_highest_version_mismatches(vec![ExpectedMismatchEvent {
      dependency_name: "wat",
      instance_id: "wat in /dependencies of package-a",
      actual_specifier: "1.0.0",
      expected_specifier: "2.0.0",
    }]);
  }

  #[test]
  fn highest_version_mismatch_in_different_version_groups() {
    let mut effects = MockEffects::new();
    let config = Config::from_mock(json!({
      "versionGroups": [
        { "packages": ["package-a"] },
        { "packages": ["package-b"] }
      ]
    }));
    let mut packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "dependencies": {
          "good": "1.0.0"
        }
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "good": "2.0.0"
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_highest_version_mismatches(vec![]);

    expect(&effects).to_have_standard_version_group_matches(vec![
      ExpectedMatchEvent {
        dependency_name: "good",
        instance_id: "good in /dependencies of package-a",
        specifier: "1.0.0",
      },
      ExpectedMatchEvent {
        dependency_name: "good",
        instance_id: "good in /dependencies of package-b",
        specifier: "2.0.0",
      },
    ]);
  }

  #[test]
  fn local_version_highest_version_mismatch() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0"
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "package-a": "1.1.0"
        },
        "devDependencies": {
          "package-a": "workspace:*"
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_local_version_mismatches(vec![
      ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual_specifier: "1.1.0",
        expected_specifier: "1.0.0",
      },
      ExpectedMismatchEvent {
        dependency_name: "package-a",
        instance_id: "package-a in /devDependencies of package-b",
        actual_specifier: "workspace:*",
        expected_specifier: "1.0.0",
      },
    ]);
  }

  #[test]
  fn refuse_to_pin_local_version() {
    let mut effects = MockEffects::new();
    let config = Config::from_mock(json!({
      "versionGroups": [{
        "dependencies": ["package-a"],
        "pinVersion": "1.2.0"
      }]
    }));
    let mut packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "version": "1.0.0"
      }),
      json!({
        "name": "package-b",
        "dependencies": {
          "package-a": "1.1.0"
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_mismatches_changing_local_versions(vec![ExpectedMismatchEvent {
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual_specifier: "1.0.0",
      expected_specifier: "1.2.0",
    }]);

    expect(&effects).to_have_pinned_version_mismatches(vec![ExpectedMismatchEvent {
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual_specifier: "1.1.0",
      expected_specifier: "1.2.0",
    }]);
  }

  #[test]
  fn highest_version_matches_and_mismatches() {
    let mut effects = MockEffects::new();
    let config = Config::new();
    let mut packages = Packages::from_mocks(vec![
      json!({
        "name": "package-a",
        "dependencies": {
          "mix": "0.3.0"
        },
        "devDependencies": {
          "mix": "0.1.0"
        },
        "peerDependencies": {
          "mix": "0.2.0"
        }
      }),
      json!({
        "name": "package-b",
        "devDependencies": {
          "mix": "0.3.0"
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect(&effects).to_have_standard_version_group_matches(vec![
      ExpectedMatchEvent {
        dependency_name: "mix",
        instance_id: "mix in /dependencies of package-a",
        specifier: "0.3.0",
      },
      ExpectedMatchEvent {
        dependency_name: "mix",
        instance_id: "mix in /devDependencies of package-b",
        specifier: "0.3.0",
      },
    ]);

    expect(&effects).to_have_highest_version_mismatches(vec![
      ExpectedMismatchEvent {
        dependency_name: "mix",
        instance_id: "mix in /devDependencies of package-a",
        actual_specifier: "0.1.0",
        expected_specifier: "0.3.0",
      },
      ExpectedMismatchEvent {
        dependency_name: "mix",
        instance_id: "mix in /peerDependencies of package-a",
        actual_specifier: "0.2.0",
        expected_specifier: "0.3.0",
      },
    ]);
  }
}
