use serde::Deserialize;

use crate::group_selector;

#[derive(Debug)]
pub struct BannedVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub is_banned: bool,
}

#[derive(Debug)]
pub struct IgnoredVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub is_ignored: bool,
}

#[derive(Debug)]
pub struct PinnedVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub pin_version: String,
}

#[derive(Debug)]
pub struct SameRangeVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub policy: String,
}

#[derive(Debug)]
pub struct SnappedToVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub snap_to: Vec<String>,
}

#[derive(Debug)]
pub struct StandardVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub prefer_version: String,
}

#[derive(Debug)]
pub enum VersionGroup {
  Banned(BannedVersionGroup),
  Ignored(IgnoredVersionGroup),
  Pinned(PinnedVersionGroup),
  SameRange(SameRangeVersionGroup),
  SnappedTo(SnappedToVersionGroup),
  Standard(StandardVersionGroup),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyVersionGroup {
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub dependency_types: Vec<String>,
  #[serde(default)]
  pub label: String,
  #[serde(default)]
  pub packages: Vec<String>,
  #[serde(default)]
  pub specifier_types: Vec<String>,
  //
  pub is_banned: Option<bool>,
  pub is_ignored: Option<bool>,
  pub pin_version: Option<String>,
  pub policy: Option<String>,
  pub snap_to: Option<Vec<String>>,
  pub prefer_version: Option<String>,
}

impl AnyVersionGroup {
  pub fn create(&self) -> VersionGroup {
    let selector = group_selector::GroupSelector {
      dependencies: self.dependencies.clone(),
      dependency_types: self.dependency_types.clone(),
      label: self.label.clone(),
      packages: self.packages.clone(),
      specifier_types: self.specifier_types.clone(),
    };

    if let Some(true) = self.is_banned {
      return VersionGroup::Banned(BannedVersionGroup {
        selector,
        is_banned: true,
      });
    }
    if let Some(true) = self.is_ignored {
      return VersionGroup::Ignored(IgnoredVersionGroup {
        selector,
        is_ignored: true,
      });
    }
    if let Some(pin_version) = &self.pin_version {
      return VersionGroup::Pinned(PinnedVersionGroup {
        selector,
        pin_version: pin_version.clone(),
      });
    }
    if let Some(policy) = &self.policy {
      return VersionGroup::SameRange(SameRangeVersionGroup {
        selector,
        policy: policy.clone(),
      });
    }
    if let Some(snap_to) = &self.snap_to {
      return VersionGroup::SnappedTo(SnappedToVersionGroup {
        selector,
        snap_to: snap_to.clone(),
      });
    }
    if let Some(prefer_version) = &self.prefer_version {
      return VersionGroup::Standard(StandardVersionGroup {
        selector,
        prefer_version: prefer_version.clone(),
      });
    }
    panic!("Invalid version group");
  }
}
