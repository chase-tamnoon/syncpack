pub fn get_paths(pattern: &str) -> Vec<String> {
  let mut file_paths: Vec<String> = vec![];
  let glob_paths = glob::glob(pattern).expect("Failed to read glob pattern");
  for glob_path in glob_paths {
    let path_buffer = glob_path.expect("Failed to unwrap glob entry");
    match path_buffer.to_str() {
      None => panic!("New path is not a valid UTF-8 sequence"),
      Some(path_str) => {
        file_paths.push(String::from(path_str));
      }
    }
  }
  file_paths
}
