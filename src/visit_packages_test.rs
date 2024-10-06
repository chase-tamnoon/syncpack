use crate::{
  effects::InstanceState,
  test::{
    self,
    expect::{expect, ExpectedFixableMismatchEvent, ExpectedMatchEvent, ExpectedOverrideEvent, ExpectedUnfixableMismatchEvent},
  },
};
use serde_json::json;

use super::*;

#[cfg(test)]
#[ctor::ctor]
fn init() {
  use crate::logger;
  logger::init();
}

// = Standard Version Group: Local =============================================

#[test]
fn instance_depends_on_local_version_which_is_missing() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a"
    }),
    json!({
      "name": "package-b",
      "devDependencies": {
        "package-a": "0.1.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::MismatchesInvalidLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /devDependencies of package-b",
      actual: "0.1.0",
    }]);
}

#[test]
fn instance_depends_on_local_version_which_is_not_exact_semver() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "^1.0.0"
    }),
    json!({
      "name": "package-b",
      "devDependencies": {
        "package-a": "0.1.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "^1.0.0",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::MismatchesInvalidLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /devDependencies of package-b",
      actual: "0.1.0",
    }]);
}

#[test]
fn instance_has_higher_version_than_local_package_and_has_no_semver_group() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: "1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_identical_to_local_package_and_has_no_semver_group() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "1.0.0"
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "package-a": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::ValidLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "1.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_different_version_to_local_package_and_has_no_semver_group() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: "1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_a_different_range_and_has_no_semver_group() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "1.0.0"
    }),
    json!({
      "name": "package-b",
      "devDependencies": {
        "package-a": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /devDependencies of package-b",
      actual: "~1.0.0",
      expected: "1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_matches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": "^"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "1.0.0"
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "package-a": "^1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::ValidLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "^1.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_mismatches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": "^"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "1.0.0"
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "package-a": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::SemverRangeMismatch,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "~1.0.0",
      expected: "^1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_matches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": "<"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "1.0.0"
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "package-a": "<1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::SemverRangeMatchConflictsWithLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "<1.0.0",
    }]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_mismatches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": ">"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "1.0.0"
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "package-a": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::SemverRangeMismatchConflictsWithLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "~1.0.0",
    }]);
}

// = Standard Version Group: Highest/Lowest ====================================

#[test]
fn reports_one_highest_version_mismatch_in_one_file() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "dependencies": {
      "wat": "1.0.0"
    },
    "devDependencies": {
      "wat": "2.0.0"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "wat",
      instance_id: "wat in /devDependencies of package-a",
      actual: "2.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPreferVersion,
      dependency_name: "wat",
      instance_id: "wat in /dependencies of package-a",
      actual: "1.0.0",
      expected: "2.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn reports_many_highest_version_mismatches_in_one_file() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "wat",
      instance_id: "wat in /devDependencies of package-a",
      actual: "0.3.0",
    }])
    .to_have_fixable_mismatches(vec![
      ExpectedFixableMismatchEvent {
        variant: InstanceState::MismatchesPreferVersion,
        dependency_name: "wat",
        instance_id: "wat in /dependencies of package-a",
        actual: "0.1.0",
        expected: "0.3.0",
      },
      ExpectedFixableMismatchEvent {
        variant: InstanceState::MismatchesPreferVersion,
        dependency_name: "wat",
        instance_id: "wat in /peerDependencies of package-a",
        actual: "0.2.0",
        expected: "0.3.0",
      },
    ])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn reports_highest_version_mismatches_in_many_files() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "wat",
      instance_id: "wat in /dependencies of package-b",
      actual: "2.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPreferVersion,
      dependency_name: "wat",
      instance_id: "wat in /dependencies of package-a",
      actual: "1.0.0",
      expected: "2.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn does_not_report_highest_version_mismatches_when_in_different_version_groups() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [
      { "packages": ["package-a"] },
      { "packages": ["package-b"] }
    ]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPreferVersion,
        dependency_name: "good",
        instance_id: "good in /dependencies of package-a",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPreferVersion,
        dependency_name: "good",
        instance_id: "good in /dependencies of package-b",
        actual: "2.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn does_not_confuse_highest_version_matches_and_mismatches_of_the_same_dependency() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "0.0.0",
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
      "version": "0.0.0",
      "devDependencies": {
        "mix": "0.3.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::ValidLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "0.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::ValidLocal,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "0.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPreferVersion,
        dependency_name: "mix",
        instance_id: "mix in /dependencies of package-a",
        actual: "0.3.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPreferVersion,
        dependency_name: "mix",
        instance_id: "mix in /devDependencies of package-b",
        actual: "0.3.0",
      },
    ])
    .to_have_fixable_mismatches(vec![
      ExpectedFixableMismatchEvent {
        variant: InstanceState::MismatchesPreferVersion,
        dependency_name: "mix",
        instance_id: "mix in /devDependencies of package-a",
        actual: "0.1.0",
        expected: "0.3.0",
      },
      ExpectedFixableMismatchEvent {
        variant: InstanceState::MismatchesPreferVersion,
        dependency_name: "mix",
        instance_id: "mix in /peerDependencies of package-a",
        actual: "0.2.0",
        expected: "0.3.0",
      },
    ])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_identical_to_highest_semver_and_has_no_semver_group() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-b",
        actual: "1.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_different_version_to_highest_semver_and_has_no_semver_group() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "1.1.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-b",
      actual: "1.1.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-a",
      actual: "1.0.0",
      expected: "1.1.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_a_different_range_and_has_no_semver_group() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "^1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "devDependencies": {
        "foo": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-a",
      actual: "^1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /devDependencies of package-b",
      actual: "~1.0.0",
      expected: "^1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_matches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "~"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "^1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MatchesPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-b",
        actual: "~1.0.0",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-a",
      actual: "^1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_mismatches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "^"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": ">=1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-a",
      actual: ">=1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::SemverRangeMismatch,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-b",
      actual: "~1.0.0",
      expected: "^1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_matches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "<"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "<1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::SemverRangeMatchConflictsWithPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-b",
      actual: "<1.0.0",
    }]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_mismatches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "<"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "~1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-a",
      actual: "~1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::SemverRangeMismatchConflictsWithPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-b",
      actual: "1.0.0",
    }]);
}

