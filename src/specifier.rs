use std::cmp::Ordering;

use semver::Semver;
use simple_semver::SimpleSemver;

use crate::specifier::non_semver::NonSemver;

pub mod any_specifier;
pub mod non_semver;
pub mod parser;
pub mod regexes;
pub mod semver;
pub mod semver_range;
pub mod simple_semver;

#[derive(Clone, Debug, Hash)]
pub enum Specifier {
  Semver(Semver),
  NonSemver(NonSemver),
  None,
}

impl Specifier {
  pub fn new(specifier: &String) -> Self {
    let str = parser::sanitise(specifier);
    let string = str.to_string();
    if parser::is_exact(str) {
      Self::Semver(Semver::Simple(SimpleSemver::Exact(string)))
    } else if parser::is_latest(str) {
      Self::Semver(Semver::Simple(SimpleSemver::Latest(string)))
    } else if parser::is_major(str) {
      Self::Semver(Semver::Simple(SimpleSemver::Major(string)))
    } else if parser::is_minor(str) {
      Self::Semver(Semver::Simple(SimpleSemver::Minor(string)))
    } else if parser::is_range(str) {
      Self::Semver(Semver::Simple(SimpleSemver::Range(string)))
    } else if parser::is_range_minor(str) {
      Self::Semver(Semver::Simple(SimpleSemver::RangeMinor(string)))
    } else if parser::is_complex_range(str) {
      Self::Semver(Semver::Complex(string))
    } else if parser::is_alias(str) {
      Self::NonSemver(NonSemver::Alias(string))
    } else if parser::is_file(str) {
      Self::NonSemver(NonSemver::File(string))
    } else if parser::is_git(str) {
      Self::NonSemver(NonSemver::Git(string))
    } else if parser::is_tag(str) {
      Self::NonSemver(NonSemver::Tag(string))
    } else if parser::is_url(str) {
      Self::NonSemver(NonSemver::Url(string))
    } else if parser::is_workspace_protocol(str) {
      Self::NonSemver(NonSemver::WorkspaceProtocol(string))
    } else {
      Self::NonSemver(NonSemver::Unsupported(string))
    }
  }

  /// Get the `specifier_type` name as used in config files.
  pub fn get_config_identifier(&self) -> String {
    match self {
      Self::Semver(Semver::Simple(SimpleSemver::Exact(_))) => "exact",
      Self::Semver(Semver::Simple(SimpleSemver::Latest(_))) => "latest",
      Self::Semver(Semver::Simple(SimpleSemver::Major(_))) => "major",
      Self::Semver(Semver::Simple(SimpleSemver::Minor(_))) => "minor",
      Self::Semver(Semver::Simple(SimpleSemver::Range(_))) => "range",
      Self::Semver(Semver::Simple(SimpleSemver::RangeMinor(_))) => "range-minor",
      Self::Semver(Semver::Complex(_)) => "range-complex",
      Self::NonSemver(NonSemver::Alias(_)) => "alias",
      Self::NonSemver(NonSemver::File(_)) => "file",
      Self::NonSemver(NonSemver::Git(_)) => "git",
      Self::NonSemver(NonSemver::Tag(_)) => "tag",
      Self::NonSemver(NonSemver::Url(_)) => "url",
      Self::NonSemver(NonSemver::WorkspaceProtocol(_)) => "workspace-protocol",
      Self::NonSemver(NonSemver::Unsupported(_)) => "unsupported",
      Self::None => "missing",
    }
    .to_string()
  }

  pub fn unwrap(&self) -> String {
    match self {
      Self::Semver(Semver::Simple(SimpleSemver::Exact(string))) => string,
      Self::Semver(Semver::Simple(SimpleSemver::Latest(string))) => string,
      Self::Semver(Semver::Simple(SimpleSemver::Major(string))) => string,
      Self::Semver(Semver::Simple(SimpleSemver::Minor(string))) => string,
      Self::Semver(Semver::Simple(SimpleSemver::Range(string))) => string,
      Self::Semver(Semver::Simple(SimpleSemver::RangeMinor(string))) => string,
      Self::Semver(Semver::Complex(string)) => string,
      Self::NonSemver(NonSemver::Alias(string)) => string,
      Self::NonSemver(NonSemver::File(string)) => string,
      Self::NonSemver(NonSemver::Git(string)) => string,
      Self::NonSemver(NonSemver::Tag(string)) => string,
      Self::NonSemver(NonSemver::Url(string)) => string,
      Self::NonSemver(NonSemver::WorkspaceProtocol(string)) => string,
      Self::NonSemver(NonSemver::Unsupported(string)) => string,
      Self::None => {
        panic!("Cannot unwrap a Specifier::None");
      }
    }
    .clone()
  }

  pub fn is_simple_semver(&self) -> bool {
    matches!(self, Specifier::Semver(Semver::Simple(_)))
  }

  pub fn get_simple_semver(&self) -> Option<SimpleSemver> {
    if let Specifier::Semver(Semver::Simple(simple_semver)) = self {
      Some(simple_semver.clone())
    } else {
      None
    }
  }
}

