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
    let semver_groups = config.rcfile.get_semver_groups();
    let mut version_groups = config.rcfile.get_version_groups(&packages.all_names);
    let mut instances_by_id: InstancesById = BTreeMap::new();

    // @TODO add some debug!("{}", logs);

    packages.get_all_instances(config, |mut instance| {
      version_groups
        .iter_mut()
        .find(|vgroup| vgroup.selector.can_add(&instance))
        .inspect(|vgroup| {
          instance.prefer_range = semver_groups
            .iter()
            .find(|sgroup| sgroup.selector.can_add(&instance))
            .and_then(|sgroup| sgroup.range.clone());
        })
        .map(|vgroup| {
          let dependency = vgroup.get_or_create_dependency(&instance);

          dependency.all.push(instance.id.clone());
          if instance.is_local {
            dependency.local_instance_id = Some(instance.id.clone());
          }

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
