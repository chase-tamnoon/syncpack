use serde::Deserialize;

use crate::group_selector;

pub struct BannedVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub is_banned: bool,
}

pub struct IgnoredVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub is_ignored: bool,
}

pub struct PinnedVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub pin_version: String,
}

pub struct SameRangeVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub policy: String,
}

pub struct SnappedToVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub snap_to: Vec<String>,
}

pub struct StandardVersionGroup {
  pub selector: group_selector::GroupSelector,
  pub prefer_version: String,
}

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
  pub fn create(&self, index: usize) -> VersionGroup {
    if let Some(true) = self.is_banned {
      return VersionGroup::Banned(BannedVersionGroup {
        selector: create_selector(self, index),
        is_banned: true,
      });
    }
    if let Some(true) = self.is_ignored {
      return VersionGroup::Ignored(IgnoredVersionGroup {
        selector: create_selector(self, index),
        is_ignored: true,
      });
    }
    if let Some(pin_version) = &self.pin_version {
      return VersionGroup::Pinned(PinnedVersionGroup {
        selector: create_selector(self, index),
        pin_version: pin_version.clone(),
      });
    }
    if let Some(policy) = &self.policy {
      return VersionGroup::SameRange(SameRangeVersionGroup {
        selector: create_selector(self, index),
        policy: policy.clone(),
      });
    }
    if let Some(snap_to) = &self.snap_to {
      return VersionGroup::SnappedTo(SnappedToVersionGroup {
        selector: create_selector(self, index),
        snap_to: snap_to.clone(),
      });
    }
    if let Some(prefer_version) = &self.prefer_version {
      return VersionGroup::Standard(StandardVersionGroup {
        selector: create_selector(self, index),
        prefer_version: prefer_version.clone(),
      });
    }
    panic!("Invalid version group");
  }
}

fn create_selector(group: &AnyVersionGroup, index: usize) -> group_selector::GroupSelector {
  group_selector::GroupSelector {
    dependencies: group.dependencies.clone(),
    dependency_types: group.dependency_types.clone(),
    label: group.label.clone(),
    index,
    packages: group.packages.clone(),
    specifier_types: group.specifier_types.clone(),
  }
}