// = Standard Version Group: Non Semver ========================================

#[test]
fn no_instances_are_semver_but_all_are_identical() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "workspace:*"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "workspace:*"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::EqualsNonSemverPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: "workspace:*",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsNonSemverPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-b",
        actual: "workspace:*",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn no_instances_are_semver_and_they_differ() {
  let config = test::mock::config();
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": "workspace:*"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "workspace:^"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MismatchesNonSemverPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: "workspace:*",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MismatchesNonSemverPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-b",
        actual: "workspace:^",
      },
    ]);
}

// = Ignored Version Group =====================================================

#[test]
fn all_instances_are_ignored() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "isIgnored": true,
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "1.1.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

// = Pinned Version Group: Local ===============================================

#[test]
fn refuses_to_pin_local_version() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["package-a"],
      "pinVersion": "1.2.0"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::RefuseToPinLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPin,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: "1.2.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

// = Pinned Version Group: Normal ==============================================

#[test]
fn a_pinned_version_will_replace_anything_different() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "pinVersion": "1.2.0"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "workspace:*"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPin,
      dependency_name: "foo",
      instance_id: "foo in /devDependencies of package-a",
      actual: "workspace:*",
      expected: "1.2.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn pin_version_will_override_instance_with_same_version_number_as_pinned_but_matching_a_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "dependencies": ["foo"],
      "range": "^"
    }],
    "versionGroups": [{
      "dependencies": ["foo"],
      "pinVersion": "1.0.0"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "^1.0.0"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![ExpectedOverrideEvent {
      variant: InstanceState::PinMatchOverridesSemverRangeMatch,
      dependency_name: "foo",
      instance_id: "foo in /devDependencies of package-a",
      actual: "^1.0.0",
      expected: "1.0.0",
      overridden: "^1.0.0",
    }])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn pin_version_will_override_instance_with_same_version_number_as_pinned_but_mismatching_a_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "dependencies": ["foo"],
      "range": "^"
    }],
    "versionGroups": [{
      "dependencies": ["foo"],
      "pinVersion": "1.0.0"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": ">=1.0.0"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![ExpectedOverrideEvent {
      variant: InstanceState::PinMatchOverridesSemverRangeMismatch,
      dependency_name: "foo",
      instance_id: "foo in /devDependencies of package-a",
      actual: ">=1.0.0",
      expected: "1.0.0",
      overridden: "^1.0.0",
    }])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn pin_version_will_override_instance_with_same_version_number_as_pinned_but_a_different_range_and_no_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "pinVersion": "1.0.0"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "^1.0.0"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::ValidLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPin,
      dependency_name: "foo",
      instance_id: "foo in /devDependencies of package-a",
      actual: "^1.0.0",
      expected: "1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn an_already_pinned_version_is_valid() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "pinVersion": "1.2.0"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "1.2.0"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::ValidLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPin,
        dependency_name: "foo",
        instance_id: "foo in /devDependencies of package-a",
        actual: "1.2.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

// = Banned Version Group ======================================================

#[test]
fn refuses_to_ban_local_version() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["package-a"],
      "isBanned": true
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
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

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::RefuseToBanLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::Banned,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: "VERSION_IS_MISSING",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

// = Same Range Version Group ==================================================

