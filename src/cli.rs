use clap::{
  builder::ValueParser, crate_description, crate_name, crate_version, Arg, ArgMatches, Command,
};

extern crate glob;
extern crate serde;
extern crate serde_json;

pub fn create() -> Command {
  Command::new(crate_name!())
    .about(crate_description!())
    .version(crate_version!())
    .subcommand(
      Command::new("lint")
        .about("Lint command")
        .arg(
          Arg::new("format")
            .short('f')
            .long("format")
            .action(clap::ArgAction::SetTrue)
            .help("enable to lint the formatting and order of package.json files"),
        )
        .arg(
          Arg::new("ranges")
            .short('r')
            .long("ranges")
            .action(clap::ArgAction::SetTrue)
            .help("enable to lint semver range mismatches"),
        )
        .arg(
          Arg::new("versions")
            .short('v')
            .long("versions")
            .action(clap::ArgAction::SetTrue)
            .help("enable to lint version mismatches"),
        )
        .arg(
          Arg::new("source")
            .short('s')
            .long("source")
            .action(clap::ArgAction::Append)
            .value_parser(ValueParser::new(validate_source))
            .help("a list of quoted glob patterns for package.json files to read from"),
        ),
    )
    .subcommand(
      Command::new("fix")
        .about("Fix command")
        .arg(
          Arg::new("format")
            .short('f')
            .long("format")
            .action(clap::ArgAction::SetTrue)
            .help("enable to fix the formatting and order of package.json files"),
        )
        .arg(
          Arg::new("ranges")
            .short('r')
            .long("ranges")
            .action(clap::ArgAction::SetTrue)
            .help("enable to fix semver range mismatches"),
        )
        .arg(
          Arg::new("versions")
            .short('v')
            .long("versions")
            .action(clap::ArgAction::SetTrue)
            .help("enable to fix version mismatches"),
        )
        .arg(
          Arg::new("source")
            .short('s')
            .long("source")
            .action(clap::ArgAction::Append)
            .value_parser(ValueParser::new(validate_source))
            .help("a list of quoted glob patterns for package.json files to read from"),
        ),
    )
}

fn validate_source(value: &str) -> Result<String, String> {
  if value.ends_with("package.json") {
    Ok(value.to_string())
  } else {
    Err("Source file must end with 'package.json'".to_string())
  }
}

#[derive(Debug)]
pub struct CliOptions {
  /// `true` when `--format` is passed or if none of `--format`, `--ranges`
  /// or `--versions` are passed
  pub format: bool,
  /// `true` when `--ranges` is passed or if none of `--format`, `--ranges`
  /// or `--versions` are passed
  pub ranges: bool,
  /// `true` when `--versions` is passed or if none of `--format`, `--ranges`
  /// or `--versions` are passed
  pub versions: bool,
  /// Optional glob patterns to package.json files
  pub source: Vec<String>,
}

/// returns which steps to run. if none are true, then all of them are returned
/// as true. if any of them are true, then only those are returned as true.
pub fn get_cli_options(matches: &ArgMatches) -> CliOptions {
  let use_format = matches.get_flag("format");
  let use_ranges = matches.get_flag("ranges");
  let use_versions = matches.get_flag("versions");
  let use_all = !use_format && !use_ranges && !use_versions;
  let sources = matches
    .get_many::<String>("source")
    .unwrap_or_default()
    .map(|source| source.to_owned())
    .collect::<Vec<_>>();

  CliOptions {
    format: use_all || use_format,
    ranges: use_all || use_ranges,
    versions: use_all || use_versions,
    source: sources,
  }
}
