use clap::{
  builder::ValueParser, crate_description, crate_name, crate_version, Arg, ArgMatches, Command,
};
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
  #[cfg(test)]
  pub fn new() -> Self {
    Cli {
      command_name: Subcommand::Lint,
      options: CliOptions::new(),
    }
  }

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
        )
        .arg(
          Arg::new("filter")
            .long("filter")
            .action(clap::ArgAction::Set)
            .value_parser(ValueParser::new(validate_filter))
            .help("only include dependencies whose name matches this regex"),
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
        )
        .arg(
          Arg::new("filter")
            .long("filter")
            .action(clap::ArgAction::Set)
            .value_parser(ValueParser::new(validate_filter))
            .help("only include dependencies whose name matches this regex"),
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
  /// Optional regex to filter dependencies by name
  pub filter: Option<Regex>,
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

impl CliOptions {
  #[cfg(test)]
  pub fn new() -> Self {
    Self {
      filter: None,
      format: false,
      ranges: true,
      versions: true,
      source: vec![],
    }
  }

  /// Create a new `CliOptions` from CLI arguments provided by the user
  pub fn from_arg_matches(matches: &ArgMatches) -> CliOptions {
    // 1. if no command is true, then all of them are true
    // 2. if any commands are true, then only those are true
    let use_format = matches.get_flag("format");
    let use_ranges = matches.get_flag("ranges");
    let use_versions = matches.get_flag("versions");
    let use_all = !use_format && !use_ranges && !use_versions;
    let source = matches
      .get_many::<String>("source")
      .unwrap_or_default()
      .map(|source| source.to_owned())
      .collect::<Vec<_>>();
    let filter = matches
      .get_one::<Regex>("filter")
      .map(|filter| filter.to_owned());

    CliOptions {
      filter,
      format: use_all || use_format,
      ranges: use_all || use_ranges,
      versions: use_all || use_versions,
      source,
    }
  }
}
