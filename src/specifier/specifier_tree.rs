use super::{non_semver::NonSemver, semver::Semver, AnySpecifier};

#[derive(Clone, Debug)]
pub enum SpecifierTree {
  Semver(Semver),
  NonSemver(NonSemver),
  None,
}

impl SpecifierTree {
  pub fn new(specifier: &AnySpecifier) -> Self {
    match specifier {
      AnySpecifier::Exact(_) | AnySpecifier::Latest(_) | AnySpecifier::Major(_) | AnySpecifier::Minor(_) | AnySpecifier::Range(_) | AnySpecifier::RangeComplex(_) | AnySpecifier::RangeMinor(_) => {
        SpecifierTree::Semver(Semver::new(specifier))
      }
      AnySpecifier::Alias(_) | AnySpecifier::File(_) | AnySpecifier::Git(_) | AnySpecifier::Tag(_) | AnySpecifier::Unsupported(_) | AnySpecifier::Url(_) | AnySpecifier::WorkspaceProtocol(_) => {
        SpecifierTree::NonSemver(NonSemver::new(specifier))
      }
      AnySpecifier::None => SpecifierTree::None,
    }
  }

  pub fn is_simple_semver(&self) -> bool {
    matches!(self, SpecifierTree::Semver(Semver::Simple(_)))
  }
}
