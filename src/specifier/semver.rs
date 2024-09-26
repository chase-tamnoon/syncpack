use super::{
  orderable::{IsOrderable, Orderable},
  parser,
  simple_semver::SimpleSemver,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Semver {
  Simple(SimpleSemver),
  Complex(String),
}

impl Semver {
  pub fn new(specifier: &String) -> Self {
    let str = parser::sanitise(specifier);
    let string = str.to_string();
    if parser::is_exact(str) {
      Self::Simple(SimpleSemver::Exact(string))
    } else if parser::is_latest(str) {
      Self::Simple(SimpleSemver::Latest(string))
    } else if parser::is_major(str) {
      Self::Simple(SimpleSemver::Major(string))
    } else if parser::is_minor(str) {
      Self::Simple(SimpleSemver::Minor(string))
    } else if parser::is_range(str) {
      Self::Simple(SimpleSemver::Range(string))
    } else if parser::is_range_major(str) {
      Self::Simple(SimpleSemver::RangeMajor(string))
    } else if parser::is_range_minor(str) {
      Self::Simple(SimpleSemver::RangeMinor(string))
    } else if parser::is_complex_range(str) {
      Self::Complex(string)
    } else {
      panic!("{specifier:?} is not Semver");
    }
  }
}

impl IsOrderable for Semver {
  fn get_orderable(&self) -> Orderable {
    match self {
      Self::Simple(simple_semver) => simple_semver.get_orderable(),
      Self::Complex(_) => Orderable::new(),
    }
  }
}
