use clap::{builder::ValueParser, crate_description, crate_name, crate_version, Arg, ArgMatches, Command};
use log::LevelFilter;
use regex::Regex;

#[derive(Debug)]
pub enum Subcommand {
  Lint,
  Fix,
}

#[derive(Debug)]
pub struct Cli {
  pub command_name: Subcommand,
  pub options: CliOptions,
}

impl Cli {
  pub fn parse() -> Cli {
    match create().get_matches().subcommand() {
      Some(("lint", matches)) => Cli {
        command_name: Subcommand::Lint,
        options: CliOptions::from_arg_matches(matches),
      },
      Some(("fix", matches)) => Cli {
        command_name: Subcommand::Fix,
        options: CliOptions::from_arg_matches(matches),
      },
      _ => {
        std::process::exit(1);
      }
    }
  }
}

fn create() -> Command {
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
        )
        .arg(
          Arg::new("filter")
            .long("filter")
            .action(clap::ArgAction::Set)
            .value_parser(ValueParser::new(validate_filter))
            .help("only include dependencies whose name matches this regex"),
        )
        .arg(
          Arg::new("log-levels")
            .long("log-levels")
            .value_delimiter(',')
            .value_parser(["off", "error", "warn", "info", "debug"])
            .default_values(["error", "warn", "info"])
            .help("control how detailed log output should be"),
        )
        .arg(
          Arg::new("no-color")
            .long("no-color")
            .action(clap::ArgAction::SetTrue)
            .help("disable colored output"),
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
        )
        .arg(
          Arg::new("filter")
            .long("filter")
            .action(clap::ArgAction::Set)
            .value_parser(ValueParser::new(validate_filter))
            .help("only include dependencies whose name matches this regex"),
        )
        .arg(
          Arg::new("log-levels")
            .long("log-levels")
            .value_delimiter(',')
            .value_parser(["off", "error", "warn", "info", "debug"])
            .default_values(["error", "warn", "info"])
            .help("control how detailed log output should be"),
        )
        .arg(
          Arg::new("no-color")
            .long("no-color")
            .action(clap::ArgAction::SetTrue)
            .help("disable colored output"),
        ),
    )
}

fn validate_filter(value: &str) -> Result<Regex, String> {
  Regex::new(value).map_err(|_| "not a valid Regex".to_string())
}

fn validate_source(value: &str) -> Result<String, String> {
  if value.ends_with("package.json") {
    Ok(value.to_string())
  } else {
    Err("must end with 'package.json'".to_string())
  }
}

#[derive(Debug)]
pub struct CliOptions {
  /// Disable colored output
  pub disable_color: bool,
  /// Optional regex to filter dependencies by name
  pub filter: Option<Regex>,
  /// `true` when `--format` is passed or if none of `--format`, `--ranges`
  /// or `--versions` are passed
  pub format: bool,
  /// How detailed the terminal output should be
  pub log_levels: Vec<LevelFilter>,
  /// Optional glob patterns to package.json files
  pub source: Vec<String>,
  /// `true` when `--versions` is passed or if none of `--format`, `--ranges`
  /// or `--versions` are passed
  pub versions: bool,
}

impl CliOptions {
  /// Create a new `CliOptions` from CLI arguments provided by the user
  pub fn from_arg_matches(matches: &ArgMatches) -> CliOptions {
    // 1. if no command is true, then all of them are true
    // 2. if any commands are true, then only those are true
    let use_format = matches.get_flag("format");
    let use_versions = matches.get_flag("versions");
    let use_all = !use_format && !use_versions;

    CliOptions {
      disable_color: matches.get_flag("no-color"),
      filter: matches.get_one::<String>("filter").map(|filter| Regex::new(filter).unwrap()),
      format: use_all || use_format,
      log_levels: matches
        .get_many::<String>("log-levels")
        .unwrap()
        .map(|level| match level.as_str() {
          "off" => LevelFilter::Off,
          "error" => LevelFilter::Error,
          "warn" => LevelFilter::Warn,
          "info" => LevelFilter::Info,
          "debug" => LevelFilter::Debug,
          _ => unreachable!(),
        })
        .collect(),
      source: matches
        .get_many::<String>("source")
        .unwrap_or_default()
        .map(|source| source.to_owned())
        .collect::<Vec<_>>(),
      versions: use_all || use_versions,
    }
  }
}
