#![allow(dead_code)]
#![allow(unused_variables)]

use colored::*;
use std::io;

mod cli;
mod config;
mod context;
mod dependency_type;
mod format;
mod group_selector;
mod instance;
mod package_json;
mod semver_group;
mod semver_ranges;
mod specifier;
mod version_group;
mod versions;

fn main() -> io::Result<()> {
  let cwd = std::env::current_dir()?;
  let mut ctx = context::Ctx::new(&cwd)?;

  // - [x] find all package.json files
  // - [x] get enabled dependency types
  // - [x] create semver groups
  // - [x] create version groups
  // - [x] get all instances
  //   - [ ] parse version specifiers
  //   - [ ] set read only initial specifier on instance
  //   - [ ] set latest specifier on instance
  // - [ ] assign instances to semver groups
  // - [ ] assign instances to version groups

  let enabled_dependency_types = config::Rcfile::get_enabled_dependency_types(&ctx.rcfile);
  let semver_groups = semver_group::SemverGroup::from_rcfile(&ctx.rcfile);
  let version_groups = version_group::VersionGroup::from_rcfile(&ctx.rcfile);
  let all_instances: Vec<instance::Instance> = ctx
    .packages
    .iter()
    .flat_map(|package| package.get_instances(&enabled_dependency_types))
    .collect();

  println!("{}", "ctx.rcfile".yellow());
  println!("{:#?}", ctx.rcfile);
  println!("{}", "strategies".yellow());
  println!("{:#?}", enabled_dependency_types);
  println!("{}", "semver_groups".yellow());
  println!("{:#?}", semver_groups);
  println!("{}", "version_groups".yellow());
  println!("{:#?}", version_groups);
  println!("{}", "all_instances".yellow());
  println!("{:#?}", all_instances);

  match cli::create().get_matches().subcommand() {
    Some(("lint", matches)) => {
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        println!("{}", "Formatting".yellow());
        format::lint_all(&mut ctx);
        println!("@TODO: log whether formatting is valid or not");
      }
      if enabled_steps.ranges {
        println!("{}", "Semver Ranges".yellow());
        semver_ranges::lint_all(&mut ctx);
        println!("@TODO: log whether semver ranges match or not");
      }
      if enabled_steps.versions {
        println!("{}", "Versions".yellow());
        versions::lint_all(&mut ctx);
        println!("@TODO: log whether version mismatches are valid or not");
      }
      Ok(())
    }
    Some(("fix", matches)) => {
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        println!("{}", "Formatting".yellow());
        format::fix_all(&mut ctx);
        println!("@TODO: log whether formatting was fixed or not");
      }
      if enabled_steps.ranges {
        println!("{}", "Semver Ranges".yellow());
        semver_ranges::fix_all(&mut ctx);
        println!("@TODO: log whether semver range mismatches were fixed or not");
      }
      if enabled_steps.versions {
        println!("{}", "Versions".yellow());
        versions::fix_all(&mut ctx);
        println!("@TODO: log whether version mismatches were fixed or not");
      }
      Ok(())
    }
    _ => Err(create_error("unrecognized subcommand")),
  }
}

fn create_error(message: &str) -> io::Error {
  io::Error::new(io::ErrorKind::Other, message)
}
