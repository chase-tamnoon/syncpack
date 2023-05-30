// get 0-* file paths from 1 glob pattern
pub fn get_paths(pattern: &str) -> Vec<String> {
  glob::glob(pattern)
    .expect("Failed to read glob pattern")
    .filter_map(Result::ok)
    .map(|path_buffer| path_buffer.into_os_string().into_string().ok())
    .flatten()
    .collect()
}
