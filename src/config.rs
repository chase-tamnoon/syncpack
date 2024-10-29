use {
  crate::{cli::Cli, rcfile::Rcfile},
  std::path::PathBuf,
};

#[derive(Debug)]
pub struct Config {
  pub cli: Cli,
  pub cwd: PathBuf,
  pub rcfile: Rcfile,
}

impl Config {
  /// Try to read the rcfile from the current working directory and fall back to
  /// defaults if one is not found
  pub fn from_cli(cwd: PathBuf, cli: Cli, rcfile_json: String) -> Config {
    let file_path = cwd.join(".syncpackrc.json");
    Config {
      cli,
      cwd,
      rcfile: Rcfile::from_json(rcfile_json),
    }
  }
}
