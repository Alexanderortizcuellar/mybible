use crate::types::BookNumber;
use serde::{Deserialize, Serialize};

/// Represents an item from the `reading_plan` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingPlanRecord {
    pub day: u32,
    pub evening: bool,
    pub item: u32,
    pub book_number: BookNumber,
    pub start_chapter: u32,
    pub start_verse: Option<u32>,
    pub end_chapter: u32,
    pub end_verse: Option<u32>,
}
