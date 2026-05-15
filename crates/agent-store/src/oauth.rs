use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, OsRng, Payload, rand_core::RngCore},
};
use agent_auth::{OAuthClientRegistration, OAuthCredential, PendingOAuthState};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeMap;
use std::{fmt, path::Path};
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
    fn put_client_registration(
        &mut self,
        registration: OAuthClientRegistration,
    ) -> Result<(), StoreError>;
    fn client_registration(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthClientRegistration>, StoreError>;
    fn clear_subject_upstream(
        &mut self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<(), StoreError>;
    fn clear_upstream(&mut self, upstream_id: &str) -> Result<(), StoreError>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryOAuthStore {
    pending_states: BTreeMap<String, PendingOAuthState>,
    credentials: BTreeMap<(String, String), OAuthCredential>,
    client_registrations: BTreeMap<(String, String), OAuthClientRegistration>,
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

    fn put_client_registration(
        &mut self,
        registration: OAuthClientRegistration,
    ) -> Result<(), StoreError> {
        self.client_registrations.insert(
            (
                registration.subject.clone(),
                registration.upstream_id.clone(),
            ),
            registration,
        );
        Ok(())
    }

    fn client_registration(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthClientRegistration>, StoreError> {
        Ok(self
            .client_registrations
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
        self.client_registrations
            .remove(&(subject.to_string(), upstream_id.to_string()));
        Ok(())
    }

    fn clear_upstream(&mut self, upstream_id: &str) -> Result<(), StoreError> {
        self.credentials
            .retain(|(_, existing_upstream_id), _| existing_upstream_id != upstream_id);
        self.pending_states
            .retain(|_, state| state.upstream_id != upstream_id);
        self.client_registrations
            .retain(|(_, existing_upstream_id), _| existing_upstream_id != upstream_id);
        Ok(())
    }
}

impl<T> OAuthStore for Box<T>
where
    T: OAuthStore + ?Sized,
{
    fn put_pending_state(&mut self, state: PendingOAuthState) -> Result<(), StoreError> {
        (**self).put_pending_state(state)
    }

    fn consume_pending_state(
        &mut self,
        state: &str,
    ) -> Result<Option<PendingOAuthState>, StoreError> {
        (**self).consume_pending_state(state)
    }

    fn put_credential(&mut self, credential: OAuthCredential) -> Result<(), StoreError> {
        (**self).put_credential(credential)
    }

    fn credential(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthCredential>, StoreError> {
        (**self).credential(subject, upstream_id)
    }

    fn put_client_registration(
        &mut self,
        registration: OAuthClientRegistration,
    ) -> Result<(), StoreError> {
        (**self).put_client_registration(registration)
    }

    fn client_registration(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthClientRegistration>, StoreError> {
        (**self).client_registration(subject, upstream_id)
    }

    fn clear_subject_upstream(
        &mut self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<(), StoreError> {
        (**self).clear_subject_upstream(subject, upstream_id)
    }

    fn clear_upstream(&mut self, upstream_id: &str) -> Result<(), StoreError> {
        (**self).clear_upstream(upstream_id)
    }
}

pub struct SqliteOAuthStore {
    conn: Connection,
    cipher: CredentialCipher,
}

impl fmt::Debug for SqliteOAuthStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SqliteOAuthStore")
            .field("conn", &"<sqlite>")
            .field("cipher", &"<credential-cipher>")
            .finish()
    }
}

impl SqliteOAuthStore {
    pub fn open(path: impl AsRef<Path>, key: [u8; 32]) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|_| StoreError::OperationFailed)?;
        }
        let conn = Connection::open(path).map_err(|_| StoreError::OperationFailed)?;
        restrict_permissions(path)?;
        Self::from_connection(conn, key)
    }

    pub fn in_memory(key: [u8; 32]) -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory().map_err(|_| StoreError::OperationFailed)?;
        Self::from_connection(conn, key)
    }

    fn from_connection(conn: Connection, key: [u8; 32]) -> Result<Self, StoreError> {
        let store = Self {
            conn,
            cipher: CredentialCipher::new(key)?,
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS oauth_pending_states (
                    state TEXT PRIMARY KEY NOT NULL,
                    subject TEXT NOT NULL,
                    upstream_id TEXT NOT NULL,
                    expires_at_unix INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS oauth_credentials (
                    subject TEXT NOT NULL,
                    upstream_id TEXT NOT NULL,
                    access_nonce BLOB NOT NULL,
                    access_ciphertext BLOB NOT NULL,
                    refresh_nonce BLOB,
                    refresh_ciphertext BLOB,
                    scopes_json TEXT NOT NULL,
                    expires_at_unix INTEGER NOT NULL,
                    refresh_failed INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (subject, upstream_id)
                );

                CREATE TABLE IF NOT EXISTS oauth_client_registrations (
                    subject TEXT NOT NULL,
                    upstream_id TEXT NOT NULL,
                    client_id TEXT NOT NULL,
                    client_secret_nonce BLOB,
                    client_secret_ciphertext BLOB,
                    client_id_issued_at_unix INTEGER,
                    client_secret_expires_at_unix INTEGER,
                    PRIMARY KEY (subject, upstream_id)
                );
                "#,
            )
            .map_err(|_| StoreError::OperationFailed)
    }

    #[cfg(test)]
    pub(crate) fn raw_access_token_ciphertext(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        self.conn
            .query_row(
                "SELECT access_ciphertext FROM oauth_credentials WHERE subject = ?1 AND upstream_id = ?2",
                params![subject, upstream_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| StoreError::OperationFailed)
    }

    #[cfg(test)]
    pub(crate) fn raw_access_token_nonce(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        self.conn
            .query_row(
                "SELECT access_nonce FROM oauth_credentials WHERE subject = ?1 AND upstream_id = ?2",
                params![subject, upstream_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| StoreError::OperationFailed)
    }
}

impl OAuthStore for SqliteOAuthStore {
    fn put_pending_state(&mut self, state: PendingOAuthState) -> Result<(), StoreError> {
        self.conn
            .execute(
                r#"
                INSERT INTO oauth_pending_states (state, subject, upstream_id, expires_at_unix)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(state) DO UPDATE SET
                    subject = excluded.subject,
                    upstream_id = excluded.upstream_id,
                    expires_at_unix = excluded.expires_at_unix
                "#,
                params![
                    state.state,
                    state.subject,
                    state.upstream_id,
                    state.expires_at_unix as i64
                ],
            )
            .map_err(|_| StoreError::OperationFailed)?;
        Ok(())
    }

    fn consume_pending_state(
        &mut self,
        state: &str,
    ) -> Result<Option<PendingOAuthState>, StoreError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|_| StoreError::OperationFailed)?;
        let pending = tx
            .query_row(
                r#"
                SELECT state, subject, upstream_id, expires_at_unix
                FROM oauth_pending_states
                WHERE state = ?1
                "#,
                params![state],
                |row| {
                    let expires_at_unix: i64 = row.get(3)?;
                    Ok(PendingOAuthState {
                        state: row.get(0)?,
                        subject: row.get(1)?,
                        upstream_id: row.get(2)?,
                        expires_at_unix: expires_at_unix as u64,
                    })
                },
            )
            .optional()
            .map_err(|_| StoreError::OperationFailed)?;

        if pending.is_some() {
            tx.execute(
                "DELETE FROM oauth_pending_states WHERE state = ?1",
                params![state],
            )
            .map_err(|_| StoreError::OperationFailed)?;
        }
        tx.commit().map_err(|_| StoreError::OperationFailed)?;
        Ok(pending)
    }

    fn put_credential(&mut self, credential: OAuthCredential) -> Result<(), StoreError> {
        let encrypted_access = self.cipher.encrypt(
            &credential.subject,
            &credential.upstream_id,
            "access_token",
            credential.access_token.as_bytes(),
        )?;
        let encrypted_refresh = credential
            .refresh_token
            .as_deref()
            .map(|token| {
                self.cipher.encrypt(
                    &credential.subject,
                    &credential.upstream_id,
                    "refresh_token",
                    token.as_bytes(),
                )
            })
            .transpose()?;
        let scopes_json =
            serde_json::to_string(&credential.scopes).map_err(|_| StoreError::OperationFailed)?;
        let (refresh_nonce, refresh_ciphertext) = encrypted_refresh
            .map(|encrypted| (Some(encrypted.nonce), Some(encrypted.ciphertext)))
            .unwrap_or((None, None));

        self.conn
            .execute(
                r#"
                INSERT INTO oauth_credentials (
                    subject, upstream_id, access_nonce, access_ciphertext,
                    refresh_nonce, refresh_ciphertext, scopes_json, expires_at_unix, refresh_failed
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(subject, upstream_id) DO UPDATE SET
                    access_nonce = excluded.access_nonce,
                    access_ciphertext = excluded.access_ciphertext,
                    refresh_nonce = excluded.refresh_nonce,
                    refresh_ciphertext = excluded.refresh_ciphertext,
                    scopes_json = excluded.scopes_json,
                    expires_at_unix = excluded.expires_at_unix,
                    refresh_failed = excluded.refresh_failed
                "#,
                params![
                    credential.subject,
                    credential.upstream_id,
                    encrypted_access.nonce,
                    encrypted_access.ciphertext,
                    refresh_nonce,
                    refresh_ciphertext,
                    scopes_json,
                    credential.expires_at_unix as i64,
                    i64::from(credential.refresh_failed),
                ],
            )
            .map_err(|_| StoreError::OperationFailed)?;
        Ok(())
    }

    fn credential(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthCredential>, StoreError> {
        self.conn
            .query_row(
                r#"
                SELECT access_nonce, access_ciphertext, refresh_nonce, refresh_ciphertext,
                       scopes_json, expires_at_unix, refresh_failed
                FROM oauth_credentials
                WHERE subject = ?1 AND upstream_id = ?2
                "#,
                params![subject, upstream_id],
                |row| {
                    let access_nonce: Vec<u8> = row.get(0)?;
                    let access_ciphertext: Vec<u8> = row.get(1)?;
                    let refresh_nonce: Option<Vec<u8>> = row.get(2)?;
                    let refresh_ciphertext: Option<Vec<u8>> = row.get(3)?;
                    let scopes_json: String = row.get(4)?;
                    let expires_at_unix: i64 = row.get(5)?;
                    let refresh_failed: i64 = row.get(6)?;
                    Ok((
                        access_nonce,
                        access_ciphertext,
                        refresh_nonce,
                        refresh_ciphertext,
                        scopes_json,
                        expires_at_unix,
                        refresh_failed,
                    ))
                },
            )
            .optional()
            .map_err(|_| StoreError::OperationFailed)?
            .map(
                |(
                    access_nonce,
                    access_ciphertext,
                    refresh_nonce,
                    refresh_ciphertext,
                    scopes_json,
                    expires_at_unix,
                    refresh_failed,
                )| {
                    let access_token = self.cipher.decrypt(
                        subject,
                        upstream_id,
                        "access_token",
                        &access_nonce,
                        &access_ciphertext,
                    )?;
                    let refresh_token = match (refresh_nonce, refresh_ciphertext) {
                        (Some(nonce), Some(ciphertext)) => Some(self.cipher.decrypt(
                            subject,
                            upstream_id,
                            "refresh_token",
                            &nonce,
                            &ciphertext,
                        )?),
                        _ => None,
                    };
                    let scopes = serde_json::from_str(&scopes_json)
                        .map_err(|_| StoreError::OperationFailed)?;
                    Ok(OAuthCredential {
                        subject: subject.to_string(),
                        upstream_id: upstream_id.to_string(),
                        access_token,
                        refresh_token,
                        scopes,
                        expires_at_unix: expires_at_unix as u64,
                        refresh_failed: refresh_failed != 0,
                    })
                },
            )
            .transpose()
    }

    fn put_client_registration(
        &mut self,
        registration: OAuthClientRegistration,
    ) -> Result<(), StoreError> {
        let encrypted_secret = registration
            .client_secret
            .as_deref()
            .map(|secret| {
                self.cipher.encrypt(
                    &registration.subject,
                    &registration.upstream_id,
                    "client_secret",
                    secret.as_bytes(),
                )
            })
            .transpose()?;
        let (client_secret_nonce, client_secret_ciphertext) = encrypted_secret
            .map(|encrypted| (Some(encrypted.nonce), Some(encrypted.ciphertext)))
            .unwrap_or((None, None));
        self.conn
            .execute(
                r#"
                INSERT INTO oauth_client_registrations (
                    subject, upstream_id, client_id, client_secret_nonce,
                    client_secret_ciphertext, client_id_issued_at_unix,
                    client_secret_expires_at_unix
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(subject, upstream_id) DO UPDATE SET
                    client_id = excluded.client_id,
                    client_secret_nonce = excluded.client_secret_nonce,
                    client_secret_ciphertext = excluded.client_secret_ciphertext,
                    client_id_issued_at_unix = excluded.client_id_issued_at_unix,
                    client_secret_expires_at_unix = excluded.client_secret_expires_at_unix
                "#,
                params![
                    registration.subject,
                    registration.upstream_id,
                    registration.client_id,
                    client_secret_nonce,
                    client_secret_ciphertext,
                    registration
                        .client_id_issued_at_unix
                        .map(|value| value as i64),
                    registration
                        .client_secret_expires_at_unix
                        .map(|value| value as i64),
                ],
            )
            .map_err(|_| StoreError::OperationFailed)?;
        Ok(())
    }

    fn client_registration(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthClientRegistration>, StoreError> {
        self.conn
            .query_row(
                r#"
                SELECT client_id, client_secret_nonce, client_secret_ciphertext,
                       client_id_issued_at_unix, client_secret_expires_at_unix
                FROM oauth_client_registrations
                WHERE subject = ?1 AND upstream_id = ?2
                "#,
                params![subject, upstream_id],
                |row| {
                    let client_id: String = row.get(0)?;
                    let client_secret_nonce: Option<Vec<u8>> = row.get(1)?;
                    let client_secret_ciphertext: Option<Vec<u8>> = row.get(2)?;
                    let client_id_issued_at_unix: Option<i64> = row.get(3)?;
                    let client_secret_expires_at_unix: Option<i64> = row.get(4)?;
                    Ok((
                        client_id,
                        client_secret_nonce,
                        client_secret_ciphertext,
                        client_id_issued_at_unix,
                        client_secret_expires_at_unix,
                    ))
                },
            )
            .optional()
            .map_err(|_| StoreError::OperationFailed)?
            .map(
                |(
                    client_id,
                    client_secret_nonce,
                    client_secret_ciphertext,
                    client_id_issued_at_unix,
                    client_secret_expires_at_unix,
                )| {
                    let client_secret = match (client_secret_nonce, client_secret_ciphertext) {
                        (Some(nonce), Some(ciphertext)) => Some(self.cipher.decrypt(
                            subject,
                            upstream_id,
                            "client_secret",
                            &nonce,
                            &ciphertext,
                        )?),
                        _ => None,
                    };
                    Ok(OAuthClientRegistration {
                        subject: subject.to_string(),
                        upstream_id: upstream_id.to_string(),
                        client_id,
                        client_secret,
                        client_id_issued_at_unix: client_id_issued_at_unix
                            .map(|value| value as u64),
                        client_secret_expires_at_unix: client_secret_expires_at_unix
                            .map(|value| value as u64),
                    })
                },
            )
            .transpose()
    }

    fn clear_subject_upstream(
        &mut self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|_| StoreError::OperationFailed)?;
        tx.execute(
            "DELETE FROM oauth_credentials WHERE subject = ?1 AND upstream_id = ?2",
            params![subject, upstream_id],
        )
        .map_err(|_| StoreError::OperationFailed)?;
        tx.execute(
            "DELETE FROM oauth_pending_states WHERE subject = ?1 AND upstream_id = ?2",
            params![subject, upstream_id],
        )
        .map_err(|_| StoreError::OperationFailed)?;
        tx.execute(
            "DELETE FROM oauth_client_registrations WHERE subject = ?1 AND upstream_id = ?2",
            params![subject, upstream_id],
        )
        .map_err(|_| StoreError::OperationFailed)?;
        tx.commit().map_err(|_| StoreError::OperationFailed)?;
        Ok(())
    }

    fn clear_upstream(&mut self, upstream_id: &str) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|_| StoreError::OperationFailed)?;
        tx.execute(
            "DELETE FROM oauth_credentials WHERE upstream_id = ?1",
            params![upstream_id],
        )
        .map_err(|_| StoreError::OperationFailed)?;
        tx.execute(
            "DELETE FROM oauth_pending_states WHERE upstream_id = ?1",
            params![upstream_id],
        )
        .map_err(|_| StoreError::OperationFailed)?;
        tx.execute(
            "DELETE FROM oauth_client_registrations WHERE upstream_id = ?1",
            params![upstream_id],
        )
        .map_err(|_| StoreError::OperationFailed)?;
        tx.commit().map_err(|_| StoreError::OperationFailed)?;
        Ok(())
    }
}

struct CredentialCipher {
    cipher: Aes256Gcm,
}

impl CredentialCipher {
    fn new(key: [u8; 32]) -> Result<Self, StoreError> {
        Ok(Self {
            cipher: Aes256Gcm::new_from_slice(&key).map_err(|_| StoreError::OperationFailed)?,
        })
    }

    fn encrypt(
        &self,
        subject: &str,
        upstream_id: &str,
        field: &str,
        plaintext: &[u8],
    ) -> Result<EncryptedValue, StoreError> {
        let mut nonce = [0_u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let ciphertext = self
            .cipher
            .encrypt(
                (&nonce).into(),
                Payload {
                    msg: plaintext,
                    aad: aad(subject, upstream_id, field).as_bytes(),
                },
            )
            .map_err(|_| StoreError::OperationFailed)?;
        Ok(EncryptedValue {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    fn decrypt(
        &self,
        subject: &str,
        upstream_id: &str,
        field: &str,
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> Result<String, StoreError> {
        let plaintext = self
            .cipher
            .decrypt(
                nonce.into(),
                Payload {
                    msg: ciphertext,
                    aad: aad(subject, upstream_id, field).as_bytes(),
                },
            )
            .map_err(|_| StoreError::OperationFailed)?;
        String::from_utf8(plaintext).map_err(|_| StoreError::OperationFailed)
    }
}

#[derive(Debug)]
struct EncryptedValue {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

fn aad(subject: &str, upstream_id: &str, field: &str) -> String {
    format!("agentcast/oauth/{subject}/{upstream_id}/{field}")
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), StoreError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|_| StoreError::OperationFailed)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(path, permissions).map_err(|_| StoreError::OperationFailed)
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), StoreError> {
    Ok(())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StoreError {
    #[error("OAuth store operation failed")]
    OperationFailed,
    #[error("sqlite error: {0}")]
    Sqlite(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("json error: {0}")]
    Json(String),
}

impl StoreError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::OperationFailed => "operation_failed",
            Self::Sqlite(_) => "sqlite_error",
            Self::Io(_) => "io_error",
            Self::Json(_) => "json_error",
        }
    }
}
