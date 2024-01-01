use std::io;

use crate::config;
use crate::file_paths;
use crate::package_json;

pub struct Ctx {
  pub cwd: std::path::PathBuf,
  pub is_invalid: bool,
  pub packages: Vec<package_json::Package>,
  pub rcfile: config::Rcfile,
}

impl Ctx {
  pub fn new(
    cwd: &std::path::PathBuf,
    rcfile: config::Rcfile,
  ) -> Result<Self, io::Error> {
    let sources = rcfile.get_sources(&cwd);
    let packages: Vec<package_json::Package> = sources
      .into_iter()
      .filter_map(|file_path| package_json::read_file(&file_path).ok())
      .collect();

    Ok(Self {
      cwd: cwd.clone(),
      is_invalid: false,
      packages,
      rcfile,
    })
  }
}
