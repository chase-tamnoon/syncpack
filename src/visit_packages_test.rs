use crate::{
  instance::InstanceState,
  test::{
    self,
    expect::{expect, ExpectedInstance},
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesInvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /devDependencies of package-b",
      actual: "0.1.0",
      expected: Some("0.1.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_depends_on_local_version_which_is_not_exact_semver() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "^1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesInvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /devDependencies of package-b",
      actual: "0.1.0",
      expected: Some("0.1.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_higher_version_than_local_package_and_has_no_semver_group() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesLocal,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_identical_to_local_package_and_has_no_semver_group() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsLocal,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_different_version_to_local_package_and_has_no_semver_group() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesLocal,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_a_different_range_and_has_no_semver_group() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesLocal,
      dependency_name: "package-a",
      id: "package-a in /devDependencies of package-b",
      actual: "~1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_matches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": "^"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesLocal,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "^1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_mismatches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": "^"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMismatch,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "~1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_matches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": "<"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMatchConflictsWithLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "<1.0.0",
      expected: Some("<1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_local_package_but_mismatches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "range": ">"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMismatchConflictsWithLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "~1.0.0",
      expected: Some("~1.0.0"),
      overridden: None,
    },
  ]);
}

// = Standard Version Group: Highest/Lowest ====================================

#[test]
fn reports_one_highest_version_mismatch_in_one_file() {
  let config = test::mock::config();
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "dependencies": {
      "wat": "1.0.0"
    },
    "devDependencies": {
      "wat": "2.0.0"
    }
  })]);
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "wat",
      id: "wat in /devDependencies of package-a",
      actual: "2.0.0",
      expected: Some("2.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "wat",
      id: "wat in /dependencies of package-a",
      actual: "1.0.0",
      expected: Some("2.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn reports_many_highest_version_mismatches_in_one_file() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "wat",
      id: "wat in /devDependencies of package-a",
      actual: "0.3.0",
      expected: Some("0.3.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "wat",
      id: "wat in /dependencies of package-a",
      actual: "0.1.0",
      expected: Some("0.3.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "wat",
      id: "wat in /peerDependencies of package-a",
      actual: "0.2.0",
      expected: Some("0.3.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn reports_highest_version_mismatches_in_many_files() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "wat",
      id: "wat in /dependencies of package-b",
      actual: "2.0.0",
      expected: Some("2.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "wat",
      id: "wat in /dependencies of package-a",
      actual: "1.0.0",
      expected: Some("2.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn does_not_report_highest_version_mismatches_when_in_different_version_groups() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [
      { "packages": ["package-a"] },
      { "packages": ["package-b"] }
    ]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "good",
      id: "good in /dependencies of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "good",
      id: "good in /dependencies of package-b",
      actual: "2.0.0",
      expected: Some("2.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn does_not_confuse_highest_version_matches_and_mismatches_of_the_same_dependency() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "0.0.0",
      expected: Some("0.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "0.0.0",
      expected: Some("0.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "mix",
      id: "mix in /dependencies of package-a",
      actual: "0.3.0",
      expected: Some("0.3.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "mix",
      id: "mix in /devDependencies of package-b",
      actual: "0.3.0",
      expected: Some("0.3.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "mix",
      id: "mix in /devDependencies of package-a",
      actual: "0.1.0",
      expected: Some("0.3.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "mix",
      id: "mix in /peerDependencies of package-a",
      actual: "0.2.0",
      expected: Some("0.3.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_identical_to_highest_semver_and_has_no_semver_group() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_different_version_to_highest_semver_and_has_no_semver_group() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "1.1.0",
      expected: Some("1.1.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "1.0.0",
      expected: Some("1.1.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_a_different_range_and_has_no_semver_group() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "^1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPreferVersion,
      dependency_name: "foo",
      id: "foo in /devDependencies of package-b",
      actual: "~1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_matches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "~"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "~1.0.0",
      expected: Some("~1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "^1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_mismatches_a_different_but_compatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "^"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: ">=1.0.0",
      expected: Some(">=1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMismatch,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "~1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_matches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "<"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMatchConflictsWithPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "<1.0.0",
      expected: Some("<1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_has_same_version_number_as_highest_semver_but_mismatches_a_different_but_incompatible_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "semverGroups": [{
      "packages": ["package-b"],
      "range": "<"
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "~1.0.0",
      expected: Some("~1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMismatchConflictsWithPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

// = Standard Version Group: Non Semver ========================================

#[test]
fn no_instances_are_semver_but_all_are_identical() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsNonSemverPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "workspace:*",
      expected: Some("workspace:*"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsNonSemverPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "workspace:*",
      expected: Some("workspace:*"),
      overridden: None,
    },
  ]);
}

#[test]
fn no_instances_are_semver_and_they_differ() {
  let config = test::mock::config();
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesNonSemverPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: "workspace:*",
      expected: Some("workspace:*"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesNonSemverPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "workspace:^",
      expected: Some("workspace:^"),
      overridden: None,
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: Some("1.1.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::RefuseToPinLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPin,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: Some("1.2.0"),
      overridden: None,
    },
  ]);
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
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "workspace:*"
    }
  })]);
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPin,
      dependency_name: "foo",
      id: "foo in /devDependencies of package-a",
      actual: "workspace:*",
      expected: Some("1.2.0"),
      overridden: None,
    },
  ]);
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
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "^1.0.0"
    }
  })]);
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::PinMatchOverridesSemverRangeMatch,
      dependency_name: "foo",
      id: "foo in /devDependencies of package-a",
      actual: "^1.0.0",
      expected: Some("1.0.0"),
      overridden: Some("^1.0.0"),
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
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
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": ">=1.0.0"
    }
  })]);
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::PinMatchOverridesSemverRangeMismatch,
      dependency_name: "foo",
      id: "foo in /devDependencies of package-a",
      actual: ">=1.0.0",
      expected: Some("1.0.0"),
      overridden: Some("^1.0.0"),
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn pin_version_will_override_instance_with_same_version_number_as_pinned_but_a_different_range_and_no_semver_group() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "pinVersion": "1.0.0"
    }]
  }));
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "^1.0.0"
    }
  })]);
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesPin,
      dependency_name: "foo",
      id: "foo in /devDependencies of package-a",
      actual: "^1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn an_already_pinned_version_is_valid() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "pinVersion": "1.2.0"
    }]
  }));
  let packages = test::mock::packages_from_mocks(vec![json!({
    "name": "package-a",
    "version": "1.0.0",
    "devDependencies": {
      "foo": "1.2.0"
    }
  })]);
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPin,
      dependency_name: "foo",
      id: "foo in /devDependencies of package-a",
      actual: "1.2.0",
      expected: Some("1.2.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::RefuseToBanLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::Banned,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "1.1.0",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesSameRangeGroup,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: ">=1.0.0",
      expected: Some(">=1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesSameRangeGroup,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "<=2.0.0",
      expected: Some("<=2.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesSameRangeGroup,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: ">=1.0.0",
      expected: Some(">=1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesSameRangeGroup,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "^1.2.3",
      expected: Some("^1.2.3"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesSameRangeGroup,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: ">=1.0.0",
      expected: Some(">=1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMismatch,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "^1.2.3",
      expected: Some("~1.2.3"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::Ignored,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesSameRangeGroup,
      dependency_name: "foo",
      id: "foo in /dependencies of package-a",
      actual: ">=1.0.0",
      expected: Some(">=1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesSameRangeGroup,
      dependency_name: "foo",
      id: "foo in /dependencies of package-b",
      actual: "<1.0.0",
      expected: Some("<1.0.0"),
      overridden: None,
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsSnapToVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesSnapToVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "1.1.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: "^1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MismatchesSnapToVersion,
      dependency_name: "foo",
      id: "foo in /devDependencies of follower",
      actual: "~1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::MatchesSnapToVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "~1.0.0",
      expected: Some("~1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: "^1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: ">=1.0.0",
      expected: Some(">=1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMismatch,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "~1.0.0",
      expected: Some("^1.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMatchConflictsWithSnapToVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "<1.0.0",
      expected: Some("<1.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsPreferVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: "~1.0.0",
      expected: Some("~1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SemverRangeMismatchConflictsWithSnapToVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::ValidLocal,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "0.1.0",
      expected: Some("0.1.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::SnapToVersionNotFound,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn instance_is_in_a_snapped_to_group_and_is_itself_a_snapped_to_target() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "dependencies": ["foo"],
      "snapTo": ["leader"]
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "leader",
      id: "leader in /version of leader",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::InvalidLocalVersion,
      dependency_name: "follower",
      id: "follower in /version of follower",
      actual: "VERSION_IS_MISSING",
      expected: Some("VERSION_IS_MISSING"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsSnapToVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of leader",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsSnapToVersion,
      dependency_name: "foo",
      id: "foo in /dependencies of follower",
      actual: "1.0.0",
      expected: Some("1.0.0"),
      overridden: None,
    },
  ]);
}

#[test]
fn refuses_to_snap_local_version_to_another_target() {
  let config = test::mock::config_from_mock(json!({
    "versionGroups": [{
      "snapTo": ["package-b"]
    }]
  }));
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
  let ctx = visit_packages(config, packages);
  expect(&ctx).to_have_instances(vec![
    ExpectedInstance {
      state: InstanceState::RefuseToSnapLocal,
      dependency_name: "package-a",
      id: "package-a in /version of package-a",
      actual: "1.1.0",
      expected: Some("1.1.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsSnapToVersion,
      dependency_name: "package-b",
      id: "package-b in /version of package-b",
      actual: "0.1.0",
      expected: Some("0.1.0"),
      overridden: None,
    },
    ExpectedInstance {
      state: InstanceState::EqualsSnapToVersion,
      dependency_name: "package-a",
      id: "package-a in /dependencies of package-b",
      actual: "0.0.1",
      expected: Some("0.0.1"),
      overridden: None,
    },
  ]);
}
