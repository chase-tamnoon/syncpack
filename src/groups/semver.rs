use crate::selector;

pub struct DisabledSemverGroup {
  pub selector: selector::GroupSelector,
  pub is_disabled: bool,
}

pub struct IgnoredSemverGroup {
  pub selector: selector::GroupSelector,
  pub is_ignored: bool,
}

pub struct WithRangeSemverGroup {
  pub selector: selector::GroupSelector,
  pub range: String,
}

pub enum SemverGroup {
  Disabled(DisabledSemverGroup),
  Ignored(IgnoredSemverGroup),
  WithRange(WithRangeSemverGroup),
}