#[test]
fn instance_in_a_same_range_group_satisfies_every_other_and_there_are_no_semver_groups() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencyTypes": ["local"],
      "isIgnored": true
    }, {
      "dependencies": ["foo"],
      "policy": "sameRange"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": ">=1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "<=2.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesSameRangeGroup,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: ">=1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesSameRangeGroup,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-b",
        actual: "<=2.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_in_a_same_range_group_satisfies_every_other_and_matches_its_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "^"
    }],
    "versionGroups": [{
      "dependencyTypes": ["local"],
      "isIgnored": true
    }, {
      "dependencies": ["foo"],
      "policy": "sameRange"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": ">=1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "^1.2.3"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesSameRangeGroup,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: ">=1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesSameRangeGroup,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-b",
        actual: "^1.2.3",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_in_a_same_range_group_satisfies_every_other_but_mismatches_its_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "~"
    }],
    "versionGroups": [{
      "dependencyTypes": ["local"],
      "isIgnored": true
    }, {
      "dependencies": ["foo"],
      "policy": "sameRange"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": ">=1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "^1.2.3"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesSameRangeGroup,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: ">=1.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::SemverRangeMismatch,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-b",
      actual: "^1.2.3",
      expected: "~1.2.3",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_in_a_same_range_group_does_not_satisfy_another() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencyTypes": ["local"],
      "isIgnored": true
    }, {
      "dependencies": ["foo"],
      "policy": "sameRange"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "dependencies": {
        "foo": ">=1.0.0"
      }
    }),
    json!({
      "name": "package-b",
      "dependencies": {
        "foo": "<1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedMatchEvent {
        variant: InstanceState::Ignored,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MismatchesSameRangeGroup,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: ">=1.0.0",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MismatchesSameRangeGroup,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-b",
        actual: "<1.0.0",
      },
    ]);
}

// = Snapped To Version Group ==================================================

#[test]
fn instance_identical_to_snapped_to_and_has_no_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::EqualsPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of leader",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsSnapToVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of follower",
        actual: "1.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_different_version_to_snapped_to_and_has_no_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "dependencies": {
        "foo": "1.1.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of leader",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesSnapToVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of follower",
      actual: "1.1.0",
      expected: "1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_snapped_to_but_a_different_range_and_has_no_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": "^1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "devDependencies": {
        "foo": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of leader",
      actual: "^1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesSnapToVersion,
      dependency_name: "foo",
      instance_id: "foo in /devDependencies of follower",
      actual: "~1.0.0",
      expected: "^1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_snapped_to_but_matches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["follower"],
      "range": "~"
    }],
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": "^1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "dependencies": {
        "foo": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MatchesSnapToVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of follower",
        actual: "~1.0.0",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of leader",
      actual: "^1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_snapped_to_but_mismatches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["follower"],
      "range": "^"
    }],
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": ">=1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "dependencies": {
        "foo": "~1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of leader",
      actual: ">=1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::SemverRangeMismatch,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of follower",
      actual: "~1.0.0",
      expected: "^1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn instance_has_same_version_number_as_snapped_to_but_matches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["follower"],
      "range": "<"
    }],
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "dependencies": {
        "foo": "<1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of leader",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::SemverRangeMatchConflictsWithSnapToVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of follower",
      actual: "<1.0.0",
    }]);
}

#[test]
fn instance_has_same_version_number_as_snapped_to_but_mismatches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["follower"],
      "range": "<"
    }],
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": "~1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of leader",
      actual: "~1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::SemverRangeMismatchConflictsWithSnapToVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of follower",
      actual: "1.0.0",
    }]);
}

#[test]
fn instance_cannot_find_a_snapped_to_version() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "packages": ["follower"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "version": "1.0.0"
    }),
    json!({
      "name": "follower",
      "version": "0.1.0",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::ValidLocal,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::ValidLocal,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "0.1.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::SnapToVersionNotFound,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of follower",
      actual: "1.0.0",
    }]);
}

#[test]
fn instance_is_in_a_snapped_to_group_and_is_itself_a_snapped_to_target() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "snapTo": ["leader"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "leader",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
    json!({
      "name": "follower",
      "dependencies": {
        "foo": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "leader",
        instance_id: "leader in /version of leader",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::InvalidLocalVersion,
        dependency_name: "follower",
        instance_id: "follower in /version of follower",
        actual: "VERSION_IS_MISSING",
      },
    ])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::EqualsSnapToVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of leader",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsSnapToVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of follower",
        actual: "1.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}

#[test]
fn refuses_to_snap_local_version_to_another_target() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "snapTo": ["package-b"]
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![
    json!({
      "name": "package-a",
      "version": "1.1.0"
    }),
    json!({
      "name": "package-b",
      "version": "0.1.0",
      "dependencies": {
        "package-a": "0.0.1"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_overrides(vec![])
    .to_have_warnings(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::RefuseToSnapLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.1.0",
    }])
    .to_have_warnings_of_instance_changes(vec![])
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::EqualsSnapToVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "0.1.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::EqualsSnapToVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "0.0.1",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![]);
}
