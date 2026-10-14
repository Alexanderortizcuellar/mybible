use serde::{Deserialize, Serialize};

/// Represents an entry from `morphology_indications` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MorphologyIndicationRecord {
    pub indication: String,
    pub applicable_to: Option<String>,
    pub language: Option<String>,
    pub meaning: String,
}

/// Represents an entry from `morphology_topics` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MorphologyTopicRecord {
    pub indication: String,
    pub topic: String,
}
