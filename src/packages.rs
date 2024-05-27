use glob::glob;
use std::{collections::HashMap, path::PathBuf};

use crate::{cli::CliOptions, config::Rcfile, json_file::read_json_file, package_json::Packages};

/// Get every package.json file matched by the user's source patterns
pub fn get_packages(cwd: &PathBuf, cli_options: &CliOptions, rcfile: &Rcfile) -> Packages {
  let file_paths = get_file_paths(&cwd, &cli_options, &rcfile);
  let mut packages = Packages {
    all_names: vec![],
    by_name: HashMap::new(),
  };
  for file_path in file_paths {
    if let Ok(file) = read_json_file(&file_path) {
      let name = file.get_name();
      packages.all_names.push(name.clone());
      packages.by_name.insert(name.clone(), file);
    }
  }
  packages
}

/// Resolve every source glob pattern into their absolute file paths of
/// package.json files
fn get_file_paths(cwd: &PathBuf, cli_options: &CliOptions, rcfile: &Rcfile) -> Vec<PathBuf> {
  get_source_patterns(cli_options, rcfile)
    .iter()
    .map(|pattern| {
      if PathBuf::from(pattern).is_absolute() {
        pattern.clone()
      } else {
        cwd.join(pattern).to_str().unwrap().to_string()
      }
    })
    .flat_map(|pattern| glob(&pattern).ok())
    .flat_map(|paths| {
      paths
        .filter_map(Result::ok)
        .fold(vec![], |mut paths, path| {
          paths.push(path.clone());
          paths
        })
    })
    .collect()
}

/// Based on the user's config file and command line `--source` options, return
/// the source glob patterns which should be used to resolve package.json files
fn get_source_patterns(cli_options: &CliOptions, rcfile: &Rcfile) -> Vec<String> {
  get_cli_patterns(cli_options)
    .or_else(|| get_rcfile_patterns(rcfile))
    .or_else(get_npm_patterns)
    .or_else(get_pnpm_patterns)
    .or_else(get_yarn_patterns)
    .or_else(get_lerna_patterns)
    .or_else(get_default_patterns)
    .unwrap()
}

fn get_cli_patterns(cli_options: &CliOptions) -> Option<Vec<String>> {
  if cli_options.source.is_empty() {
    return None;
  } else {
    return Some(cli_options.source.clone());
  }
}

fn get_rcfile_patterns(rcfile: &Rcfile) -> Option<Vec<String>> {
  if rcfile.source.is_empty() {
    return None;
  } else {
    return Some(rcfile.source.clone());
  }
}

fn get_npm_patterns() -> Option<Vec<String>> {
  None
}

fn get_pnpm_patterns() -> Option<Vec<String>> {
  None
}

fn get_yarn_patterns() -> Option<Vec<String>> {
  None
}

fn get_lerna_patterns() -> Option<Vec<String>> {
  None
}

fn get_default_patterns() -> Option<Vec<String>> {
  Some(vec![
    String::from("package.json"),
    String::from("packages/*/package.json"),
  ])
}
