use crate::test::{
  self,
  expect::{expect, ExpectedFixableMismatchEvent, ExpectedMatchEvent, ExpectedUnfixableMismatchEvent},
};
use serde_json::json;

use super::*;

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
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::MatchesPreferVersion,
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
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::MissingLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
    }]);
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
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::MatchesPreferVersion,
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
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::MissingLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
    }]);
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
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::MatchesPreferVersion,
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
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ]);
}

#[test]
fn does_not_consider_instances_in_different_version_groups_a_highest_version_mismatch() {
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
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::MatchesPreferVersion,
        dependency_name: "good",
        instance_id: "good in /dependencies of package-a",
        actual: "1.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesPreferVersion,
        dependency_name: "good",
        instance_id: "good in /dependencies of package-b",
        actual: "2.0.0",
      },
    ])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ]);
}

#[test]
fn rejects_pinned_version_when_it_would_replace_local_version() {
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
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesPin,
      dependency_name: "package-a",
      instance_id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: "1.2.0",
    }])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::RefuseToPinLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ]);
}

#[test]
fn does_not_confuse_highest_version_matches_and_mismatches() {
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
    .to_have_matches(vec![
      ExpectedMatchEvent {
        variant: InstanceState::MatchesPreferVersion,
        dependency_name: "mix",
        instance_id: "mix in /dependencies of package-a",
        actual: "0.3.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::MatchesPreferVersion,
        dependency_name: "mix",
        instance_id: "mix in /devDependencies of package-b",
        actual: "0.3.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::LocalWithValidVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "0.0.0",
      },
      ExpectedMatchEvent {
        variant: InstanceState::LocalWithValidVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "0.0.0",
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
fn reports_local_version_mismatch_when_an_instance_uses_a_higher_version() {
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
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::LocalWithValidVersion,
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
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::MissingLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }]);
}

#[test]
fn instance_has_same_version_as_local_package_but_does_not_match_its_semver_group() {
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
        "package-a": "1.0.0"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![
      // refuse to break local package's version
      ExpectedFixableMismatchEvent {
        variant: InstanceState::RefuseToChangeLocalSemverRange,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
        expected: "1.0.0",
      },
    ])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::LocalMatchConflictsWithSemverGroup,
        dependency_name: "package-a",
        instance_id: "package-a in /dependencies of package-b",
        actual: "1.0.0",
      },
    ]);
}

#[test]
fn instance_is_highest_or_lowest_semver_once_semver_group_is_fixed() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "dependencyTypes": ["dev"],
      "range": ">"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "dependencies": {
      "foo": "1.0.0"
    },
    "devDependencies": {
      "foo": "1.0.0"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![
      ExpectedFixableMismatchEvent {
        variant: InstanceState::MismatchesPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /dependencies of package-a",
        actual: "1.0.0",
        expected: ">1.0.0",
      },
      ExpectedFixableMismatchEvent {
        variant: InstanceState::SemverRangeMismatchWillFixPreferVersion,
        dependency_name: "foo",
        instance_id: "foo in /devDependencies of package-a",
        actual: "1.0.0",
        expected: ">1.0.0",
      },
    ])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::MissingLocalVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
    }]);
}

#[test]
fn instance_is_no_longer_highest_or_lowest_semver_once_semver_group_is_fixed() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "dependencyTypes": ["dev"],
      "range": "<"
    }]
  }));
  let mut effects = test::mock::effects(&config);
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "dependencies": {
      "foo": "1.0.0"
    },
    "devDependencies": {
      "foo": "1.0.0"
    }
  })]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::MatchesPreferVersion,
      dependency_name: "foo",
      instance_id: "foo in /dependencies of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::PreferVersionMatchConflictsWithSemverGroup,
        dependency_name: "foo",
        instance_id: "foo in /devDependencies of package-a",
        actual: "1.0.0",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
    ]);
}

#[test]
fn reports_local_version_mismatch_when_an_instance_uses_workspace_protocol() {
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
        "package-a": "workspace:*"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::LocalWithValidVersion,
      dependency_name: "package-a",
      instance_id: "package-a in /version of package-a",
      actual: "1.0.0",
    }])
    .to_have_fixable_mismatches(vec![ExpectedFixableMismatchEvent {
      variant: InstanceState::MismatchesLocal,
      dependency_name: "package-a",
      instance_id: "package-a in /devDependencies of package-b",
      actual: "workspace:*",
      expected: "1.0.0",
    }])
    .to_have_unfixable_mismatches(vec![ExpectedUnfixableMismatchEvent {
      variant: InstanceState::MissingLocalVersion,
      dependency_name: "package-b",
      instance_id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
    }]);
}

#[test]
fn protects_local_version_when_naively_pinned_to_use_workspace_protocol() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencyTypes": ["**"],
      "dependencies": ["**"],
      "packages": ["**"],
      "pinVersion": "workspace:*",
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
      "devDependencies": {
        "package-a": "workspace:*"
      }
    }),
  ]);

  visit_packages(&config, &packages, &mut effects);

  expect(&effects)
    .to_have_matches(vec![ExpectedMatchEvent {
      variant: InstanceState::MatchesPin,
      dependency_name: "package-a",
      instance_id: "package-a in /devDependencies of package-b",
      actual: "workspace:*",
    }])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::RefuseToPinLocal,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "1.0.0",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::RefuseToPinLocal,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ]);
}

#[test]
fn reports_unfixable_local_version_mismatch_when_local_version_is_missing() {
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
    .to_have_matches(vec![])
    .to_have_fixable_mismatches(vec![])
    .to_have_unfixable_mismatches(vec![
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MismatchesMissingLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /devDependencies of package-b",
        actual: "0.1.0",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-a",
        instance_id: "package-a in /version of package-a",
        actual: "VERSION_IS_MISSING",
      },
      ExpectedUnfixableMismatchEvent {
        variant: InstanceState::MissingLocalVersion,
        dependency_name: "package-b",
        instance_id: "package-b in /version of package-b",
        actual: "VERSION_IS_MISSING",
      },
    ]);
}

#[test]
#[ignore]
fn reports_unfixable_local_version_mismatch_when_local_version_is_not_exact_semver() {
  panic!("@TODO");
}

#[test]
#[ignore]
fn reports_local_version_mismatch_when_an_instance_has_same_version_but_different_range() {
  panic!("@TODO");
  panic!("@TODO");
}
