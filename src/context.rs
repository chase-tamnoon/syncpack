use std::collections::BTreeMap;

use crate::{
  config::Config,
  instance::{Instance, InstanceId},
  packages::Packages,
  semver_group::SemverGroup,
  version_group::VersionGroup,
};

/// The location which owns all instances
pub type InstancesById = BTreeMap<InstanceId, Instance>;

pub struct Context {
  pub instances_by_id: InstancesById,
  pub semver_groups: Vec<SemverGroup>,
  pub version_groups: Vec<VersionGroup>,
}

impl Context {
  pub fn create(config: &Config, packages: &Packages) -> Self {
    let semver_groups = config.rcfile.get_semver_groups();
    let mut version_groups = config.rcfile.get_version_groups(&packages.all_names);
    let mut instances_by_id: InstancesById = BTreeMap::new();

    packages.get_all_instances(&config, |mut instance| {
      version_groups
        .iter_mut()
        .find(|vgroup| vgroup.selector.can_add(&instance))
        .inspect(|vgroup| {
          semver_groups.iter().find(|sgroup| sgroup.selector.can_add(&instance)).inspect(|sgroup| {
            instance.apply_semver_group(&sgroup);
          });
        })
        .map(|vgroup| {
          let dependency = vgroup.get_or_create_dependency(&instance);
          // let the dependency briefly own the instance
          let instance = dependency.add_instance(instance);
          // finally move the instance to the lookup
          instances_by_id.insert(instance.id.clone(), instance);
        });
    });

    Self {
      instances_by_id,
      semver_groups,
      version_groups,
    }
  }
}
