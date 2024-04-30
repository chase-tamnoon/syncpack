use log::debug;

use crate::config;
use crate::package_json;
use crate::semver_group::SemverGroup;

/// Check whether all version ranges are according to config
/// Returns true if all are valid
pub fn lint_all(semver_groups: &Vec<SemverGroup>) -> bool {
  semver_groups.iter().for_each(|group| {
    // group.instances.iter().for_each(|instance| {
    //   println!("{} {:?}", instance.name, instance.expected_range);
    // })
  });
  false
}

/// Format every package according to config
/// Returns true if all are were fixable
pub fn fix_all(
  cwd: &std::path::PathBuf,
  rcfile: &config::Rcfile,
  packages: &mut Vec<package_json::PackageJson>,
) -> bool {
  debug!("@TODO fix semver range mismatches");
  false
}
