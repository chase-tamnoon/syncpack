#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;
use std::{env::current_dir, process};

use crate::{
  cli::{Cli, Subcommand},
  config::Config,
  effects_fix::FixEffects,
  effects_lint::LintEffects,
  packages::Packages,
  visit_packages::visit_packages,
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
mod format;
mod group_selector;
mod instance;
mod package_json;
mod packages;
mod semver_group;
mod specifier;
mod version_group;
mod visit_packages;

fn main() {
  init_logger();

  let cwd = current_dir().unwrap();
  let cli = Cli::parse();
  let config = Config::from_cli(cwd, cli);
  let packages = Packages::from_config(&config);

  match config.cli.command_name {
    Subcommand::Fix => {
      let mut effects = FixEffects::new(&config);
      visit_packages(&config, packages, &mut effects);
      if !effects.is_valid {
        process::exit(1);
      }
    }
    Subcommand::Lint => {
      let mut effects = LintEffects::new(&config);
      visit_packages(&config, packages, &mut effects);
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
