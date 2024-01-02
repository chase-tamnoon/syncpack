use serde::Deserialize;

use crate::groups::selector;

pub struct BannedVersionGroup {
  pub selector: selector::GroupSelector,
  pub is_banned: bool,
}

pub struct IgnoredVersionGroup {
  pub selector: selector::GroupSelector,
  pub is_ignored: bool,
}

pub struct PinnedVersionGroup {
  pub selector: selector::GroupSelector,
  pub pin_version: String,
}

pub struct SameRangeVersionGroup {
  pub selector: selector::GroupSelector,
  pub policy: String,
}

pub struct SnappedToVersionGroup {
  pub selector: selector::GroupSelector,
  pub snap_to: Vec<String>,
}

pub struct StandardVersionGroup {
  pub selector: selector::GroupSelector,
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
