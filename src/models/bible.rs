use crate::types::BookNumber;
use serde::{Deserialize, Serialize};

/// Represents a record from either `books` or `books_all` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookRecord {
    pub book_number: BookNumber,
    pub book_color: String,
    pub short_name: String,
    pub title: Option<String>,
    pub long_name: String,
    pub is_present: bool,
    pub sorting_order: Option<u32>,
}

impl BookRecord {
    /// Return the preferred title to display for this book.
    pub fn display_title(&self) -> &str {
        if let Some(ref t) = self.title {
            if !t.trim().is_empty() {
                return t;
            }
        }
        &self.long_name
    }
}

/// Represents a single Bible verse from the `verses` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerseRecord {
    pub book_number: BookNumber,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

impl VerseRecord {
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

/// Module or book introduction from the `introductions` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntroductionRecord {
    pub book_number: BookNumber,
    pub introduction: String,
}

impl IntroductionRecord {
    /// Check if this is an overall module introduction (book_number == 0).
    pub fn is_module_intro(&self) -> bool {
        self.book_number.0 == 0
    }
}

/// Embedded subheading or story from the `stories` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryRecord {
    pub book_number: BookNumber,
    pub chapter: u32,
    pub verse: u32,
    pub order_if_several: Option<u32>,
    pub title: String,
}
