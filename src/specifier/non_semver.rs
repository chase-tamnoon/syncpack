use super::Specifier;

#[derive(Clone, Debug)]
pub enum NonSemver {
  /// eg. `npm:1.2.3`
  Alias(String),
  /// eg. `file:./path/to/package`
  File(String),
  /// eg. `git://github.com/user/repo.git`
  Git(String),
  /// eg. `alpha`
  Tag(String),
  /// eg. `{wh}at[the]fuu`
  Unsupported(String),
  /// eg. `https://example.com`
  Url(String),
  /// eg. `workspace:*`
  WorkspaceProtocol(String),
}

impl NonSemver {
  pub fn new(specifier: &Specifier) -> Self {
    match specifier {
      Specifier::Alias(s) => NonSemver::Alias(s.clone()),
      Specifier::File(s) => NonSemver::File(s.clone()),
      Specifier::Git(s) => NonSemver::Git(s.clone()),
      Specifier::Tag(s) => NonSemver::Tag(s.clone()),
      Specifier::Unsupported(s) => NonSemver::Unsupported(s.clone()),
      Specifier::Url(s) => NonSemver::Url(s.clone()),
      Specifier::WorkspaceProtocol(s) => NonSemver::WorkspaceProtocol(s.clone()),
      _ => panic!("{specifier:?} is not NonSemver"),
    }
  }
}
