use serde::{Deserialize, Serialize};

/// Represents an article from the `devotions` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevotionRecord {
    pub day: u32,
    pub devotion: String,
}
