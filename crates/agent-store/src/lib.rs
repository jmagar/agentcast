//! Persistence boundaries for AgentCast.
//!
//! Store owns persistence traits, migrations, and concrete local storage
//! implementations. Runtime code should depend on store traits instead of
//! scattering database calls through orchestration modules.

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-store"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_boundary_label() {
        assert_eq!(crate_boundary(), "agent-store");
    }
}
