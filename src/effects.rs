use crate::context::Context;

pub mod fix;
pub mod lint;

/// Side effects in Syncpack commands are handled by structs which implement
/// this trait. Multiple commands such as `lint`, `fix`, and `json` all depend
/// on the same core logic, but have different side effects.
///
/// This trait allows the core logic to be reused across all commands, while the
/// side effects are handled by the command-specific structs which implement
/// this trait.
pub trait Effects {
  fn run(ctx: Context) -> Context;
}
