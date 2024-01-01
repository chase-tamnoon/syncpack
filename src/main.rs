use clap::{arg, Arg, ArgMatches, Command};
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
        .arg(
          Arg::new("format")
            .long("format")
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
          Arg::new("versions")
            .long("versions")
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
          Arg::new("ranges")
            .long("ranges")
            .action(clap::ArgAction::SetTrue),
        ),
    )
    .subcommand(
      Command::new("fix")
        .about("Fix command")
        .arg(
          Arg::new("format")
            .long("format")
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
          Arg::new("versions")
            .long("versions")
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
          Arg::new("ranges")
            .long("ranges")
            .action(clap::ArgAction::SetTrue),
        ),
    )
    .get_matches();

  // You can then check which subcommand was used and which flags were set
  match matches.subcommand() {
    Some(("lint", lint_matches)) => {
      println!("lint enabled: {:?}", get_enabled_steps(lint_matches));
      Ok(())
    }
    Some(("fix", fix_matches)) => {
      println!("fix enabled: {:?}", get_enabled_steps(fix_matches));
      Ok(())
    }
    _ => Err(create_error("unrecognized subcommand")),
  }
}

/// returns which steps to run from "format", "versions", "ranges". if none are true, then all of them are returned as true.
/// if any of them are true, then only those are returned as true.
fn get_enabled_steps(matches: &ArgMatches) -> (bool, bool, bool) {
  let format_is_set = matches.get_flag("format");
  let versions_is_set = matches.get_flag("versions");
  let ranges_is_set = matches.get_flag("ranges");

  if !format_is_set && !versions_is_set && !ranges_is_set {
    return (true, true, true);
  }

  (format_is_set, versions_is_set, ranges_is_set)
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
