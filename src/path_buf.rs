/// Convert a PathBuf to a String
pub fn path_buf_to_string(file_path: &std::path::PathBuf) -> String {
  path_buf_to_str(file_path).to_string()
}

/// Convert a PathBuf to a &str
pub fn path_buf_to_str(file_path: &std::path::PathBuf) -> &str {
  file_path.to_str().unwrap()
}

/// Convert a PathBuf to a String
pub fn path_to_string(file_path: &std::path::Path) -> String {
  file_path.to_str().unwrap().to_string()
}
