use std::{collections::BTreeMap, rc::Rc};

use crate::{
  config::Config,
  instance::{Instance, InstanceId},
  packages::Packages,
  semver_group::SemverGroup,
  version_group::VersionGroup,
};

/// The location which owns all instances
pub type InstancesById = BTreeMap<InstanceId, Rc<Instance>>;

pub struct Context {
  pub instances_by_id: InstancesById,
  pub semver_groups: Vec<SemverGroup>,
  pub version_groups: Vec<VersionGroup>,
}

impl Context {
  pub fn create(config: &Config, packages: &Packages) -> Self {
    let semver_groups = config.rcfile.get_semver_groups();
    let version_groups = config.rcfile.get_version_groups(&packages.all_names);
    let mut instances_by_id: InstancesById = BTreeMap::new();

    packages.get_all_instances(config, |instance| {
      // first move the instance to the lookup
      let instance = Rc::new(instance);
      instances_by_id.insert(instance.id.clone(), Rc::clone(&instance));

      if let Some(vgroup) = version_groups.iter().find(|vgroup| vgroup.selector.can_add(&instance)) {
        if let Some(sgroup) = semver_groups.iter().find(|sgroup| sgroup.selector.can_add(&instance)) {
          instance.apply_semver_group(sgroup);
        }
        vgroup.add_instance(instance);
      }
    });

    Self {
      instances_by_id,
      semver_groups,
      version_groups,
    }
  }
}
