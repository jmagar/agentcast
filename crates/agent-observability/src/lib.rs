//! Observability primitives for AgentCast.
//!
//! This crate is intentionally small until implementation begins. It will own
//! tracing setup, audit-event shapes, log redaction, and metrics-facing helpers
//! that need to be shared across CLI, API, runtime, and gateway surfaces.

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-observability"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_boundary_label() {
        assert_eq!(crate_boundary(), "agent-observability");
    }
}
