pub mod catalog;
pub mod migrations;
pub mod oauth;
pub mod sqlite;

pub type StoreResult<T> = Result<T, StoreError>;

pub use catalog::{CatalogSnapshot, SqliteCatalogStore};
pub use migrations::run_migrations;
pub use oauth::{InMemoryOAuthStore, OAuthStore, SqliteOAuthStore, StoreError};
pub use sqlite::open_sqlite_store;
