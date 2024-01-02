use std::fs;
use std::io;
use std::path;

use crate::config;
use crate::package_json;
use crate::semver_group;
use crate::version_group;

pub struct Ctx {
  pub cwd: std::path::PathBuf,
  pub is_invalid: bool,
  pub packages: Vec<package_json::Package>,
  pub rcfile: config::Rcfile,
  pub semver_groups: Vec<semver_group::SemverGroup>,
  pub version_groups: Vec<version_group::VersionGroup>,
}

impl Ctx {
  pub fn new(cwd: &std::path::PathBuf) -> Result<Self, io::Error> {
    let rcfile = config::get();
    let sources = rcfile.get_sources(&cwd);
    let packages: Vec<package_json::Package> = sources
      .into_iter()
      .filter_map(|file_path| read_file(&cwd, &file_path).ok())
      .collect();
    let semver_groups = rcfile
      .semver_groups
      .iter()
      .map(|group| semver_group::SemverGroup::from_config(group))
      .collect();
    let version_groups = rcfile
      .version_groups
      .iter()
      .map(|group| version_group::VersionGroup::from_config(group))
      .collect();

    Ok(Self {
      cwd: cwd.clone(),
      is_invalid: false,
      packages,
      rcfile,
      semver_groups,
      version_groups,
    })
  }
}

/// Read and parse a package.json file
fn read_file<P: AsRef<path::Path>>(
  cwd: &std::path::PathBuf,
  file_path: &P,
) -> io::Result<package_json::Package> {
  let file_contents = fs::read_to_string(file_path)?;
  let parsed_json: serde_json::Value = serde_json::from_str(&file_contents)?;

  Ok(package_json::Package {
    contents: parsed_json,
    json: file_contents,
    file_path: file_path.as_ref().to_path_buf(),
    short_path: file_path
      .as_ref()
      .strip_prefix(&cwd)
      .unwrap()
      .to_str()
      .unwrap()
      .to_string(),
  })
}
