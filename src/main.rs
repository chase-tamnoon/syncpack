extern crate glob;
extern crate serde;
extern crate serde_json;

mod dependencies;
mod file_paths;
mod package_json;

use std::io;

fn main() -> io::Result<()> {
  // let pattern = "/Users/foldleft/Dev/FluidFramework/packages/*/*/package.json";
  let pattern = "/Users/foldleft/Dev/FluidFramework/package.json";
  let paths = file_paths::get_paths(pattern);

  println!("PATHS: {:?}", paths);

  paths.into_iter().try_for_each(|path| {
    match package_json::read_and_parse_json(path) {
      Ok(mut json) => {
        // Example of mutating the JSON object
        json["name"] = serde_json::Value::from("NewName");
        println!("Updated JSON: {:?}", json);
      }
      Err(e) => println!("Error reading file: {}", e),
    }
    // let package = package_json::get_package(path)?;
    // dependencies::list_dependencies(&package);
    Ok(())
  })
}
