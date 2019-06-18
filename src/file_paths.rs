pub fn get_paths(pattern: &str) -> Vec<String> {
  let mut paths: Vec<String> = vec![];
  for entry in glob::glob(pattern).expect("Failed to read glob pattern") {
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