impl Ord for Specifier {
  fn cmp(&self, other: &Self) -> Ordering {
    if let Specifier::Semver(Semver::Simple(a)) = self {
      if let Specifier::Semver(Semver::Simple(b)) = other {
        return a.cmp(b);
      }
    }
    println!("@TODO: compare {:?} and {:?}", self, other);
    Ordering::Equal
  }
}

impl PartialOrd for Specifier {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for Specifier {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

impl Eq for Specifier {}

#[cfg(test)]
mod tests {
  use super::*;
  use std::cmp::Ordering;

  #[test]
  fn compare() {
    let cases: Vec<(&str, &str, Ordering)> = vec![
      /* normal versions */
      ("0.0.0", "0.0.1", Ordering::Less),
      ("0.0.0", "0.1.0", Ordering::Less),
      ("0.0.0", "1.0.0", Ordering::Less),
      ("0.0.0", "0.0.0", Ordering::Equal),
      ("0.0.1", "0.0.0", Ordering::Greater),
      ("0.1.0", "0.0.0", Ordering::Greater),
      ("1.0.0", "0.0.0", Ordering::Greater),
      /* range versions where versions differ */
      ("0.0.0", "~0.0.1", Ordering::Less),
      ("0.0.0", "~0.1.0", Ordering::Less),
      ("0.0.0", "~1.0.0", Ordering::Less),
      ("0.0.1", "~0.0.0", Ordering::Greater),
      ("0.1.0", "~0.0.0", Ordering::Greater),
      ("1.0.0", "~0.0.0", Ordering::Greater),
      /* range greediness applies only when versions are equal */
      ("0.0.0", "~0.0.0", Ordering::Less),
      ("0.0.0", "~0.0", Ordering::Less),
      ("0.0", "~0.0", Ordering::Less),
      ("0", "~0.0", Ordering::Less),
      ("0.0.0", "^0.0.0", Ordering::Less),
      ("0.0.0", "^0.0", Ordering::Less),
      ("0.0", "^0.0", Ordering::Less),
      ("0", "^0.0", Ordering::Less),
      ("0.0.0", ">0.0.0", Ordering::Less),
      ("0.0.0", ">=0.0.0", Ordering::Less),
      ("0.0.0", "<=0.0.0", Ordering::Greater),
      ("0.0.0", "<0.0.0", Ordering::Greater),
      ("0.0.0", "*", Ordering::Less),
      ("^0.0.0", "*", Ordering::Less),
      ("~0.0.0", "*", Ordering::Less),
      (">0.0.0", "*", Ordering::Less),
      (">=0.0.0", "*", Ordering::Less),
      ("<=0.0.0", "*", Ordering::Less),
      ("<0.0.0", "*", Ordering::Less),
      /* stable should be older than tagged */
      ("0.0.0", "0.0.0-alpha", Ordering::Less),
      /* equal tags should not affect comparison */
      ("0.0.0-alpha", "0.0.0-alpha", Ordering::Equal),
      ("0.0.0-alpha", "0.1.0-alpha", Ordering::Less),
      ("0.0.0-alpha", "1.0.0-alpha", Ordering::Less),
      ("0.0.0-alpha", "0.0.0-alpha", Ordering::Equal),
      ("0.0.1-alpha", "0.0.0-alpha", Ordering::Greater),
      ("0.1.0-alpha", "0.0.0-alpha", Ordering::Greater),
      ("1.0.0-alpha", "0.0.0-alpha", Ordering::Greater),
      /* preleases should matter when version is equal */
      ("0.0.0-rc.0.0.0", "0.0.0-rc.0.0.0", Ordering::Equal),
      ("0.0.0-rc.0.0.0", "0.0.0-rc.0.1.0", Ordering::Less),
      ("0.0.0-rc.0.0.0", "0.0.0-rc.1.0.0", Ordering::Less),
      ("0.0.0-rc.0.0.0", "0.0.0-rc.0.0.0", Ordering::Equal),
      ("0.0.0-rc.0.0.1", "0.0.0-rc.0.0.0", Ordering::Greater),
      ("0.0.0-rc.0.1.0", "0.0.0-rc.0.0.0", Ordering::Greater),
      ("0.0.0-rc.1.0.0", "0.0.0-rc.0.0.0", Ordering::Greater),
      /* preleases should not matter when version is greater */
      ("0.1.0-rc.0.0.0", "0.0.0-rc.0.1.0", Ordering::Greater),
      /* range greediness is the same on prereleases */
      ("0.0.0-rc.0", "~0.0.1-rc.0", Ordering::Less),
      ("0.0.0-rc.0", "~0.1.0-rc.0", Ordering::Less),
      ("0.0.0-rc.0", "~1.0.0-rc.0", Ordering::Less),
      ("0.0.0-rc.0", "~0.0.0-rc.0", Ordering::Less),
      ("0.0.1-rc.0", "~0.0.0-rc.0", Ordering::Greater),
      ("0.1.0-rc.0", "~0.0.0-rc.0", Ordering::Greater),
      ("1.0.0-rc.0", "~0.0.0-rc.0", Ordering::Greater),
    ];
    for (str_a, str_b, expected) in cases {
      let a = Specifier::new(&str_a.to_string());
      let b = Specifier::new(&str_b.to_string());
      let ordering = a.cmp(&b);
      assert_eq!(ordering, expected, "{str_a} should {expected:?} {str_b}");
    }
  }
}
