use std::{
  cmp::Ordering,
  hash::{Hash, Hasher},
};

#[derive(Clone, Debug)]
pub enum SemverRange {
  /// *
  Any,
  /// ^1.4.2
  Minor,
  /// 1.4.2
  Exact,
  /// >1.4.2
  Gt,
  /// >=1.4.2
  Gte,
  /// <1.4.2
  Lt,
  /// <=1.4.2
  Lte,
  /// ~1.4.2
  Patch,
}

impl SemverRange {
  /// Create a SemverRange if the given string is a valid range
  pub fn new(range: &str) -> Option<SemverRange> {
    match range {
      "*" => Some(SemverRange::Any),
      "^" => Some(SemverRange::Minor),
      "" => Some(SemverRange::Exact),
      ">" => Some(SemverRange::Gt),
      ">=" => Some(SemverRange::Gte),
      "<" => Some(SemverRange::Lt),
      "<=" => Some(SemverRange::Lte),
      "~" => Some(SemverRange::Patch),
      _ => None,
    }
  }

  /// Get the string representation of the range
  pub fn unwrap(&self) -> String {
    match self {
      SemverRange::Any => "*",
      SemverRange::Minor => "^",
      SemverRange::Exact => "",
      SemverRange::Gt => ">",
      SemverRange::Gte => ">=",
      SemverRange::Lt => "<",
      SemverRange::Lte => "<=",
      SemverRange::Patch => "~",
    }
    .to_string()
  }

  /// Get a numeric rank according to its greediness, for use in sorting
  pub fn get_greediness_ranking(&self) -> u8 {
    match self {
      SemverRange::Any => 7,
      SemverRange::Gt => 6,
      SemverRange::Gte => 5,
      SemverRange::Minor => 4,
      SemverRange::Patch => 3,
      SemverRange::Exact => 2,
      SemverRange::Lte => 1,
      SemverRange::Lt => 0,
    }
  }
}

impl Ord for SemverRange {
  fn cmp(&self, other: &Self) -> Ordering {
    self.get_greediness_ranking().cmp(&other.get_greediness_ranking())
  }
}

impl PartialOrd for SemverRange {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for SemverRange {
  fn eq(&self, other: &Self) -> bool {
    self.get_greediness_ranking() == other.get_greediness_ranking()
  }
}

impl Eq for SemverRange {}

impl Hash for SemverRange {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.get_greediness_ranking().hash(state);
  }
}

#[cfg(test)]
mod tests {
  use std::{cmp::Ordering, collections::HashMap};

  use super::*;

  #[test]
  fn creates_a_semver_range_from_a_string() {
    let cases: Vec<(&str, SemverRange)> = vec![
      ("*", SemverRange::Any),
      ("^", SemverRange::Minor),
      ("", SemverRange::Exact),
      (">", SemverRange::Gt),
      (">=", SemverRange::Gte),
      ("<", SemverRange::Lt),
      ("<=", SemverRange::Lte),
      ("~", SemverRange::Patch),
    ];
    for (input, expected) in cases {
      let parsed = SemverRange::new(input).unwrap();
      assert_eq!(parsed, expected, "'{input}' should be '{expected:?}'");
      assert_eq!(parsed.unwrap(), input, "'{parsed:?}' should unwrap to '{input}'");
    }
  }

  #[test]
  fn returns_none_for_unrecognised_ranges() {
    let parsed = SemverRange::new("wat");
    assert_eq!(parsed, None);
  }

  #[test]
  fn compares_ranges_according_to_their_greediness() {
    let cases: Vec<(&str, &str, Ordering)> = vec![
      ("", "", Ordering::Equal),
      ("", "<", Ordering::Greater),
      ("*", "*", Ordering::Equal),
      ("*", ">", Ordering::Greater),
      ("<", "<=", Ordering::Less),
      ("<=", "<", Ordering::Greater),
      (">", ">=", Ordering::Greater),
      (">=", ">", Ordering::Less),
      ("^", "", Ordering::Greater),
      ("^", "~", Ordering::Greater),
    ];
    for (a, b, expected) in cases {
      let parsed = SemverRange::new(a);
      let ordering = parsed.cmp(&SemverRange::new(b));
      assert_eq!(ordering, expected, "'{a}' should be {expected:?} '{b}'");
    }
  }

  #[test]
  fn sorts_ranges_according_to_their_greediness() {
    fn to_ranges(ranges: Vec<&str>) -> Vec<SemverRange> {
      ranges.iter().map(|r| SemverRange::new(r).unwrap()).collect()
    }
    let mut ranges = to_ranges(vec!["", "<", "*", ">", ">=", "<=", "^", "~"]);
    let expected = to_ranges(vec!["<", "<=", "", "~", "^", ">=", ">", "*"]);

    ranges.sort();
    assert_eq!(ranges, expected, "{ranges:?}, {expected:?}");
  }

  #[test]
  fn implements_hash() {
    let semver1 = SemverRange::new("^").unwrap();
    let semver2 = SemverRange::new("~").unwrap();
    let mut map = HashMap::new();

    map.insert(&semver1, "value1");
    map.insert(&semver2, "value2");

    // Retrieve values from the map to verify the hash implementation
    assert_eq!(map.get(&semver1), Some(&"value1"));
    assert_eq!(map.get(&semver2), Some(&"value2"));
  }
}
