#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use std::{env::current_dir, process};

use crate::{
  cli::{Cli, Subcommand},
  config::Config,
  effects::{fix::FixEffects, lint::LintEffects},
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
      let mut effects = FixEffects::new(&config, &packages);
      visit_packages(&config, &packages, &mut effects);
      if !effects.is_valid {
        process::exit(1);
      }
    }
    Subcommand::Lint => {
      let mut effects = LintEffects::new(&config, &packages);
      visit_packages(&config, &packages, &mut effects);
      if !effects.is_valid {
        process::exit(1);
      }
    }
  };
}
