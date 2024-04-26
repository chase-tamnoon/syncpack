use clap::{Arg, ArgMatches, Command};

extern crate glob;
extern crate serde;
extern crate serde_json;

pub fn create() -> Command {
  Command::new("syncpack")
    .about("Consistent dependency versions in large JavaScript Monorepos")
    .version("13.0.0")
    .author("Jamie Mason <jamie@foldleft.io> (https://github.com/JamieMason)")
    .subcommand(Command::new("list").about("List command"))
    .subcommand(
      Command::new("lint")
        .about("Lint command")
        .arg(
          Arg::new("format")
            .long("format")
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
          Arg::new("ranges")
            .long("ranges")
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
          Arg::new("versions")
            .long("versions")
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
          Arg::new("ranges")
            .long("ranges")
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
          Arg::new("versions")
            .long("versions")
            .action(clap::ArgAction::SetTrue),
        ),
    )
}

#[derive(Debug)]
pub struct EnabledSteps {
  pub format: bool,
  pub ranges: bool,
  pub versions: bool,
}

/// returns which steps to run. if none are true, then all of them are returned
/// as true. if any of them are true, then only those are returned as true.
pub fn get_enabled_steps(matches: &ArgMatches) -> EnabledSteps {
  let use_format = matches.get_flag("format");
  let use_ranges = matches.get_flag("ranges");
  let use_versions = matches.get_flag("versions");
  let use_all = !use_format && !use_ranges && !use_versions;

  EnabledSteps {
    format: use_all || use_format,
    ranges: use_all || use_ranges,
    versions: use_all || use_versions,
  }
}
