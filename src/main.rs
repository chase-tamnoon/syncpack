#![allow(dead_code)]
#![allow(unused_variables)]

use env_logger::Builder;
use log::{Level, LevelFilter};
use std::{collections::BTreeMap, io::Write};

use crate::{
  cli::Subcommand, dependency::InstancesById, effects_fix::FixEffects, effects_lint::LintEffects,
  fix::fix, lint::lint, packages::get_packages,
};

mod cli;
mod config;
mod dependency;
mod dependency_type;
mod effects;
mod effects_fix;
mod effects_lint;
mod fix;
mod format;
mod group_selector;
mod instance;
mod json_file;
mod lint;
mod package_json;
mod packages;
mod semver_group;
mod specifier;
mod version_group;

fn main() -> () {
  init_logger();

  let cwd = std::env::current_dir().unwrap();
  let cli = cli::parse_input();
  let rcfile = config::get(&cwd);
  let packages = get_packages(&cwd, &cli.options, &rcfile);

  let semver_groups = rcfile.get_semver_groups();
  let mut version_groups = rcfile.get_version_groups(&packages.all_names);
  let mut instances_by_id: InstancesById = BTreeMap::new();

  // @TODO add some debug!("{}", logs);

  packages.get_all_instances(&rcfile, |instance| {
    // assign every instance to the first group it matches
    let semver_group = semver_groups
      .iter()
      .find(|semver_group| semver_group.selector.can_add(&instance));
    version_groups
      .iter_mut()
      .find(|version_group| version_group.selector.can_add(&instance))
      .unwrap()
      .add_instance(&instance, semver_group);
    // move instance to the lookup
    instances_by_id.insert(instance.id.clone(), instance);
  });

  match cli.command_name {
    Subcommand::Fix => {
      // everything is mutated and written when fixing
      let mut packages = packages;
      fix(
        &cwd,
        &cli,
        &rcfile,
        &mut packages,
        &mut instances_by_id,
        &mut version_groups,
        &FixEffects {},
      )
    }
    Subcommand::Lint => {
      // packages are mutated when linting formatting, but not written to disk
      let mut packages = packages;
      lint(
        &cwd,
        &cli,
        &rcfile,
        &mut packages,
        &mut instances_by_id,
        &mut version_groups,
        &LintEffects {},
      )
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
