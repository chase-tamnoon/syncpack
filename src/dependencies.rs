use super::package_json;
use serde_json::Value as JsonValue;

pub fn get_dependencies(
  prop_name: &str,
  package: &JsonValue,
) -> Vec<(String, String)> {
  package
    .get(prop_name)
    .and_then(JsonValue::as_object)
    .map(|versions_by_name| {
      versions_by_name
        .iter()
        .map(|(name, version)| {
          (name.to_string(), version.as_str().unwrap().to_string())
        })
        .collect()
    })
    .unwrap_or_else(Vec::new)
}

pub fn list_dependencies(package: &package_json::Package) {
  let dependencies = ["dependencies", "devDependencies", "peerDependencies"];

  println!("PACKAGE: {}", package.path);

  dependencies.iter().for_each(|prop_name| {
    println!("  {}", prop_name);
    get_dependencies(prop_name, &package.data)
      .iter()
      .for_each(|dep| {
        println!("    {}: {}", dep.0, dep.1);
      });
  });
}
