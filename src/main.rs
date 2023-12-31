extern crate glob;
extern crate serde;
extern crate serde_json;

mod dependencies;
mod file_paths;
mod package_json;

use std::io;
use serde_json::json;

fn main() -> io::Result<()> {
  // let pattern = "/Users/foldleft/Dev/FluidFramework/packages/*/*/package.json";
  let pattern = "/Users/foldleft/Dev/FluidFramework/package.json";
  let paths = file_paths::get_paths(pattern);

  println!("PATHS: {:?}", paths);

  paths.into_iter().try_for_each(|file_path| {
    match package_json::read_file(file_path) {
      Ok(mut package) => {
        // Example of mutating the JSON object
        let mut contents = package.contents;
        if let Some(name) = contents.pointer_mut("/name") {
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
