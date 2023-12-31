extern crate glob;
extern crate serde;
extern crate serde_json;

mod dependencies;
mod file_paths;
mod package_json;

use std::{io, path::Path};
use serde_json::json;

fn main() -> io::Result<()> {
  // let pattern = "/Users/foldleft/Dev/FluidFramework/packages/*/*/package.json";
  let pattern = "/Users/foldleft/Dev/FluidFramework/package.json";
  let paths = file_paths::get_paths(pattern);

  println!("PATHS: {:?}", paths);

  paths.into_iter().try_for_each(|file_path| {
    match package_json::read_file(&file_path) {
      Ok(mut package) => {
        let file_path: &Path = package.file_path();
        println!("file_path: {:?}", file_path);

        // Example of mutating the JSON object
        if let Some(name) = package.contents.pointer_mut("/name") {
          *name = json!("new value");
        }
        println!("Updated JSON: {:?}", package);
      }
      Err(e) => println!("Error reading file: {}", e),
    }
    // let package = package_json::get_package(path)?;
    // dependencies::list_dependencies(&package);
    Ok(())
  })
}
