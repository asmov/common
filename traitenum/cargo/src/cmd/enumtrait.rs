//! Handles all enumtrait commands (add, remove, etc.)

use crate::meta;

pub mod add;
pub mod remove;

pub const EXAMPLE_TRAIT_NAME: &'static str = "Example";

fn has_enumtrait(
    trait_name: &str,
    workspace: &meta::WorkspaceMeta,
    library: &meta::LibraryMeta
) -> anyhow::Result<bool> {
    todo!()
}