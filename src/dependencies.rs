use super::package_json;
use serde_json::Value as JsonValue;

pub fn get_dependencies(prop_name: &str, package: &JsonValue) -> Vec<(String, String)> {
  let mut entries: Vec<(String, String)> = vec![];
  let option = package[prop_name].as_object();
  match option {
    None => {}
    Some(versions_by_name) => {
      for key in versions_by_name.iter() {
        let name: &str = key.0;
        let version: &str = key.1.as_str().unwrap();
        let entry: (String, String) = (String::from(name), String::from(version));
        entries.push(entry);
      }
    }
  }
  entries
}

pub fn list_dependencies(package: &package_json::Package) {
  let deps = get_dependencies("dependencies", &package.data);
  let dev_deps = get_dependencies("devDependencies", &package.data);
  let peer_deps = get_dependencies("peerDependencies", &package.data);
  println!("PACKAGE: {}", package.path);
  println!("  DEPENDENCIES");
  for dep in deps {
    println!("    {}: {}", dep.0, dep.1);
  }
  println!("  DEV_DEPENDENCIES");
  for dep in dev_deps {
    println!("    {}: {}", dep.0, dep.1);
  }
  println!("  PEER_DEPENDENCIES");
  for dep in peer_deps {
    println!("    {}: {}", dep.0, dep.1);
  }
}
