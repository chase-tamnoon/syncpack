#![allow(dead_code)]
#![allow(unused_variables)]

use std::io;

extern crate glob;
extern crate serde;
extern crate serde_json;

mod cli;
mod config;
mod context;
mod dependencies;
mod file_paths;
mod format;
mod package_json;
mod semver_ranges;
mod versions;

fn main() -> io::Result<()> {
  let cwd = std::env::current_dir()?;
  let mut ctx = context::Ctx::new(&cwd)?;

  match cli::create().get_matches().subcommand() {
    Some(("lint", matches)) => {
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        format::lint_all(&mut ctx);
        println!("@TODO: log whether formatting is valid or not");
      }
      if enabled_steps.ranges {
        semver_ranges::lint_all(&mut ctx);
        println!("@TODO: log whether semver ranges match or not");
      }
      if enabled_steps.versions {
        versions::lint_all(&mut ctx);
        println!("@TODO: log whether version mismatches are valid or not");
      }
      Ok(())
    }
    Some(("fix", matches)) => {
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        format::fix_all(&mut ctx);
        println!("@TODO: log whether formatting was fixed or not");
      }
      if enabled_steps.ranges {
        semver_ranges::fix_all(&mut ctx);
        println!("@TODO: log whether semver range mismatches were fixed or not");
      }
      if enabled_steps.versions {
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
