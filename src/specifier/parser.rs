use log::debug;

use crate::specifier::{regexes, specifier_tree::SpecifierTree, Specifier};

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
pub fn parse(specifier: &String, is_recursive: bool) -> Specifier {
  let str = sanitise(specifier);
  let string = str.to_string();
  if regexes::REGEX_EXACT.is_match(str) || regexes::REGEX_EXACT_TAG.is_match(str) {
    Specifier::Exact(string)
  } else if is_range(str) {
    Specifier::Range(string)
  } else if str == "*" || str == "latest" || str == "x" {
    Specifier::Latest(string)
  } else if regexes::REGEX_WORKSPACE_PROTOCOL.is_match(str) {
    Specifier::WorkspaceProtocol(string)
  } else if regexes::REGEX_ALIAS.is_match(str) {
    Specifier::Alias(string)
  } else if regexes::REGEX_MAJOR.is_match(str) {
    Specifier::Major(string)
  } else if regexes::REGEX_MINOR.is_match(str) {
    Specifier::Minor(string)
  } else if regexes::REGEX_TAG.is_match(str) {
    Specifier::Tag(string)
  } else if regexes::REGEX_GIT.is_match(str) {
    Specifier::Git(string)
  } else if regexes::REGEX_URL.is_match(str) {
    Specifier::Url(string)
  } else if is_range_minor(str) {
    Specifier::RangeMinor(string)
  } else if regexes::REGEX_FILE.is_match(str) {
    Specifier::File(string)
  } else if !is_recursive && is_complex_range(str) {
    Specifier::RangeComplex(string)
  } else {
    Specifier::Unsupported(string)
  }
}

/// Is this a semver range containing multiple parts?
/// Such as OR (" || ") or AND (" ")
pub fn is_complex_range(specifier: &str) -> bool {
  regexes::REGEX_OR_OPERATOR.split(specifier).map(|str| str.trim()).filter(|str| str.len() > 0).all(|or_condition| {
    or_condition
      .split(" ")
      .map(|str| str.trim())
      .filter(|str| str.len() > 0)
      .map(|and_condition| parse(&and_condition.to_string(), true))
      .map(|specifier| SpecifierTree::new(&specifier))
      .all(|specifier_tree| specifier_tree.is_simple_semver())
  })
}

pub fn is_range(specifier: &str) -> bool {
  regexes::REGEX_CARET.is_match(specifier)
    || regexes::REGEX_CARET_TAG.is_match(specifier)
    || regexes::REGEX_TILDE.is_match(specifier)
    || regexes::REGEX_TILDE_TAG.is_match(specifier)
    || regexes::REGEX_GT.is_match(specifier)
    || regexes::REGEX_GT_TAG.is_match(specifier)
    || regexes::REGEX_GTE.is_match(specifier)
    || regexes::REGEX_GTE_TAG.is_match(specifier)
    || regexes::REGEX_LT.is_match(specifier)
    || regexes::REGEX_LT_TAG.is_match(specifier)
    || regexes::REGEX_LTE.is_match(specifier)
    || regexes::REGEX_LTE_TAG.is_match(specifier)
}

pub fn is_range_minor(specifier: &str) -> bool {
  regexes::REGEX_CARET_MINOR.is_match(specifier)
    || regexes::REGEX_TILDE_MINOR.is_match(specifier)
    || regexes::REGEX_GT_MINOR.is_match(specifier)
    || regexes::REGEX_GTE_MINOR.is_match(specifier)
    || regexes::REGEX_LT_MINOR.is_match(specifier)
    || regexes::REGEX_LTE_MINOR.is_match(specifier)
}
