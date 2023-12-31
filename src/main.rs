// Standard library imports
use std::io;

// External crates
extern crate glob;
extern crate serde;
extern crate serde_json;

// Local modules
mod dependencies;
mod file_paths;
mod format;
mod package_json;

// Imports from external crates
use serde_json::json;

fn main() -> io::Result<()> {
  let pattern = "/Users/foldleft/Dev/tightrope/package.json";
  let paths = file_paths::get_file_paths(pattern);
  let sort_first = &vec![
    "private".to_string(),
    "homepage".to_string(),
    "name".to_string(),
    "version".to_string(),
    "description".to_string(),
  ];

  paths
    .into_iter()
    .map(|file_path| package_json::read_file(&file_path))
    .filter_map(Result::ok)
    .for_each(|mut package| {
      package.set_prop("/name", json!("new name"));
      package.set_prop("/c8/cache-dir", json!("new cache-dir"));
      format::sort_alphabetically(&mut package.contents);
      format::sort_first(&mut package.contents, sort_first);
      package.pretty_print();
    });

  Ok(())
}
