// Standard library imports
use std::{io, path::Path};

// External crates
extern crate glob;
extern crate serde;
extern crate serde_json;

// Local modules
mod dependencies;
mod file_paths;
mod package_json;

// Imports from external crates
use serde_json::json;
use std::collections::BTreeMap;

fn main() -> io::Result<()> {
  let pattern = "/Users/foldleft/Dev/FluidFramework/package.json";
  let paths = file_paths::get_paths(pattern);

  println!("PATHS: {:?}", paths);

  paths
    .into_iter()
    .map(|file_path| package_json::read_file(&file_path))
    .filter_map(Result::ok)
    .for_each(|mut package| {
      let file_path: &Path = package.file_path();
      println!("file_path: {:?}", file_path);

      if let Some(name) = package.contents.pointer_mut("/name") {
        *name = json!("new value");
      }

      // Sort the package.contents object alphabetically by keys
      if let Some(contents) = package.contents.as_object_mut() {
        let sorted_contents: BTreeMap<_, _> = contents.into_iter().collect();
        package.contents = json!(sorted_contents);
      }

      println!("Updated JSON: {:#?}", package.contents);
    });

  Ok(())
}
