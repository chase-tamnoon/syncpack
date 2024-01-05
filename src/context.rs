use std::collections::HashMap;
use std::fs;
use std::io;
use std::path;

use crate::config;
use crate::instance::Instance;
use crate::package_json;

pub struct Ctx<'a> {
  /// Current working directory.
  pub cwd: std::path::PathBuf,
  /// Whether to exit with a non-zero exit code.
  pub is_invalid: bool,
  /// A lookup of all locally developed packages by name.
  pub local_instances_by_name: HashMap<String, &'a Instance<'a>>,
  /// Every package.json file which matched the CLI options and rcfile.
  pub packages: Vec<package_json::PackageJson>,
  /// The user's configuration file.
  pub rcfile: config::Rcfile,
}

impl Ctx<'_> {
  pub fn new(cwd: &std::path::PathBuf) -> Result<Self, io::Error> {
    let rcfile = config::get();
    let sources = rcfile.get_sources(&cwd);
    let packages: Vec<package_json::PackageJson> = sources
      .into_iter()
      .filter_map(|file_path| read_file(&cwd, &file_path).ok())
      .collect();

    Ok(Self {
      cwd: cwd.clone(),
      is_invalid: false,
      local_instances_by_name: HashMap::new(),
      packages,
      rcfile,
    })
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
