//! JSON Schema normalization and validation for AgentCast.
//!
//! Schema owns MCP tool input-schema normalization, CLI argument coercion
//! rules, validation helpers, and future form metadata generation.

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-schema"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_boundary_label() {
        assert_eq!(crate_boundary(), "agent-schema");
    }
}
