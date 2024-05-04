use log::debug;

use crate::config;
use crate::package_json;
use crate::version_group::VersionGroup;
use crate::version_group::VersionGroupVariant;

/// Check whether all versions are according to config
/// Returns true if all are valid
pub fn is_valid(version_groups: &Vec<VersionGroup>) -> bool {
  version_groups.iter().for_each(|version_group| {
    println!("== {}", &version_group.selector.label);
    if matches!(&version_group.variant, VersionGroupVariant::Standard) {
      version_group
        .instance_groups_by_name
        .iter()
        .for_each(|(name, instance_group)| {
          println!("{} {:?}", name, instance_group.expected_version);
        });
    }
  });
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
