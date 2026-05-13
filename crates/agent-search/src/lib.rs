//! Search and ranking for AgentCast launcher actions.
//!
//! Search may begin as simple filtering in gateway/runtime, but this crate is
//! the ownership boundary for ranking, indexing, aliases, recency signals, and
//! query normalization once the behavior grows.

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-search"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_boundary_label() {
        assert_eq!(crate_boundary(), "agent-search");
    }
}
