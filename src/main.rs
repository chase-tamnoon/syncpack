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

fn main() -> io::Result<()> {
  match cli::create().get_matches().subcommand() {
    Some(("lint", matches)) => {
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        format_lint()?;
      }
      if enabled_steps.ranges {
        println!("lint ranges");
      }
      if enabled_steps.versions {
        println!("lint versions");
      }
      Ok(())
    }
    Some(("fix", matches)) => {
      let enabled_steps = cli::get_enabled_steps(matches);
      if enabled_steps.format {
        println!("fix format");
      }
      if enabled_steps.ranges {
        println!("fix ranges");
      }
      if enabled_steps.versions {
        println!("fix versions");
      }
      Ok(())
    }
    _ => Err(create_error("unrecognized subcommand")),
  }
}

fn create_error(message: &str) -> io::Error {
  io::Error::new(io::ErrorKind::Other, message)
}

fn format_lint() -> Result<(), io::Error> {
  let cwd = std::env::current_dir()?;
  let mut ctx = context::Ctx::new(&cwd)?;
  let rcfile = &ctx.rcfile;

  ctx.packages.into_iter().for_each(|mut package| {
    format::fix(&mut package, &rcfile);
    if package.has_changed() {
      ctx.is_invalid = true;
      println!("X {}", package.short_path);
    } else {
      println!("- {}", package.short_path);
    }
    package.pretty_print();
  });

  if ctx.is_invalid {
    println!("Invalid package.json files found. Please run `syncpack fix --format` to fix them.");
  }

  Ok(())
}
