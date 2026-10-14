use crate::types::BookNumber;
use serde::{Deserialize, Serialize};

/// Represents a dictionary article from the `dictionary` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictionaryRecord {
    pub topic: String,
    pub definition: String,
    pub short_definition: Option<String>,
    pub lexeme: Option<String>,
    pub transliteration: Option<String>,
    pub pronunciation: Option<String>,
}

/// Represents a word form mapping from the `words` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordRecord {
    pub standard_form: String,
    pub variation: String,
    pub book_number: Option<BookNumber>,
    pub chapter_number: Option<u32>,
    pub verse_number: Option<u32>,
}

/// Type of text processing in `words_processing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WordsProcessingType {
    Pre,
    Post,
}

impl WordsProcessingType {
    pub fn from_str_info(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "pre" => Some(WordsProcessingType::Pre),
            "post" => Some(WordsProcessingType::Post),
            _ => None,
        }
    }
}

/// Rule from `words_processing` table for language-specific replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordsProcessingRecord {
    pub proc_type: WordsProcessingType,
    pub input: String,
    pub output: String,
}

/// Pre-indexed topic entry from `lookup_topics`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupTopicRecord {
    pub topic: String,
    pub topic_hash: i32,
    pub dictionary_id: u32,
}

/// Pre-indexed word entry from `lookup_words`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupWordRecord {
    pub word_nf1: String,
    pub word_nf2: String,
    pub topic_hash1: i32,
    pub topic_hash2: i32,
    pub book_number: u16,
    pub chapter_number: u32,
    pub verse_number: u32,
    pub source_dictionary_id: u32,
    pub target_dictionary_id: u32,
}

/// Pre-indexed reference to a Bible verse from `lookup_references`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupReferenceRecord {
    pub topic: String,
    pub book_number: BookNumber,
    pub chapter_number: u32,
    pub verse_number: u32,
    pub dictionary_id: Option<u32>,
}
