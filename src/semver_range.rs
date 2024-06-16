#[derive(Debug, Eq, PartialEq)]
pub enum SemverRange {
  /// *
  Any,
  /// ^1.4.2
  Caret,
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
  Tilde,
}

impl SemverRange {
  pub fn new(range: &String) -> Option<SemverRange> {
    match range.as_str() {
      "*" => Some(SemverRange::Any),
      "^" => Some(SemverRange::Caret),
      "" => Some(SemverRange::Exact),
      ">" => Some(SemverRange::Gt),
      ">=" => Some(SemverRange::Gte),
      "<" => Some(SemverRange::Lt),
      "<=" => Some(SemverRange::Lte),
      "~" => Some(SemverRange::Tilde),
      _ => None,
    }
  }

  pub fn unwrap(&self) -> String {
    match self {
      SemverRange::Any => "*",
      SemverRange::Caret => "^",
      SemverRange::Exact => "",
      SemverRange::Gt => ">",
      SemverRange::Gte => ">=",
      SemverRange::Lt => "<",
      SemverRange::Lte => "<=",
      SemverRange::Tilde => "~",
    }
    .to_string()
  }
}
