use std::rc::Rc;

use crate::{config::Config, instance::Instance, packages::Packages, version_group::VersionGroup};

pub struct Context {
  /// Every instance in the project
  pub instances: Vec<Rc<Instance>>,
  /// All version groups, their dependencies, and their instances
  pub version_groups: Vec<VersionGroup>,
}

impl Context {
  pub fn create(config: &Config, packages: &Packages) -> Self {
    let mut instances = vec![];
    let semver_groups = config.rcfile.get_semver_groups();
    let version_groups = config.rcfile.get_version_groups(packages);

    packages.get_all_instances(config, |instance| {
      let instance = Rc::new(instance);
      instances.push(Rc::clone(&instance));
      if let Some(semver_group) = semver_groups.iter().find(|semver_group| semver_group.selector.can_add(&instance)) {
        instance.set_semver_group(semver_group);
      }
      if let Some(version_group) = version_groups
        .iter()
        .find(|version_group| version_group.selector.can_add(&instance))
      {
        version_group.add_instance(instance);
      }
    });

    Self { instances, version_groups }
  }
}
