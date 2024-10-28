use {
  clap::{builder::ValueParser, crate_description, crate_name, crate_version, Arg, ArgMatches, Command},
  color_print::cformat,
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

fn filter_option(command: &str) -> Arg {
  let operation = if command == "lint" { "display" } else { "fix" };
  Arg::new("filter")
    .long("filter")
    .long_help(cformat!(
      r#"Only <bold>{operation}</bold> dependencies whose <bold>name</bold> matches this <bold>RegEx</bold>

<bold><underline>Important:</underline></bold>
--filter only affects what syncpack will <bold>{operation}</bold>. syncpack will still
inspect and exit 1/0 based on every dependency in your project.

<bold><underline>Examples:</underline></bold>
<dim>An exact match for "react"</>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --filter '^react$'</>
<dim>Any name containing "react" anywhere within it</>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --filter 'react'</>"#
    ))
    .action(clap::ArgAction::Set)
    .value_parser(ValueParser::new(validate_filter))
}

fn log_levels_option(command: &str) -> Arg {
  Arg::new("log-levels")
    .long("log-levels")
    .long_help(cformat!(
      r#"Control how detailed the log output should be

<bold><underline>Examples:</underline></bold>
<dim>Turn off logging completely</dim>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --log-levels off</>
<dim>Only show verbose debugging logs</dim>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --log-levels debug</>
<dim>Show everything</dim>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --log-levels error,warn,info,debug</>"#
    ))
    .value_delimiter(',')
    .value_parser(["off", "error", "warn", "info", "debug"])
    .default_value("error,warn,info")
}

fn no_ansi_option(command: &str) -> Arg {
  Arg::new("no-ansi")
    .long("no-ansi")
    .long_help(cformat!(
      r#"Disable ANSI colored output and terminal hyperlinks

<bold><underline>Examples:</underline></bold>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --no-ansi</>"#
    ))
    .action(clap::ArgAction::SetTrue)
}

fn only_option(command: &str) -> Arg {
  Arg::new("only")
    .long("only")
    .long_help(cformat!(
      r#"Only inspect version mismatches, or formatting issues

<bold><underline>Examples:</underline></bold>
<dim>Only inspect version mismatches</dim>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --only mismatches</>
<dim>Only inspect formatting of package.json files</dim>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --only formatting</>"#
    ))
    .value_delimiter(',')
    .value_parser(["formatting", "mismatches"])
    .default_value("formatting,mismatches")
}

fn show_option(command: &str) -> Arg {
  Arg::new("show")
    .long("show")
    .long_help(cformat!(
      r#"Control what information is displayed in lint output

<bold><underline>Values:</underline></bold>
<yellow>ignored</>       Show instances and dependencies which syncpack is ignoring
<yellow>instances</>     Show every instance of every dependency
<yellow>local-hints</>   Show a hint alongside dependencies developed in this repo
<yellow>packages</>      Show formatting status of each package.json file
<yellow>status-codes</>  Show specifically how/why a dependency or instance is valid or invalid

<bold><underline>Examples:</underline></bold>
<dim>Show highest level of detail</dim>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --show ignored,instances,local-hints,packages,status-codes</>"#
    ))
    .value_delimiter(',')
    .value_parser(["ignored", "instances", "local-hints", "packages", "status-codes"])
    .default_value("instances,local-hints,packages,status-codes")
}

fn source_option(command: &str) -> Arg {
  Arg::new("source")
    .long("source")
    .long_help(cformat!(
      r#"A list of quoted glob patterns for package.json files to read from

<bold><underline>Examples:</underline></bold>
<dim>$</dim> <yellow><bold>syncpack {command}</bold> --source 'package.json' --source 'apps/*/package.json'</>

<bold><underline>Resolving Packages:</underline></bold>
Patterns are discovered in the following order, first one wins:

1. <yellow>--source</> CLI options
2. <yellow>.source</> property of your syncpack config file
3. <yellow>.workspaces.packages</> property of package.json (yarn)
4. <yellow>.workspaces</> property of package.json (npm and yarn)
5. <yellow>.packages</> property of pnpm-workspace.yaml
6. <yellow>.packages</> property of lerna.json
7. Default to <yellow>["package.json","packages/*/package.json"]</>"#
    ))
    .action(clap::ArgAction::Append)
    .value_parser(ValueParser::new(validate_source))
}

fn additional_help() -> String {
  cformat!(
    r#"<bold><underline>References:</underline></bold>
- Documentation: <blue><underline>https://jamiemason.github.io/syncpack</></>
- Learn glob patterns: <blue><underline>https://github.com/isaacs/node-glob#glob-primer</></>
- lerna.json: <blue><underline>https://github.com/lerna/lerna#lernajson</></>
- Yarn Workspaces: <blue><underline>https://yarnpkg.com/lang/en/docs/workspaces</></>
- Pnpm Workspaces: <blue><underline>https://pnpm.js.org/en/workspaces</></>"#
  )
}

fn create() -> Command {
  Command::new(crate_name!())
    .about(crate_description!())
    .version(crate_version!())
    .subcommand(
      Command::new("lint")
        .about("Find and list all version mismatches and package.json formatting issues")
        .after_long_help(additional_help())
        .arg(filter_option("lint"))
        .arg(log_levels_option("lint"))
        .arg(no_ansi_option("lint"))
        .arg(only_option("lint"))
        .arg(show_option("lint"))
        .arg(source_option("lint")),
    )
    .subcommand(
      Command::new("fix")
        .about("Fix all autofixable issues in affected package.json files")
        .after_long_help(additional_help())
        .arg(filter_option("fix"))
        .arg(log_levels_option("fix"))
        .arg(no_ansi_option("fix"))
        .arg(only_option("fix"))
        .arg(source_option("fix")),
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
  pub disable_ansi: bool,
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
      disable_ansi: matches.get_flag("no-ansi"),
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
