use std::rc::Rc;

use crate::{config::Config, packages::Packages, semver_group::SemverGroup, version_group::VersionGroup};

pub struct Context {
  pub semver_groups: Vec<SemverGroup>,
  pub version_groups: Vec<VersionGroup>,
}

impl Context {
  pub fn create(config: &Config, packages: &Packages) -> Self {
    let semver_groups = config.rcfile.get_semver_groups();
    let version_groups = config.rcfile.get_version_groups(&packages.all_names);

    packages.get_all_instances(config, |instance| {
      // first move the instance to the lookup
      let instance = Rc::new(instance);

      if let Some(vgroup) = version_groups.iter().find(|vgroup| vgroup.selector.can_add(&instance)) {
        if let Some(sgroup) = semver_groups.iter().find(|sgroup| sgroup.selector.can_add(&instance)) {
          instance.apply_semver_group(sgroup);
        }
        vgroup.add_instance(instance);
      }
    });

    Self {
      semver_groups,
      version_groups,
    }
  }
}
