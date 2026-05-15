use agent_auth::{OAuthCredential, PendingOAuthState};
use std::collections::BTreeMap;
use thiserror::Error;

#[cfg(test)]
mod tests;

pub trait OAuthStore {
    fn put_pending_state(&mut self, state: PendingOAuthState) -> Result<(), StoreError>;
    fn consume_pending_state(
        &mut self,
        state: &str,
    ) -> Result<Option<PendingOAuthState>, StoreError>;
    fn put_credential(&mut self, credential: OAuthCredential) -> Result<(), StoreError>;
    fn credential(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthCredential>, StoreError>;
    fn clear_subject_upstream(
        &mut self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<(), StoreError>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryOAuthStore {
    pending_states: BTreeMap<String, PendingOAuthState>,
    credentials: BTreeMap<(String, String), OAuthCredential>,
}

impl OAuthStore for InMemoryOAuthStore {
    fn put_pending_state(&mut self, state: PendingOAuthState) -> Result<(), StoreError> {
        self.pending_states.insert(state.state.clone(), state);
        Ok(())
    }

    fn consume_pending_state(
        &mut self,
        state: &str,
    ) -> Result<Option<PendingOAuthState>, StoreError> {
        Ok(self.pending_states.remove(state))
    }

    fn put_credential(&mut self, credential: OAuthCredential) -> Result<(), StoreError> {
        self.credentials.insert(
            (credential.subject.clone(), credential.upstream_id.clone()),
            credential,
        );
        Ok(())
    }

    fn credential(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthCredential>, StoreError> {
        Ok(self
            .credentials
            .get(&(subject.to_string(), upstream_id.to_string()))
            .cloned())
    }

    fn clear_subject_upstream(
        &mut self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<(), StoreError> {
        self.credentials
            .remove(&(subject.to_string(), upstream_id.to_string()));
        self.pending_states
            .retain(|_, state| state.subject != subject || state.upstream_id != upstream_id);
        Ok(())
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StoreError {
    #[error("OAuth store operation failed")]
    OperationFailed,
}
