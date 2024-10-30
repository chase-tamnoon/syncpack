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
  /// Read the rcfile from stdin and fall back to defaults if none was sent
  pub fn from_cli(cwd: PathBuf, cli: Cli) -> Config {
    Config {
      cli,
      cwd,
      rcfile: Rcfile::from_stdin(),
    }
  }
}
