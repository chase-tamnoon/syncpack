use super::{simple_semver::SimpleSemver, AnySpecifier};

#[derive(Clone, Debug)]
pub enum Semver {
  Simple(SimpleSemver),
  Complex(String),
}

impl Semver {
  pub fn new(specifier: &AnySpecifier) -> Self {
    match specifier {
      AnySpecifier::Exact(_)
      | AnySpecifier::Latest(_)
      | AnySpecifier::Major(_)
      | AnySpecifier::Minor(_)
      | AnySpecifier::Range(_)
      | AnySpecifier::RangeMinor(_) => Semver::Simple(SimpleSemver::new(specifier)),
      AnySpecifier::RangeComplex(s) => Semver::Complex(s.clone()),
      _ => panic!("{specifier:?} is not Semver"),
    }
  }
}
