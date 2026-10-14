use serde::{Deserialize, Serialize};

/// Represents an HTML fragment from the `content_fragments` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentFragmentRecord {
    pub id: String,
    pub fragment: String,
}
