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
  use crate::effects_mock::MockEffects;
  use serde_json::json;

  struct ExpectedMismatch<'a> {
    dependency_name: &'a str,
    mismatch_id: &'a str,
    mismatching_version: &'a str,
    expected_version: &'a str,
  }

  fn expect_instance_mismatches_highest_version(
    effects: &MockEffects,
    expected_mismatches: Vec<ExpectedMismatch>,
  ) {
    let actual_mismatches = &effects.events.instance_mismatches_highest_version;

    if expected_mismatches.len() != actual_mismatches.len() {
      panic!(
        "expected {} highest semver mismatches but found {}",
        expected_mismatches.len(),
        actual_mismatches.len()
      );
    }

    for expected in expected_mismatches {
      let dependency_name = expected.dependency_name.to_string();
      let mismatch_id = expected.mismatch_id.to_string();
      let mismatch_version = expected.mismatching_version.to_string();
      let expected_version = expected.expected_version.to_string();

      let event = effects
        .events
        .instance_mismatches_highest_version
        .iter()
        .find(|event| event.dependency_name == dependency_name);

      match event {
        None => {
          panic!(
            "expected highest semver mismatch for '{}' but none was found",
            dependency_name
          );
        }
        Some(event) => {
          if event.mismatches_with.0 != expected_version {
            panic!(
              "expected highest semver mismatch for '{}' to suggest '{}' but found '{}'",
              dependency_name, expected_version, event.mismatches_with.0
            );
          }
          if event.target.0 != mismatch_version {
            panic!(
              "expected highest semver mismatch for '{}' to find mismatching version '{}' but found '{}'",
              dependency_name, mismatch_version, event.target.0);
          }
          if !event.target.1.contains(&mismatch_id) {
            panic!(
              "expected highest semver mismatch for '{}' for instance '{}' but none was found",
              dependency_name, mismatch_id
            );
          }
        }
      };
    }
  }

  fn expect_instance_mismatches_local_version(
    effects: &MockEffects,
    expected_mismatches: Vec<ExpectedMismatch>,
  ) {
    let actual_mismatches = &effects.events.instance_mismatches_local_version;

    if expected_mismatches.len() != actual_mismatches.len() {
      panic!(
        "expected {} highest semver mismatches but found {}",
        expected_mismatches.len(),
        actual_mismatches.len()
      );
    }

    for expected in expected_mismatches {
      let dependency_name = expected.dependency_name.to_string();
      let mismatch_id = expected.mismatch_id.to_string();
      let mismatch_version = expected.mismatching_version.to_string();
      let expected_version = expected.expected_version.to_string();

      let event = actual_mismatches
        .iter()
        .find(|event| event.dependency_name == dependency_name);

      match event {
        None => {
          panic!(
            "expected local version mismatch for '{}' but none was found\n{:#?}",
            dependency_name, event
          );
        }
        Some(event) => {
          if event.mismatches_with.0 != expected_version {
            panic!(
              "expected local version mismatch for '{}' to suggest '{}' but found '{}'\n{:#?}",
              dependency_name, expected_version, event.mismatches_with.0, event
            );
          }
          if event.target.0 != mismatch_version {
            panic!(
              "expected local version mismatch for '{}' to find mismatching version '{}' but found '{}'\n{:#?}",
              dependency_name,
              mismatch_version,
              event.target.0,
              event
            );
          }
          if !event.target.1.contains(&mismatch_id) {
            panic!(
              "expected local version mismatch for '{}' for instance '{}' but none was found\n{:#?}",
              dependency_name, mismatch_id, event
            );
          }
        }
      };
    }
  }

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
    let mut packages = Packages::new();
    let mut effects = MockEffects::new();

    packages.add_mock_packages(vec![json!({
      "name": "package-a",
      "dependencies": {
        "wat": "1.0.0",
      },
      "devDependencies": {
        "wat": "2.0.0"
      }
    })]);

    lint(&config, &mut packages, &mut effects);

    expect_instance_mismatches_highest_version(
      &effects,
      vec![ExpectedMismatch {
        dependency_name: "wat",
        mismatch_id: "wat in /dependencies of package-a",
        mismatching_version: "1.0.0",
        expected_version: "2.0.0",
      }],
    );
  }

  #[test]
  fn highest_version_mismatch_in_multiple_files() {
    let config = Config::new();
    let mut packages = Packages::new();
    let mut effects = MockEffects::new();

    packages.add_mock_packages(vec![
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

    expect_instance_mismatches_highest_version(
      &effects,
      vec![ExpectedMismatch {
        dependency_name: "wat",
        mismatch_id: "wat in /dependencies of package-a",
        mismatching_version: "1.0.0",
        expected_version: "2.0.0",
      }],
    );
  }

  #[test]
  fn local_version_mismatch_in_multiple_files() {
    let config = Config::new();
    let mut packages = Packages::new();
    let mut effects = MockEffects::new();

    packages.add_mock_packages(vec![
      json!({
        "name": "package-a",
        "version": "2.0.0",
      }),
      json!({
        "name": "package-b",
        "devDependencies": {
          "package-a": "1.0.0",
        }
      }),
    ]);

    lint(&config, &mut packages, &mut effects);

    expect_instance_mismatches_local_version(
      &effects,
      vec![ExpectedMismatch {
        dependency_name: "package-a",
        mismatch_id: "package-a in /devDependencies of package-b",
        mismatching_version: "1.0.0",
        expected_version: "2.0.0",
      }],
    );
  }
}
