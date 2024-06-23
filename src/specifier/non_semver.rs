use super::AnySpecifier;

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
  pub fn new(specifier: &AnySpecifier) -> Self {
    match specifier {
      AnySpecifier::Alias(s) => NonSemver::Alias(s.clone()),
      AnySpecifier::File(s) => NonSemver::File(s.clone()),
      AnySpecifier::Git(s) => NonSemver::Git(s.clone()),
      AnySpecifier::Tag(s) => NonSemver::Tag(s.clone()),
      AnySpecifier::Unsupported(s) => NonSemver::Unsupported(s.clone()),
      AnySpecifier::Url(s) => NonSemver::Url(s.clone()),
      AnySpecifier::WorkspaceProtocol(s) => NonSemver::WorkspaceProtocol(s.clone()),
      _ => panic!("{specifier:?} is not NonSemver"),
    }
  }
}
