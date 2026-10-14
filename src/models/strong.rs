use serde::{Deserialize, Serialize};

/// Represents an entry from `cognate_strong_numbers` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognateRecord {
    pub group_id: u32,
    pub strong_number: String,
}

/// Represents an entry from `synonymous_strong_numbers` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynonymRecord {
    pub group_id: u32,
    pub strong_number: String,
}
