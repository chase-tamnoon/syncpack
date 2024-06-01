#![allow(dead_code)]
#![allow(unused_variables)]

use context::RunState;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;
use std::{env::current_dir, process};

use crate::{
  cli::{Cli, Subcommand},
  config::Config,
  effects_fix::fix_effects,
  effects_lint::lint_effects,
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
mod fix;
mod format;
mod group_selector;
mod instance;
mod lint;
mod package_json;
mod packages;
mod semver_group;
mod specifier;
mod version_group;

fn main() -> () {
  init_logger();

  let cwd = current_dir().unwrap();
  let cli = Cli::parse();
  let config = Config::from_cli(cwd, cli);
  let packages = Packages::from_config(&config);

  let mut state = RunState { is_valid: true };

  match config.cli.command_name {
    Subcommand::Fix => {
      // everything is mutated and written when fixing
      let mut packages = packages;
      fix(&config, &mut packages, fix_effects, &mut state);
    }
    Subcommand::Lint => {
      // packages are mutated when linting formatting, but not written to disk
      let mut packages = packages;
      lint(&config, &mut packages, lint_effects, &mut state);
    }
  };

  if !state.is_valid {
    process::exit(1);
  }
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
