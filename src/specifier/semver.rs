use super::Exact;
use super::Latest;
use super::Major;
use super::Minor;
use super::Range;
use super::RangeComplex;
use super::RangeMinor;
use super::Specifier;

pub enum Semver {
  Simple(simple_semver::SimpleSemver),
  Complex(String),
}

impl Semver {
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Exact(_) | Specifier::Latest(_) | Specifier::Major(_) | Specifier::Minor(_) | Specifier::Range(_) | Specifier::RangeMinor(_) => Semver::Simple(simple_semver::SimpleSemver::new(specifier)),
      Specifier::RangeComplex(s) => Semver::Complex(s.clone()),
      _ => panic!("{specifier:?} is not Semver"),
    }
  }
}
