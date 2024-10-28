use {
  clap::{builder::ValueParser, crate_description, crate_name, crate_version, Arg, ArgMatches, Command},
  itertools::Itertools,
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

fn filter_option() -> Arg {
  Arg::new("filter")
    .long("filter")
    .help("Only include dependencies whose name matches this regex")
    .action(clap::ArgAction::Set)
    .value_parser(ValueParser::new(validate_filter))
}

fn log_levels_option() -> Arg {
  Arg::new("log-levels")
    .long("log-levels")
    .help("Control how detailed log output should be")
    .value_delimiter(',')
    .value_parser(["off", "error", "warn", "info", "debug"])
    .default_values(["error", "warn", "info"])
}

fn no_color_option() -> Arg {
  Arg::new("no-color")
    .long("no-color")
    .help("Disable colored output")
    .action(clap::ArgAction::SetTrue)
}

fn only_option() -> Arg {
  Arg::new("only")
    .short('o')
    .long("only")
    .help("Only inspect version mismatches, or formatting issues")
    .value_delimiter(',')
    .value_parser(["formatting", "mismatches"])
    .default_values(["formatting", "mismatches"])
}

fn source_option() -> Arg {
  Arg::new("source")
    .short('s')
    .long("source")
    .help("A list of quoted glob patterns for package.json files to read from")
    .action(clap::ArgAction::Append)
    .value_parser(ValueParser::new(validate_source))
}

fn create() -> Command {
  Command::new(crate_name!())
    .about(crate_description!())
    .version(crate_version!())
    .subcommand(
      Command::new("lint")
        .about("Find and list all version mismatches and package.json formatting issues")
        .arg(filter_option())
        .arg(log_levels_option())
        .arg(no_color_option())
        .arg(only_option())
        .arg(source_option()),
    )
    .subcommand(
      Command::new("fix")
        .about("Fix all autofixable issues in affected package.json files")
        .arg(filter_option())
        .arg(log_levels_option())
        .arg(no_color_option())
        .arg(only_option())
        .arg(source_option()),
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
  /// `true` when `--format` is passed or if none of `--formatting`, `--ranges`
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
    let only = matches.get_many::<String>("only").unwrap().collect_vec();
    CliOptions {
      disable_color: matches.get_flag("no-color"),
      filter: matches.get_one::<String>("filter").map(|filter| Regex::new(filter).unwrap()),
      format: only.contains(&&"formatting".to_string()),
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
      versions: only.contains(&&"mismatches".to_string()),
    }
  }
}
