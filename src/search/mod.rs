pub mod matcher;
pub mod types;

pub use matcher::search_verse_text;
pub use types::{MatchSpan, SearchHit, SearchMode, SearchQuery, SearchResults, SearchScope};
