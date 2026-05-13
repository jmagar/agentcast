//! UI-facing DTOs shared by AgentCast clients.
//!
//! This crate owns stable API/view DTOs for web, desktop, and external clients
//! without putting frontend implementation logic into Rust domain crates.

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-ui-contracts"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_boundary_label() {
        assert_eq!(crate_boundary(), "agent-ui-contracts");
    }
}
