use crate::types::BookNumber;
use serde::{Deserialize, Serialize};

/// Represents a commentary or footnote entry from the `commentaries` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentaryRecord {
    pub book_number: BookNumber,
    pub chapter_number_from: u32,
    pub verse_number_from: u32,
    pub chapter_number_to: Option<u32>,
    pub verse_number_to: Option<u32>,
    pub is_preceding: bool,
    pub marker: Option<String>,
    pub text: String,
}

impl CommentaryRecord {
    /// Check if this commentary applies to the entire book (`chapter_number_from == 0`).
    pub fn is_entire_book(&self) -> bool {
        self.chapter_number_from == 0
    }

    /// Check if this commentary applies to the specified verse.
    pub fn applies_to_verse(&self, book: BookNumber, chapter: u32, verse: u32) -> bool {
        if self.book_number != book {
            return false;
        }
        if self.is_entire_book() {
            return true;
        }

        let start_ch = self.chapter_number_from;
        let start_v = self.verse_number_from;

        match (self.chapter_number_to, self.verse_number_to) {
            // Both end chapter and verse are specified
            (Some(end_ch), Some(end_v)) => {
                if chapter < start_ch || chapter > end_ch {
                    return false;
                }
                if chapter == start_ch && verse < start_v {
                    return false;
                }
                if chapter == end_ch && verse > end_v {
                    return false;
                }
                true
            }
            // Only end chapter is specified, goes to the end of end_ch
            (Some(end_ch), None) => {
                if chapter < start_ch || chapter > end_ch {
                    return false;
                }
                if chapter == start_ch && verse < start_v {
                    return false;
                }
                true
            }
            // End chapter is null, end verse is specified (implicitly within start_ch)
            (None, Some(end_v)) => chapter == start_ch && verse >= start_v && verse <= end_v,
            // Both are null: applies strictly to start_ch:start_v
            (None, None) => chapter == start_ch && verse == start_v,
        }
    }

    /// Check if this record is a footnote matching a specific marker in a verse.
    pub fn matches_footnote(
        &self,
        book: BookNumber,
        chapter: u32,
        verse: u32,
        marker: &str,
    ) -> bool {
        if let Some(ref m) = self.marker {
            if m.trim() == marker.trim() {
                return self.applies_to_verse(book, chapter, verse);
            }
        }
        false
    }
}

/// Optional book record in a commentary module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentaryBookRecord {
    pub book_number: BookNumber,
    pub short_name: String,
}
