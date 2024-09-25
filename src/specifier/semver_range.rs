use std::cmp::Ordering;

#[derive(Clone, Debug, Hash)]
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

  /// Rank according to its greediness
  pub fn get_score(&self) -> u8 {
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
    self.get_score().cmp(&other.get_score())
  }
}

impl PartialOrd for SemverRange {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for SemverRange {
  fn eq(&self, other: &Self) -> bool {
    self.get_score() == other.get_score()
  }
}

impl Eq for SemverRange {}

#[cfg(test)]
mod tests {
  use std::cmp::Ordering;

  use super::*;

  fn to_strings(specifiers: Vec<&str>) -> Vec<String> {
    specifiers.iter().map(|s| s.to_string()).collect()
  }

  #[test]
  fn compare_ranges() {
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
}
