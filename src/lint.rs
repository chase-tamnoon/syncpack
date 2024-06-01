use crate::{
  config::Config,
  context::{get_context, Context},
  effects::Effects,
  format::{self, InMemoryFormattingStatus},
  packages::Packages,
};

pub fn lint<T: Effects>(config: &Config, packages: &mut Packages, effects: &T) -> () {
  let mut is_valid = true;
  let cli_options = &config.cli.options;
  let Context {
    version_groups,
    mut instances_by_id,
  } = get_context(&config, &packages);

  match (cli_options.ranges, cli_options.versions) {
    (true, true) => effects.on_begin_ranges_and_versions(),
    (true, false) => effects.on_begin_ranges_only(),
    (false, true) => effects.on_begin_versions_only(),
    (false, false) => effects.on_skip_ranges_and_versions(),
  };

  if cli_options.ranges || cli_options.versions {
    version_groups.iter().for_each(|group| {
      let group_is_valid = group.visit(&mut instances_by_id, effects, packages);
      if !group_is_valid {
        is_valid = false;
      }
    });
  }

  if cli_options.format {
    effects.on_begin_format();
    let InMemoryFormattingStatus {
      was_valid: valid,
      was_invalid: invalid,
    } = format::fix(&config, packages);
    effects.on_formatted_packages(&valid, &config);
    effects.on_unformatted_packages(&invalid, &config);
    if !invalid.is_empty() {
      is_valid = false;
    }
  }

  effects.on_complete(is_valid);
}
