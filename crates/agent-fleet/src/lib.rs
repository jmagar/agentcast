//! Fleet and remote runtime coordination for AgentCast.
//!
//! Fleet is post-v0. It will own remote nodes, device/runtime enrollment,
//! remote health, and execution target coordination when those capabilities
//! are promoted beyond the local MCP launcher.

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-fleet"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_boundary_label() {
        assert_eq!(crate_boundary(), "agent-fleet");
    }
}
