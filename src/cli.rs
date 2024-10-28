use {
  clap::{builder::ValueParser, crate_description, crate_name, crate_version, Arg, ArgMatches, Command},
  log::LevelFilter,
  regex::Regex,
};

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

fn format_option() -> Arg {
  Arg::new("format")
    .short('f')
    .long("format")
    .action(clap::ArgAction::SetTrue)
    .help("Enable to lint the formatting and order of package.json files")
}

fn versions_option() -> Arg {
  Arg::new("versions")
    .short('v')
    .long("versions")
    .action(clap::ArgAction::SetTrue)
    .help("Enable to lint version mismatches")
}

fn source_option() -> Arg {
  Arg::new("source")
    .short('s')
    .long("source")
    .action(clap::ArgAction::Append)
    .value_parser(ValueParser::new(validate_source))
    .help("A list of quoted glob patterns for package.json files to read from")
}

fn filter_option() -> Arg {
  Arg::new("filter")
    .long("filter")
    .action(clap::ArgAction::Set)
    .value_parser(ValueParser::new(validate_filter))
    .help("Only include dependencies whose name matches this regex")
}

fn log_levels_option() -> Arg {
  Arg::new("log-levels")
    .long("log-levels")
    .value_delimiter(',')
    .value_parser(["off", "error", "warn", "info", "debug"])
    .default_values(["error", "warn", "info"])
    .help("Control how detailed log output should be")
}

fn no_color_option() -> Arg {
  Arg::new("no-color")
    .long("no-color")
    .action(clap::ArgAction::SetTrue)
    .help("Disable colored output")
}

fn create() -> Command {
  Command::new(crate_name!())
    .about(crate_description!())
    .version(crate_version!())
    .subcommand(
      Command::new("lint")
        .about("Lint command")
        .arg(filter_option())
        .arg(format_option())
        .arg(log_levels_option())
        .arg(no_color_option())
        .arg(source_option())
        .arg(versions_option()),
    )
    .subcommand(
      Command::new("fix")
        .about("Fix command")
        .arg(filter_option())
        .arg(format_option())
        .arg(log_levels_option())
        .arg(no_color_option())
        .arg(source_option())
        .arg(versions_option()),
    )
}

fn validate_filter(value: &str) -> Result<String, String> {
  Regex::new(value)
    // keep the value if it is a valid regex, we will parse it again later
    .map(|_| value.to_string())
    .map_err(|_| "not a valid Regex".to_string())
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
