// Standard library imports
use std::io;

// External crates
extern crate glob;
extern crate serde;
extern crate serde_json;

// Local modules
mod config;
mod dependencies;
mod file_paths;
mod format;
mod package_json;

// Imports from external crates
use serde_json::json;

fn main() -> io::Result<()> {
  let cwd = std::env::current_dir()?;
  let pattern = cwd.join("fixtures/**/package.json");
  let pattern_str = pattern.to_str().unwrap();
  let paths = file_paths::get_file_paths(pattern_str);
  let rcfile = config::get();

  paths
    .into_iter()
    .map(|file_path| package_json::read_file(&file_path))
    .filter_map(Result::ok)
    .for_each(|mut package| {
      package.set_prop("/name", json!("new name"));
      package.set_prop("/engines/node", json!(">=1"));

      rcfile.sort_az.iter().for_each(|key| {
        let prop = package.contents.pointer_mut(format!("/{}", key).as_str());
        if let Some(pointer) = prop {
          format::sort_alphabetically(pointer);
        }
      });

      format::sort_first(&mut package.contents, &rcfile.sort_first);
      package.pretty_print();
    });

  Ok(())
}
