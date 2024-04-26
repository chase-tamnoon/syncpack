use log::debug;

use crate::instance::Instance;

#[derive(Debug)]
pub struct GroupSelector {
  /// Glob patterns to match against the installed dependency name.
  ///
  /// The keyword "$LOCAL" can also be used to match every locally-developed
  /// package used as a dependency.
  pub dependencies: Vec<String>,
  /// Named locations where dependencies should be found.
  ///
  /// Possible values:
  /// - "dev" or "!dev"
  /// - "peer" or "!peer"
  /// - "prod" or "!prod"
  /// - "local" or "!local"
  /// - "overrides" or "!overrides"
  /// - "pnpm_overrides" or "!pnpm_overrides"
  /// - "resolutions" or "!resolutions"
  pub dependency_types: Vec<String>,
  /// Optional label to describe the group.
  pub label: String,
  /// Glob patterns to match against the package name the dependency is located in.
  pub packages: Vec<String>,
  /// Types of version specifier the installed dependency should have.
  ///
  /// Possible values:
  /// - "alias" or "!alias"
  /// - "delete" or "!delete"
  /// - "exact" or "!exact"
  /// - "file" or "!file"
  /// - "hosted-git" or -!hosted-git"
  /// - "latest" or "!latest"
  /// - "range" or "!range"
  /// - "tag" or "!tag"
  /// - "unsupported" or "!unsupported"
  /// - "url" or "!url"
  /// - "workspace-protocol" or -!workspace-protocol"
  pub specifier_types: Vec<String>,
}

impl GroupSelector {
  pub fn can_add(&self, instance: &Instance) -> bool {
    debug!("@TODO implement GroupSelector::can_add");
    true
  }
}
