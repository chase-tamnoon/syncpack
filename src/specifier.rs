use any_specifier::AnySpecifier;
use semver::Semver;

use crate::specifier::non_semver::NonSemver;

pub mod any_specifier;
pub mod non_semver;
pub mod parser;
pub mod regexes;
pub mod semver;
pub mod semver_range;
pub mod simple_semver;

#[derive(Clone, Debug)]
pub enum Specifier {
  Semver(Semver),
  NonSemver(NonSemver),
  None,
}

impl Specifier {
  pub fn new(specifier: &AnySpecifier) -> Self {
    match specifier {
      AnySpecifier::Exact(_) | AnySpecifier::Latest(_) | AnySpecifier::Major(_) | AnySpecifier::Minor(_) | AnySpecifier::Range(_) | AnySpecifier::RangeComplex(_) | AnySpecifier::RangeMinor(_) => Specifier::Semver(Semver::new(specifier)),
      AnySpecifier::Alias(_) | AnySpecifier::File(_) | AnySpecifier::Git(_) | AnySpecifier::Tag(_) | AnySpecifier::Unsupported(_) | AnySpecifier::Url(_) | AnySpecifier::WorkspaceProtocol(_) => {
        Specifier::NonSemver(NonSemver::new(specifier))
      }
      AnySpecifier::None => Specifier::None,
    }
  }

  pub fn is_simple_semver(&self) -> bool {
    matches!(self, Specifier::Semver(Semver::Simple(_)))
  }
}
