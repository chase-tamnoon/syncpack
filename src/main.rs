extern crate glob;
extern crate serde;
extern crate serde_json;

mod dependencies;
mod file_paths;
mod package_json;

fn main() -> std::io::Result<()> {
  let pattern = "/Users/jmn42/Dev/pages-lib/packages/*/package.json";
  let paths = file_paths::get_paths(pattern);
  println!("PATHS: {:?}", paths);
  for path in paths {
    let package = package_json::get_package(path)?;
    dependencies::list_dependencies(&package);
  }
  Ok(())
}
