use std::collections::BTreeMap;

use crate::{
  config::Config, dependency::InstancesById, instance::Instance, packages::Packages,
  version_group::VersionGroup,
};

pub struct Context {
  pub instances_by_id: BTreeMap<String, Instance>,
  pub version_groups: Vec<VersionGroup>,
}

impl Context {
  pub fn create(config: &Config, packages: &Packages) -> Self {
    let semver_groups = config.rcfile.get_semver_groups();
    let mut version_groups = config.rcfile.get_version_groups(&packages.all_names);
    let mut instances_by_id: InstancesById = BTreeMap::new();

    // @TODO add some debug!("{}", logs);

    packages.get_all_instances(config, |instance| {
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

    Self {
      instances_by_id,
      version_groups,
    }
  }
}
