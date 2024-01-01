use std::io;

extern crate glob;
extern crate serde;
extern crate serde_json;

mod config;
mod dependencies;
mod file_paths;
mod format;
mod package_json;

fn main() -> io::Result<()> {
  let cwd = std::env::current_dir()?;
  let pattern = cwd.join("fixtures/**/package.json");
  let pattern_str = pattern.to_str().unwrap();
  let paths = file_paths::get_file_paths(pattern_str);
  let rcfile = config::get();
  let packages = paths
    .into_iter()
    .filter_map(|file_path| package_json::read_file(&file_path).ok());

  packages
    .for_each(|mut package| format::format_package(&mut package, &rcfile));

  Ok(())
}
