pub mod document;
pub mod index;
pub mod query;

pub use document::SearchDocument;
pub use index::{SearchIndex, SearchMatchKind, SearchResult};
pub use query::SearchQuery;
