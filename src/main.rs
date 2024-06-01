#![allow(dead_code)]
#![allow(unused_variables)]

use env_logger::Builder;
use log::{Level, LevelFilter};
use std::env::current_dir;
use std::io::Write;

use crate::{
  cli::{Cli, Subcommand},
  config::Config,
  effects_fix::FixEffects,
  effects_lint::LintEffects,
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
mod effects_noop;
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

  match config.cli.command_name {
    Subcommand::Fix => {
      // everything is mutated and written when fixing
      let mut packages = packages;
      fix(&config, &mut packages, &FixEffects {})
    }
    Subcommand::Lint => {
      // packages are mutated when linting formatting, but not written to disk
      let mut packages = packages;
      lint(&config, &mut packages, &LintEffects {})
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
