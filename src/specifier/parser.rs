use log::debug;

use super::{regexes, AnySpecifier};

/// Convert non-semver specifiers to semver when behaviour is identical
pub fn sanitise(specifier: &String) -> &str {
  let specifier = specifier.as_str();
  if specifier == "latest" || specifier == "x" {
    debug!("Sanitising specifier: {} → *", specifier);
    "*"
  } else {
    specifier
  }
}

/// Convert a raw string version specifier into a `Specifier` enum serving as a
/// branded type
pub fn parse(specifier: &String) -> AnySpecifier {
  let str = sanitise(specifier);
  let string = str.to_string();
  if is_exact(str) {
    AnySpecifier::Exact(string)
  } else if is_range(str) {
    AnySpecifier::Range(string)
  } else if is_latest(str) {
    AnySpecifier::Latest(string)
  } else if is_workspace_protocol(str) {
    AnySpecifier::WorkspaceProtocol(string)
  } else if is_alias(str) {
    AnySpecifier::Alias(string)
  } else if is_major(str) {
    AnySpecifier::Major(string)
  } else if is_minor(str) {
    AnySpecifier::Minor(string)
  } else if is_tag(str) {
    AnySpecifier::Tag(string)
  } else if is_git(str) {
    AnySpecifier::Git(string)
  } else if is_url(str) {
    AnySpecifier::Url(string)
  } else if is_range_minor(str) {
    AnySpecifier::RangeMinor(string)
  } else if is_file(str) {
    AnySpecifier::File(string)
  } else if is_complex_range(str) {
    AnySpecifier::RangeComplex(string)
  } else {
    AnySpecifier::Unsupported(string)
  }
}

fn is_simple_semver(str: &str) -> bool {
  is_exact(str) || is_latest(str) || is_major(str) || is_minor(str) || is_range(str) || is_range_minor(str)
}

fn is_exact(str: &str) -> bool {
  regexes::EXACT.is_match(str) || regexes::EXACT_TAG.is_match(str)
}

fn is_latest(str: &str) -> bool {
  str == "*" || str == "latest" || str == "x"
}

fn is_major(str: &str) -> bool {
  regexes::MAJOR.is_match(str)
}

fn is_minor(str: &str) -> bool {
  regexes::MINOR.is_match(str)
}

pub fn is_range(specifier: &str) -> bool {
  regexes::CARET.is_match(specifier)
    || regexes::CARET_TAG.is_match(specifier)
    || regexes::TILDE.is_match(specifier)
    || regexes::TILDE_TAG.is_match(specifier)
    || regexes::GT.is_match(specifier)
    || regexes::GT_TAG.is_match(specifier)
    || regexes::GTE.is_match(specifier)
    || regexes::GTE_TAG.is_match(specifier)
    || regexes::LT.is_match(specifier)
    || regexes::LT_TAG.is_match(specifier)
    || regexes::LTE.is_match(specifier)
    || regexes::LTE_TAG.is_match(specifier)
}

pub fn is_range_minor(specifier: &str) -> bool {
  regexes::CARET_MINOR.is_match(specifier)
    || regexes::TILDE_MINOR.is_match(specifier)
    || regexes::GT_MINOR.is_match(specifier)
    || regexes::GTE_MINOR.is_match(specifier)
    || regexes::LT_MINOR.is_match(specifier)
    || regexes::LTE_MINOR.is_match(specifier)
}

/// Is this a semver range containing multiple parts?
/// Such as OR (" || ") or AND (" ")
pub fn is_complex_range(specifier: &str) -> bool {
  regexes::OR_OPERATOR
    .split(specifier)
    .map(|str| str.trim())
    .filter(|str| str.len() > 0)
    .all(|or_condition| or_condition.split(" ").map(|str| str.trim()).filter(|str| str.len() > 0).all(|and_condition| is_simple_semver(and_condition)))
}

fn is_tag(str: &str) -> bool {
  regexes::TAG.is_match(str)
}

fn is_workspace_protocol(str: &str) -> bool {
  regexes::WORKSPACE_PROTOCOL.is_match(str)
}

fn is_alias(str: &str) -> bool {
  regexes::ALIAS.is_match(str)
}

fn is_git(str: &str) -> bool {
  regexes::GIT.is_match(str)
}

fn is_url(str: &str) -> bool {
  regexes::URL.is_match(str)
}

fn is_file(str: &str) -> bool {
  regexes::FILE.is_match(str)
}
