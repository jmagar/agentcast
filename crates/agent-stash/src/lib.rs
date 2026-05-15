//! Saved AgentCast artifacts and reusable context collection contracts.
//!
//! Stash owns portable metadata for user-owned prompts, action templates,
//! references, and history bundles. It does not own gateway invocation,
//! runtime process management, API routes, CLI rendering, or Lab service
//! workflows.

mod export;
mod meta;
mod path;
mod revision;

pub use export::{ExportManifest, ImportManifest};
pub use meta::{DriftStatus, StashItemKind, StashItemMeta};
pub use path::{SafeRelativePath, StashPathError, validate_relative_stash_path};
pub use revision::{RevisionId, StashRevision};

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-stash"
}
