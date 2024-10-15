#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use std::{env::current_dir, process};

use crate::{
  cli::{Cli, Subcommand},
  config::Config,
  effects::{fix, lint},
  packages::Packages,
  visit_packages::visit_packages,
};

#[cfg(test)]
#[path = "test/test.rs"]
mod test;

mod cli;
mod config;
mod context;
mod dependency;
mod dependency_type;
mod effects;
mod format;
mod group_selector;
mod instance;
mod instance_state;
mod logger;
mod package_json;
mod packages;
mod semver_group;
mod specifier;
mod version_group;
mod visit_packages;

fn main() {
  logger::init();

  let cwd = current_dir().unwrap();
  let cli = Cli::parse();
  let config = Config::from_cli(cwd, cli);
  let packages = Packages::from_config(&config);

  match config.cli.command_name {
    Subcommand::Fix => {
      let ctx = visit_packages(config, packages);
      let ctx = fix::run(ctx);
      process::exit(ctx.exit_code);
    }
    Subcommand::Lint => {
      let ctx = visit_packages(config, packages);
      let ctx = lint::run(ctx);
      process::exit(ctx.exit_code);
    }
  };
}
