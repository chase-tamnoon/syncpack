extern crate glob;
extern crate serde;
extern crate serde_json;

use glob::glob;

mod dependencies;
mod read_file;

fn get_paths(pattern: &str) -> Vec<String> {
  let mut paths: Vec<String> = vec![];
  for entry in glob(pattern).expect("Failed to read glob pattern") {
    match entry {
      Err(e) => println!("{:?}", e),
      Ok(path) => match path.to_str() {
        None => panic!("new path is not a valid UTF-8 sequence"),
        Some(path_str) => {
          paths.push(String::from(path_str));
        }
      },
    };
  }
  paths
}

fn main() -> std::io::Result<()> {
  let pattern = "/Users/jmn42/Dev/pages-lib/packages/*/package.json";
  let paths = get_paths(pattern);
  println!("PATHS: {:?}", paths);
  for path in paths {
    println!("{}", path);
    let package = read_file::read_package_json(path)?;
    let deps = dependencies::get_dependencies("dependencies", &package);
    let dev_deps = dependencies::get_dependencies("devDependencies", &package);
    let peer_deps = dependencies::get_dependencies("peerDependencies", &package);
    println!("    dependencies");
    for dep in deps {
      println!("        {}: {}", dep.0, dep.1);
    }
    println!("    dev_dependencies");
    for dep in dev_deps {
      println!("        {}: {}", dep.0, dep.1);
    }
    println!("    peer_dependencies");
    for dep in peer_deps {
      println!("        {}: {}", dep.0, dep.1);
    }
  }
  Ok(())
}
