use std::collections::BTreeMap;

use crate::{
  config::Config, dependency::InstancesById, instance::Instance, packages::Packages,
  semver_group::SemverGroup, version_group::VersionGroup,
};

pub struct Context {
  pub instances_by_id: BTreeMap<String, Instance>,
  pub semver_groups: Vec<SemverGroup>,
  pub version_groups: Vec<VersionGroup>,
}

impl Context {
  pub fn create(config: &Config, packages: &Packages) -> Self {
    let mut semver_groups = config.rcfile.get_semver_groups();
    let mut version_groups = config.rcfile.get_version_groups(&packages.all_names);
    let mut instances_by_id: InstancesById = BTreeMap::new();

    // @TODO add some debug!("{}", logs);

    packages.get_all_instances(config, |instance| {
      // assign every instance to the first group it matches
      semver_groups
        .iter_mut()
        .find(|semver_group| semver_group.selector.can_add(&instance))
        .unwrap()
        .add_instance(&instance);
      // assign every instance to the first group it matches
      version_groups
        .iter_mut()
        .find(|version_group| version_group.selector.can_add(&instance))
        .unwrap()
        .add_instance(&instance);
      // move instance to the lookup
      instances_by_id.insert(instance.id.clone(), instance);
    });

    Self {
      instances_by_id,
      semver_groups,
      version_groups,
    }
  }
}
