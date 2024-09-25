use std::cmp::Ordering;

use log::debug;
use node_semver::Version;

use super::semver_range::SemverRange;

pub trait IsOrderable: std::fmt::Debug {
  fn get_orderable(&self) -> Orderable {
    debug!("Default Orderable used for {:?}", self);
    Orderable::new()
  }
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub(crate) struct Orderable {
  pub range: SemverRange,
  pub version: Version,
}

impl Orderable {
  pub fn new() -> Self {
    Self {
      range: SemverRange::Lt,
      version: Version {
        major: 0,
        minor: 0,
        patch: 0,
        build: vec![],
        pre_release: vec![],
      },
    }
  }
}

impl Ord for Orderable {
  fn cmp(&self, other: &Self) -> Ordering {
    // major
    match self.version.major.cmp(&other.version.major) {
      Ordering::Greater => Ordering::Greater,
      Ordering::Less => Ordering::Less,
      // minor
      Ordering::Equal => match self.version.minor.cmp(&other.version.minor) {
        Ordering::Greater => Ordering::Greater,
        Ordering::Less => Ordering::Less,
        // patch
        Ordering::Equal => match self.version.patch.cmp(&other.version.patch) {
          Ordering::Greater => Ordering::Greater,
          Ordering::Less => Ordering::Less,
          // build
          Ordering::Equal => match self.version.build.cmp(&other.version.build) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            // pre_release
            Ordering::Equal => match self.version.pre_release.cmp(&other.version.pre_release) {
              Ordering::Greater => Ordering::Greater,
              Ordering::Less => Ordering::Less,
              Ordering::Equal => self.range.cmp(&other.range),
            },
          },
        },
      },
    }
  }
}

impl PartialOrd for Orderable {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for Orderable {}
