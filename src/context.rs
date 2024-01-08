use std::collections::HashMap;
use std::fs;
use std::io;
use std::path;

use crate::config;
use crate::config::Rcfile;
use crate::dependency_type::DependencyType;
use crate::instance::Instance;
use crate::package_json;
use crate::package_json::PackageJson;
use crate::semver_group::SemverGroup;
use crate::version_group::VersionGroup;

pub struct Ctx<'a> {
  /// Current working directory.
  pub cwd: std::path::PathBuf,
  /// Whether to exit with a non-zero exit code.
  pub is_invalid: bool,
  /// Every package.json file which matched the CLI options and rcfile.
  pub packages: Vec<package_json::PackageJson>,
  /// The user's configuration file.
  pub rcfile: config::Rcfile,

  pub enabled_dependency_types: HashMap<String, DependencyType>,
  pub semver_groups: Vec<SemverGroup<'a>>,
  pub version_groups: Vec<VersionGroup<'a>>,
  pub instances: Vec<Instance<'a>>,
}

impl<'a> Ctx<'a> {
  pub fn new(cwd: &std::path::PathBuf) -> Self {
    let rcfile = config::get();
    // let sources = rcfile.get_sources(&cwd);
    // let enabled_dependency_types = Rcfile::get_enabled_dependency_types(&rcfile);
    // let mut semver_groups = SemverGroup::from_rcfile(&rcfile);
    // let mut version_groups = VersionGroup::from_rcfile(&rcfile);
    // let mut instances: Vec<Instance> = vec![];
    // let packages: Vec<PackageJson> = sources
    //   .into_iter()
    //   .filter_map(|file_path| read_file(&cwd, &file_path).ok())
    //   .collect();

    // let instances: Vec<instance::Instance> = ctx
    //   .packages
    //   .iter()
    //   .flat_map(|package| package.get_instances(&enabled_dependency_types))
    //   .collect();

    // for package in &ctx.packages {
    //   package.get_instances(&ctx.enabled_dependency_types, &mut |instance| {
    //     println!("{:#?}", &instance);

    //     // 'assignToSemverGroup: for semver_group in &mut semver_groups {
    //     //   if semver_group.add_instance(&instance) {
    //     //     break 'assignToSemverGroup;
    //     //   }
    //     // }
    //     // 'assignToVersionGroup: for version_group in &mut version_groups {
    //     //   if version_group.add_instance(&instance) {
    //     //     break 'assignToVersionGroup;
    //     //   }
    //     // }

    //     // instances.push(instance);
    //   });
    // }

    Self {
      cwd: cwd.clone(),
      enabled_dependency_types: HashMap::new(),
      instances: vec![],
      is_invalid: false,
      packages: vec![],
      rcfile,
      semver_groups: vec![],
      version_groups: vec![],
    }
  }
}

/// Read and parse a package.json file
fn read_file<P: AsRef<path::Path>>(
  cwd: &std::path::PathBuf,
  file_path: &P,
) -> io::Result<package_json::PackageJson> {
  let json = fs::read_to_string(file_path)?;
  let contents: serde_json::Value = serde_json::from_str(&json)?;

  Ok(package_json::PackageJson {
    file_path: file_path.as_ref().to_path_buf(),
    json,
    contents,
  })
}
