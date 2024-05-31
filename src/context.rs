use std::collections::BTreeMap;

use crate::{
  cli::CliOptions, config::Rcfile, dependency::InstancesById, instance::Instance,
  packages::Packages, version_group::VersionGroup,
};

pub struct Context {
  pub version_groups: Vec<VersionGroup>,
  pub instances_by_id: BTreeMap<String, Instance>,
}

pub fn get_context(cli_options: &CliOptions, rcfile: &Rcfile, packages: &Packages) -> Context {
  let semver_groups = rcfile.get_semver_groups();
  let mut version_groups = rcfile.get_version_groups(&packages.all_names);
  let mut instances_by_id: InstancesById = BTreeMap::new();

  // @TODO add some debug!("{}", logs);

  packages.get_all_instances(cli_options, rcfile, |instance| {
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

  Context {
    version_groups,
    instances_by_id,
  }
}
