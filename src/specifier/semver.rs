use super::{simple_semver::SimpleSemver, Specifier};

#[derive(Clone, Debug)]
pub enum Semver {
  Simple(SimpleSemver),
  Complex(String),
}

impl Semver {
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Exact(_) | Specifier::Latest(_) | Specifier::Major(_) | Specifier::Minor(_) | Specifier::Range(_) | Specifier::RangeMinor(_) => Semver::Simple(SimpleSemver::new(specifier)),
      Specifier::RangeComplex(s) => Semver::Complex(s.clone()),
      _ => panic!("{specifier:?} is not Semver"),
    }
  }
}
