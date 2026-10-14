use crate::types::BookNumber;
use serde::{Deserialize, Serialize};

/// Represents a record from the `subheadings` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubheadingRecord {
    pub book_number: BookNumber,
    pub chapter: u32,
    pub verse: u32,
    pub order_if_several: Option<u32>,
    pub subheading: String,
}
