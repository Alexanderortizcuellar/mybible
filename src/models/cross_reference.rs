use crate::types::BookNumber;
use serde::{Deserialize, Serialize};

/// Represents a cross reference entry from the `cross_references` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossReferenceRecord {
    pub book: BookNumber,
    pub chapter: u32,
    pub verse: u32,
    pub verse_end: Option<u32>,
    pub book_to: BookNumber,
    pub chapter_to: u32,
    pub verse_to_start: u32,
    pub verse_to_end: Option<u32>,
    pub votes: Option<i32>,
}

impl CrossReferenceRecord {
    /// Check if the given source verse matches this record's source.
    pub fn matches_source(&self, book: BookNumber, chapter: u32, verse: u32) -> bool {
        if self.book != book || self.chapter != chapter {
            return false;
        }
        match self.verse_end {
            Some(end) if end > 0 => verse >= self.verse && verse <= end,
            _ => verse == self.verse,
        }
    }

    /// Check if the given verse matches this record's target range (for reverse processing).
    pub fn matches_target(&self, book: BookNumber, chapter: u32, verse: u32) -> bool {
        if self.book_to != book || self.chapter_to != chapter {
            return false;
        }
        match self.verse_to_end {
            Some(end) if end > 0 => verse >= self.verse_to_start && verse <= end,
            _ => verse == self.verse_to_start,
        }
    }

    /// Invert the cross reference from target to source.
    pub fn reversed(&self) -> CrossReferenceRecord {
        CrossReferenceRecord {
            book: self.book_to,
            chapter: self.chapter_to,
            verse: self.verse_to_start,
            verse_end: self.verse_to_end,
            book_to: self.book,
            chapter_to: self.chapter,
            verse_to_start: self.verse,
            verse_to_end: self.verse_end,
            votes: self.votes,
        }
    }

    /// Formats the destination reference into a display string (e.g. "Rom 5:8" or "John 3:16-17").
    pub fn format_target(&self) -> String {
        let book_name = crate::books::find_book_by_number(self.book_to)
            .map(|b| b.short_name_eng)
            .unwrap_or("Book");
        match self.verse_to_end {
            Some(end) if end != self.verse_to_start && end > 0 => {
                format!(
                    "{} {}:{}-{}",
                    book_name, self.chapter_to, self.verse_to_start, end
                )
            }
            _ => format!("{} {}:{}", book_name, self.chapter_to, self.verse_to_start),
        }
    }
}
