use crate::types::BookNumber;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mode defining how query terms are matched in verses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    /// Exact contiguous sequence of words/characters.
    Phrase,
    /// All space-separated words must appear anywhere in the verse (boolean AND).
    AllWords,
    /// Any space-separated word may appear in the verse (boolean OR).
    AnyWord,
}

impl std::str::FromStr for SearchMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "phrase" | "exact" => Ok(SearchMode::Phrase),
            "all" | "and" => Ok(SearchMode::AllWords),
            "any" | "or" => Ok(SearchMode::AnyWord),
            _ => Err(format!(
                "Unknown search mode '{}'. Expected 'all', 'any', or 'phrase'",
                s
            )),
        }
    }
}

/// Scope defining which books of the Bible to search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchScope {
    /// Entire Bible (Old Testament + New Testament + Apocrypha if present)
    All,
    /// Old Testament only (books 10..=390)
    OldTestament,
    /// New Testament only (books 400..=660)
    NewTestament,
    /// Specific single book
    Book(BookNumber),
    /// Range of book numbers (inclusive)
    BookRange(u16, u16),
}

/// A search query specifying text, matching mode, scope filters, and options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub mode: SearchMode,
    pub scope: SearchScope,
    pub case_sensitive: bool,
    pub ignore_accents: bool,
    pub limit: Option<usize>,
}

impl SearchQuery {
    /// Creates a new search query from text.
    ///
    /// Automatically detects if the string is quoted in quotes (`"..."` or `'...'`)
    /// and switches mode to `SearchMode::Phrase`. Otherwise defaults to `SearchMode::AllWords`.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let trimmed = text.trim();
        let is_quoted = (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2);

        let cleaned = if is_quoted {
            trimmed[1..trimmed.len() - 1].trim()
        } else {
            trimmed
        };

        let mode = if is_quoted {
            SearchMode::Phrase
        } else {
            SearchMode::AllWords
        };

        Self {
            text: cleaned.to_string(),
            mode,
            scope: SearchScope::All,
            case_sensitive: false,
            ignore_accents: true,
            limit: Some(50),
        }
    }

    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_scope(mut self, scope: SearchScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_case_sensitive(mut self, cs: bool) -> Self {
        self.case_sensitive = cs;
        self
    }

    pub fn with_ignore_accents(mut self, ia: bool) -> Self {
        self.ignore_accents = ia;
        self
    }

    pub fn with_limit(mut self, limit: Option<usize>) -> Self {
        self.limit = limit;
        self
    }
}

/// Start and end byte offsets of a match in `plain_text`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSpan {
    pub start: usize,
    pub end: usize,
}

/// A matched Bible verse returned from a search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub bible_module: String,
    pub book_number: BookNumber,
    pub book_name: String,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
    pub plain_text: String,
    pub matches: Vec<MatchSpan>,
}

impl SearchHit {
    /// Formats the scripture reference (e.g. "Genesis 1:1" or "Matthew 3:2").
    pub fn format_reference(&self) -> String {
        format!("{} {}:{}", self.book_name, self.chapter, self.verse)
    }

    /// Renders the plain verse text with matched search terms highlighted.
    ///
    /// When `use_color` is true, highlights matches with bold bright yellow.
    /// When `use_color` is false, wraps matches in brackets `[...]`.
    pub fn render_highlighted(&self, use_color: bool) -> String {
        if self.matches.is_empty() {
            return self.plain_text.clone();
        }

        // 1. Sort matches by starting offset
        let mut sorted_spans = self.matches.clone();
        sorted_spans.sort_by_key(|s| s.start);

        // 2. Merge overlapping or contiguous spans
        let mut merged: Vec<MatchSpan> = Vec::new();
        for span in sorted_spans {
            if span.start >= self.plain_text.len() || span.end > self.plain_text.len() {
                continue;
            }
            if let Some(last) = merged.last_mut() {
                if span.start <= last.end {
                    if span.end > last.end {
                        last.end = span.end;
                    }
                    continue;
                }
            }
            merged.push(span);
        }

        // 3. Construct highlighted string
        let mut out = String::with_capacity(self.plain_text.len() + 64);
        let mut cursor = 0;

        for span in merged {
            if span.start > cursor {
                out.push_str(&self.plain_text[cursor..span.start]);
            }
            let matched_slice = &self.plain_text[span.start..span.end];
            if use_color {
                out.push_str(&matched_slice.bright_yellow().bold().underline().to_string());
            } else {
                out.push('[');
                out.push_str(matched_slice);
                out.push(']');
            }
            cursor = span.end;
        }

        if cursor < self.plain_text.len() {
            out.push_str(&self.plain_text[cursor..]);
        }

        out
    }
}

/// Aggregated search results across one or more Bible modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: SearchQuery,
    pub total_matches: usize,
    pub matches_by_module: HashMap<String, usize>,
    pub hits: Vec<SearchHit>,
    pub duration_ms: u128,
}
