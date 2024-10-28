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
    .long_help("Only include dependencies whose name matches this regex")
    .action(clap::ArgAction::Set)
    .value_parser(ValueParser::new(validate_filter))
}

fn log_levels_option() -> Arg {
  Arg::new("log-levels")
    .long("log-levels")
    .long_help("Control how detailed log output should be")
    .value_delimiter(',')
    .value_parser(["off", "error", "warn", "info", "debug"])
    .default_value("error,warn,info")
}

fn no_color_option() -> Arg {
  Arg::new("no-color")
    .long("no-color")
    .long_help("Disable colored output")
    .action(clap::ArgAction::SetTrue)
}

fn only_option() -> Arg {
  Arg::new("only")
    .long("only")
    .long_help("Only inspect version mismatches, or formatting issues")
    .value_delimiter(',')
    .value_parser(["formatting", "mismatches"])
    .default_value("formatting,mismatches")
}

fn show_option() -> Arg {
  Arg::new("show")
    .long("show")
    .long_help("Control what information is displayed in lint output")
    .value_delimiter(',')
    .value_parser(["ignored", "instances", "local-hints", "packages", "status-codes"])
    .default_value("local-hints,status-codes")
}

fn source_option() -> Arg {
  Arg::new("source")
    .long("source")
    .long_help("A list of quoted glob patterns for package.json files to read from")
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
        .arg(show_option())
        .arg(source_option()),
    )
    .subcommand(
      Command::new("fix")
        .about("Fix all autofixable issues in affected package.json files")
        .arg(filter_option())
        .arg(log_levels_option())
        .arg(no_color_option())
        .arg(only_option())
        .arg(show_option())
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
  pub dependency_name_regex: Option<Regex>,
  pub disable_color: bool,
  pub inspect_formatting: bool,
  pub inspect_mismatches: bool,
  pub log_levels: Vec<LevelFilter>,
  /// Whether to output ignored dependencies regardless
  pub show_ignored: bool,
  /// Whether to list every affected instance of a dependency when listing
  /// version or semver range mismatches
  pub show_instances: bool,
  /// Whether to indicate that a dependency is a package developed locally
  pub show_local_hints: bool,
  /// Whether to list every affected package.json file when listing formatting
  /// mismatches
  pub show_packages: bool,
  /// Whether to show the name of the status code for each dependency and
  /// instance, such as `HighestSemverMismatch`
  pub show_status_codes: bool,
  pub source_patterns: Vec<String>,
}

impl CliOptions {
  /// Create a new `CliOptions` from CLI arguments provided by the user
  pub fn from_arg_matches(matches: &ArgMatches) -> CliOptions {
    let show = matches.get_many::<String>("show").unwrap().collect_vec();
    let only = matches.get_many::<String>("only").unwrap().collect_vec();
    CliOptions {
      dependency_name_regex: matches.get_one::<String>("filter").map(|filter| Regex::new(filter).unwrap()),
      disable_color: matches.get_flag("no-color"),
      inspect_formatting: only.contains(&&"formatting".to_string()),
      inspect_mismatches: only.contains(&&"mismatches".to_string()),
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
      show_ignored: show.contains(&&"ignored".to_string()),
      show_instances: show.contains(&&"instances".to_string()),
      show_local_hints: show.contains(&&"local-hints".to_string()),
      show_packages: show.contains(&&"packages".to_string()),
      show_status_codes: show.contains(&&"status-codes".to_string()),
      source_patterns: matches
        .get_many::<String>("source")
        .unwrap_or_default()
        .map(|source| source.to_owned())
        .collect::<Vec<_>>(),
    }
  }
}
