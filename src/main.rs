#![allow(dead_code)]
#![allow(unused_variables)]

use effects_fix::FixEffects;
use effects_lint::LintEffects;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;
use std::{env::current_dir, process};

use crate::{
  cli::{Cli, Subcommand},
  config::Config,
  fix::fix,
  lint::lint,
  packages::Packages,
};

mod cli;
mod config;
mod context;
mod dependency;
mod dependency_type;
mod effects;
mod effects_fix;
mod effects_lint;
mod effects_mock;
mod expect;
mod fix;
mod format;
mod group_selector;
mod instance;
mod lint;
mod package_json;
mod packages;
mod semver_group;
mod semver_range;
mod specifier;
mod version_group;

fn main() -> () {
  init_logger();

  let cwd = current_dir().unwrap();
  let cli = Cli::parse();
  let config = Config::from_cli(cwd, cli);
  let packages = Packages::from_config(&config);

  match config.cli.command_name {
    Subcommand::Fix => {
      // everything is mutated and written when fixing
      let mut packages = packages;
      let mut effects = FixEffects::new(&config);
      fix(&config, &mut packages, &mut effects);
      if !effects.is_valid {
        process::exit(1);
      }
    }
    Subcommand::Lint => {
      // packages are mutated when linting formatting, but not written to disk
      let mut packages = packages;
      let mut effects = LintEffects::new(&config);
      lint(&config, &mut packages, &mut effects);
      if !effects.is_valid {
        process::exit(1);
      }
    }
  };
}

fn init_logger() {
  Builder::new()
    // @TODO expose cli and rcfile options for log level
    .filter_level(LevelFilter::Info)
    .format(|buf, record| {
      let level = record.level();
      if level == Level::Info {
        writeln!(buf, "{}", record.args())
      } else {
        // @TODO apply colours to log levels
        writeln!(buf, "[{level}] {}", record.args())
      }
    })
    .init();
}
