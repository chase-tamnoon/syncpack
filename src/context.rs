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
    cwd: std::path::PathBuf,
    rcfile: config::Rcfile,
  ) -> Result<Self, io::Error> {
    // let pattern = cwd.join("fixtures/**/package.json");
    // let pattern_str = pattern.to_str().unwrap();
    // let paths = file_paths::get_file_paths(pattern_str);
    let paths = rcfile.get_sources(cwd.clone());

    let packages = paths
      .into_iter()
      .filter_map(|file_path| package_json::read_file(&file_path).ok())
      .collect();

    Ok(Self {
      cwd: std::env::current_dir()?,
      is_invalid: false,
      packages: vec![],
      rcfile,
    })
  }
}
