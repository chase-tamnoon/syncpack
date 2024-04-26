use log::debug;

use crate::config;
use crate::package_json;

/// Check whether all version ranges are according to config
/// Returns true if all are valid
pub fn lint_all(
  cwd: &std::path::PathBuf,
  rcfile: &config::Rcfile,
  packages: &mut Vec<package_json::PackageJson>,
) -> bool {
  debug!("@TODO lint semver range mismatches");
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
