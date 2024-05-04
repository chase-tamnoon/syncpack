#[derive(Debug)]
pub enum BannedVersionGroupStatusCode {
  Banned,
}

#[derive(Debug)]
pub enum IgnoredVersionGroupStatusCode {
  Ignored,
}

#[derive(Debug)]
pub enum PinnedVersionGroupStatusCode {
  PinnedMismatch,
  Valid,
}

#[derive(Debug)]
pub enum SameRangeVersionGroupStatusCode {
  SameRangeMismatch,
  Valid,
}

#[derive(Debug)]
pub enum SnappedToVersionGroupStatusCode {
  MissingSnappedToMismatch,
  SnappedToMismatch,
  Valid,
}

#[derive(Debug)]
pub enum StandardVersionGroupStatusCode {
  HighestSemverMismatch,
  LocalPackageMismatch,
  LowestSemverMismatch,
  MissingLocalVersion,
  SemverRangeMismatch,
  UnsupportedMismatch,
  Valid,
}
