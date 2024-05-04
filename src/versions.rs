use log::debug;

use crate::config;
use crate::package_json;
use crate::version_group::VersionGroup;

/// Check whether all versions are according to config
/// Returns true if all are valid
pub fn is_valid(version_groups: &Vec<VersionGroup>) -> bool {
  debug!("@TODO lint version mismatches");
  false
}

/// Format every package according to config
/// Returns true if all are were fixable
pub fn fix(
  cwd: &std::path::PathBuf,
  rcfile: &config::Rcfile,
  packages: &mut Vec<package_json::PackageJson>,
) -> bool {
  debug!("@TODO: fix version mismatches");
  false
}
