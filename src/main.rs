use clap::{arg, Command};
use std::io;

extern crate glob;
extern crate serde;
extern crate serde_json;

mod config;
mod dependencies;
mod file_paths;
mod format;
mod package_json;

fn main() -> io::Result<()> {
  let matches = Command::new("syncpack")
    .about("Consistent dependency versions in large JavaScript Monorepos")
    .version("13.0.0")
    .author("Jamie Mason <jamie@foldleft.io> (https://github.com/JamieMason)")
    .subcommand(
      Command::new("lint")
        .about("Lint command")
        .arg(arg!(--format))
        .arg(arg!(--versions))
        .arg(arg!(--ranges)),
    )
    .subcommand(
      Command::new("fix")
        .about("Fix command")
        .arg(arg!(--format))
        .arg(arg!(--versions))
        .arg(arg!(--ranges)),
    )
    .get_matches();

  // You can then check which subcommand was used and which flags were set
  match matches.subcommand() {
    Some(("lint", lint_matches)) => {
      let format_is_set = lint_matches.get_one::<bool>("format");
      let versions_is_set = lint_matches.get_one::<bool>("versions");
      let ranges_is_set = lint_matches.get_one::<bool>("ranges");
      println!("lint format: {:?}", format_is_set);
      println!("lint versions: {:?}", versions_is_set);
      println!("lint ranges: {:?}", ranges_is_set);
      Ok(())
    }
    Some(("fix", fix_matches)) => {
      let format_is_set = fix_matches.get_one::<bool>("format");
      let versions_is_set = fix_matches.get_one::<bool>("versions");
      let ranges_is_set = fix_matches.get_one::<bool>("ranges");
      println!("fix format: {:?}", format_is_set);
      println!("fix versions: {:?}", versions_is_set);
      println!("fix ranges: {:?}", ranges_is_set);
      Ok(())
    }
    _ => Err(create_error("unrecognized subcommand")),
  }
}

fn create_error(message: &str) -> io::Error {
  io::Error::new(io::ErrorKind::Other, message)
}

fn format_lint() -> Result<(), io::Error> {
  let mut is_invalid = false;
  let cwd = std::env::current_dir()?;
  let pattern = cwd.join("fixtures/**/package.json");
  let pattern_str = pattern.to_str().unwrap();
  let paths = file_paths::get_file_paths(pattern_str);
  let rcfile = config::get();
  let packages = paths
    .into_iter()
    .filter_map(|file_path| package_json::read_file(&file_path).ok());

  packages.for_each(|mut package| {
    format::fix(&mut package, &rcfile);
    if package.has_changed() {
      is_invalid = true;
    }
    package.pretty_print();
  });

  if is_invalid {
    println!("Invalid package.json files found. Please run `syncpack fix --format` to fix them.")
  }

  Ok(())
}
