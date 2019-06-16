pub fn get_dependencies(prop_name: &str, package: &serde_json::Value) -> Vec<(String, String)> {
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
