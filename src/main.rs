#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use {
  crate::{
    cli::{Cli, Subcommand},
    config::Config,
    effects::{fix, lint},
    packages::Packages,
    visit_packages::visit_packages,
  },
  log::error,
  std::env::current_dir,
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
  let cli = Cli::parse();

  logger::init(&cli.options);

  let cwd = current_dir().unwrap();
  let config = Config::from_cli(cwd, cli);
  let packages = Packages::from_config(&config);

  if packages.all.is_empty() {
    error!("No package.json files were found");
    std::process::exit(1);
  }

  match config.cli.command_name {
    Subcommand::Fix => {
      let ctx = visit_packages(config, packages);
      let ctx = fix::run(ctx);
      ctx.exit_program();
    }
    Subcommand::Lint => {
      let ctx = visit_packages(config, packages);
      let ctx = lint::run(ctx);
      ctx.exit_program();
    }
  };
}
