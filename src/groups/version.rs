use crate::selector;

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
