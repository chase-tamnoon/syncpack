use super::{non_semver::NonSemver, semver::Semver, Specifier};

#[derive(Clone, Debug)]
pub enum SpecifierTree {
  Semver(Semver),
  NonSemver(NonSemver),
  None,
}

impl SpecifierTree {
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Exact(_) | Specifier::Latest(_) | Specifier::Major(_) | Specifier::Minor(_) | Specifier::Range(_) | Specifier::RangeComplex(_) | Specifier::RangeMinor(_) => SpecifierTree::Semver(Semver::new(specifier)),
      Specifier::Alias(_) | Specifier::File(_) | Specifier::Git(_) | Specifier::Tag(_) | Specifier::Unsupported(_) | Specifier::Url(_) | Specifier::WorkspaceProtocol(_) => SpecifierTree::NonSemver(NonSemver::new(specifier)),
      Specifier::None => SpecifierTree::None,
    }
  }

  pub fn is_simple_semver(&self) -> bool {
    matches!(self, SpecifierTree::Semver(Semver::Simple(_)))
  }
}
