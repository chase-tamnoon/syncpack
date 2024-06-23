use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  /// Any character used in a semver range
  pub static ref REGEX_RANGE_CHAR: Regex = Regex::new(r"[~><=*^]").unwrap();
  /// "1.2.3"
  pub static ref REGEX_EXACT: Regex = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
  /// "^1.2.3"
  pub static ref REGEX_CARET: Regex = Regex::new(r"^\^(\d+\.\d+\.\d+)$").unwrap();
  /// "~1.2.3"
  pub static ref REGEX_TILDE: Regex = Regex::new(r"^~(\d+\.\d+\.\d+)$").unwrap();
  /// ">1.2.3"
  pub static ref REGEX_GT: Regex = Regex::new(r"^>(\d+\.\d+\.\d+)$").unwrap();
  /// ">=1.2.3"
  pub static ref REGEX_GTE: Regex = Regex::new(r"^>=(\d+\.\d+\.\d+)$").unwrap();
  /// "<1.2.3"
  pub static ref REGEX_LT: Regex = Regex::new(r"^<(\d+\.\d+\.\d+)$").unwrap();
  /// "<=1.2.3"
  pub static ref REGEX_LTE: Regex = Regex::new(r"^<=(\d+\.\d+\.\d+)$").unwrap();
  /// "^1.2"
  pub static ref REGEX_CARET_MINOR: Regex = Regex::new(r"^\^(\d+\.\d+)$").unwrap();
  /// "~1.2"
  pub static ref REGEX_TILDE_MINOR: Regex = Regex::new(r"^~(\d+\.\d+)$").unwrap();
  /// ">1.2"
  pub static ref REGEX_GT_MINOR: Regex = Regex::new(r"^>(\d+\.\d+)$").unwrap();
  /// ">=1.2"
  pub static ref REGEX_GTE_MINOR: Regex = Regex::new(r"^>=(\d+\.\d+)$").unwrap();
  /// "<1.2"
  pub static ref REGEX_LT_MINOR: Regex = Regex::new(r"^<(\d+\.\d+)$").unwrap();
  /// "<=1.2"
  pub static ref REGEX_LTE_MINOR: Regex = Regex::new(r"^<=(\d+\.\d+)$").unwrap();
  /// "1"
  pub static ref REGEX_MAJOR: Regex = Regex::new(r"^(\d+)$").unwrap();
  /// "1.2"
  pub static ref REGEX_MINOR: Regex = Regex::new(r"^(\d+\.\d+)$").unwrap();
  /// "npm:"
  pub static ref REGEX_ALIAS: Regex = Regex::new(r"^npm:").unwrap();
  /// "file:"
  pub static ref REGEX_FILE: Regex = Regex::new(r"^file:").unwrap();
  /// "workspace:"
  pub static ref REGEX_WORKSPACE_PROTOCOL: Regex = Regex::new(r"^workspace:").unwrap();
  /// "https://"
  pub static ref REGEX_URL: Regex = Regex::new(r"^https?://").unwrap();
  /// "git://"
  pub static ref REGEX_GIT: Regex = Regex::new(r"^git(\+(ssh|https?))?://").unwrap();
  /// "alpha"
  pub static ref REGEX_TAG: Regex = Regex::new(r"^[a-zA-Z0-9-]+$").unwrap();
  /// a logical OR in a semver range
  pub static ref REGEX_OR_OPERATOR:Regex = Regex::new(r" ?\|\| ?").unwrap();
}
