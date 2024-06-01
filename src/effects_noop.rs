use crate::{
  config::Config,
  dependency::Dependency,
  effects::{Effects, InstanceEvent},
  group_selector::GroupSelector,
  package_json::PackageJson,
};

/// An empty implementation of all of the side effects that Syncpack can
/// perform, for the purposes of unit testing. Methods can be overridden to
/// assert that they were called correctly.
pub struct NoopEffects {}

impl Effects for NoopEffects {
  fn on_no_packages(&self, config: &Config) {}
  fn on_begin_format(&self) {}
  fn on_skip_ranges_and_versions(&self) {}
  fn on_begin_ranges_and_versions(&self) {}
  fn on_begin_ranges_only(&self) {}
  fn on_begin_versions_only(&self) {}
  fn on_complete(&self, is_valid: bool) {}
  fn on_formatted_packages(&self, valid_packages: &Vec<&PackageJson>, config: &Config) {}
  fn on_unformatted_packages(&self, invalid_packages: &Vec<&PackageJson>, config: &Config) {}
  fn on_group(&self, selector: &GroupSelector) {}
  fn on_ignored_dependency(&self, dependency: &Dependency) {}
  fn on_banned_dependency(&self, dependency: &Dependency) {}
  fn on_valid_pinned_dependency(&self, dependency: &Dependency) {}
  fn on_invalid_pinned_dependency(&self, dependency: &Dependency) {}
  fn on_valid_same_range_dependency(&self, dependency: &Dependency) {}
  fn on_invalid_same_range_dependency(&self, dependency: &Dependency) {}
  fn on_valid_snap_to_dependency(&self, dependency: &Dependency) {}
  fn on_invalid_snap_to_dependency(&self, dependency: &Dependency) {}
  fn on_valid_standard_dependency(&self, dependency: &Dependency) {}
  fn on_invalid_standard_dependency(&self, dependency: &Dependency) {}
  fn on_banned_instance(&self, event: &mut InstanceEvent) {}
  fn on_pinned_version_mismatch(&self, event: &mut InstanceEvent) {}
  fn on_same_range_mismatch(&self, event: &mut InstanceEvent) {}
  fn on_snap_to_mismatch(&self, event: &mut InstanceEvent) {}
  fn on_local_version_mismatch(&self, event: &mut InstanceEvent) {}
  fn on_unsupported_mismatch(&self, event: &mut InstanceEvent) {}
  fn on_lowest_version_mismatch(&self, event: &mut InstanceEvent) {}
  fn on_highest_version_mismatch(&self, event: &mut InstanceEvent) {}
}
