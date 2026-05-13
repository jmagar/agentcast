//! Saved AgentCast artifacts and reusable context collections.
//!
//! Stash is post-v0. It will own saved invocation history, reusable context,
//! curated actions, and persisted artifacts once the MCP launcher loop is
//! stable.

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-stash"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_boundary_label() {
        assert_eq!(crate_boundary(), "agent-stash");
    }
}
