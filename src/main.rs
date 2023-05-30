extern crate glob;
extern crate serde;
extern crate serde_json;

mod dependencies;
mod file_paths;
mod package_json;

use std::io;

fn main() -> io::Result<()> {
  let pattern = "/Users/foldleft/Dev/FluidFramework/packages/*/*/package.json";
  let paths = file_paths::get_paths(pattern);

  println!("PATHS: {:?}", paths);

  paths.into_iter().try_for_each(|path| {
    let package = package_json::get_package(path)?;
    dependencies::list_dependencies(&package);
    Ok(())
  })
}
