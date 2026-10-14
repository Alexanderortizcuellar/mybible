use serde::{Deserialize, Serialize};
use std::fmt;

/// Strongly-typed Bible book number according to MyBible specification (10..780).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BookNumber(pub u16);

impl BookNumber {
    pub const GENESIS: BookNumber = BookNumber(10);
    pub const MALACHI: BookNumber = BookNumber(460);
    pub const MATTHEW: BookNumber = BookNumber(470);
    pub const REVELATION: BookNumber = BookNumber(730);

    pub fn new(num: u16) -> Self {
        Self(num)
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }

    pub fn is_old_testament(self) -> bool {
        (10..=460).contains(&self.0) && !self.is_apocrypha()
    }

    pub fn is_new_testament(self) -> bool {
        (470..=730).contains(&self.0) && self.0 != 780
    }

    pub fn is_apocrypha(self) -> bool {
        matches!(
            self.0,
            145 | 165
                | 170
                | 180
                | 192
                | 270
                | 280
                | 305
                | 315
                | 320
                | 323
                | 325
                | 345
                | 462
                | 464
                | 466
                | 467
                | 468
                | 780
        )
    }

    pub fn testament(self) -> Testament {
        if self.is_apocrypha() {
            Testament::Apocrypha
        } else if (10..=460).contains(&self.0) {
            Testament::OldTestament
        } else {
            Testament::NewTestament
        }
    }
}

impl From<u16> for BookNumber {
    fn from(n: u16) -> Self {
        Self(n)
    }
}

impl From<BookNumber> for u16 {
    fn from(b: BookNumber) -> Self {
        b.0
    }
}

impl fmt::Display for BookNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Division of the Bible text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Testament {
    OldTestament,
    NewTestament,
    Apocrypha,
}

/// Reference to a specific Bible verse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VerseRef {
    pub book: BookNumber,
    pub chapter: u32,
    pub verse: u32,
}

impl VerseRef {
    pub fn new(book: impl Into<BookNumber>, chapter: u32, verse: u32) -> Self {
        Self {
            book: book.into(),
            chapter,
            verse,
        }
    }
}

impl fmt::Display for VerseRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Book {} {}:{}", self.book.0, self.chapter, self.verse)
    }
}

/// A range of Bible verses across chapters or within a single chapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VerseRange {
    pub book: BookNumber,
    pub start_chapter: u32,
    pub start_verse: u32,
    pub end_chapter: u32,
    pub end_verse: u32,
}

impl VerseRange {
    pub fn new(
        book: impl Into<BookNumber>,
        start_chapter: u32,
        start_verse: u32,
        end_chapter: u32,
        end_verse: u32,
    ) -> Self {
        Self {
            book: book.into(),
            start_chapter,
            start_verse,
            end_chapter,
            end_verse,
        }
    }

    pub fn single_verse(book: impl Into<BookNumber>, chapter: u32, verse: u32) -> Self {
        let b = book.into();
        Self {
            book: b,
            start_chapter: chapter,
            start_verse: verse,
            end_chapter: chapter,
            end_verse: verse,
        }
    }

    pub fn contains(&self, verse_ref: &VerseRef) -> bool {
        if self.book != verse_ref.book {
            return false;
        }
        if verse_ref.chapter < self.start_chapter || verse_ref.chapter > self.end_chapter {
            return false;
        }
        if verse_ref.chapter == self.start_chapter && verse_ref.verse < self.start_verse {
            return false;
        }
        if verse_ref.chapter == self.end_chapter && verse_ref.verse > self.end_verse {
            return false;
        }
        true
    }
}

/// The type of MyBible module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleType {
    /// Bible module (`<abbrev>.SQLite3`)
    Bible,
    /// Dictionary module (`<abbrev>.dictionary.SQLite3`)
    Dictionary,
    /// Subheadings module (`<abbrev>.subheadings.SQLite3`)
    Subheadings,
    /// Cross references module (`<abbrev>.crossreferences.SQLite3`)
    CrossReferences,
    /// Commentaries module (`<abbrev>.commentaries.SQLite3`)
    Commentaries,
    /// Reading plan module (`<abbrev>.plan.SQLite3`)
    ReadingPlan,
    /// Daily devotions module (`<abbrev>.devotions.SQLite3`)
    Devotions,
    /// Bundle module containing multiple modules and assets (`<abbrev>.bundle.zip`)
    Bundle,
}

impl ModuleType {
    /// Return the standard file suffix for this module type.
    pub fn suffix(self) -> &'static str {
        match self {
            ModuleType::Bible => "",
            ModuleType::Dictionary => ".dictionary",
            ModuleType::Subheadings => ".subheadings",
            ModuleType::CrossReferences => ".crossreferences",
            ModuleType::Commentaries => ".commentaries",
            ModuleType::ReadingPlan => ".plan",
            ModuleType::Devotions => ".devotions",
            ModuleType::Bundle => ".bundle",
        }
    }
}

/// Dictionary type specification as documented in `INFO.type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DictionaryType {
    #[default]
    Explanatory,
    Translator,
    Concordance,
    Lexicon,
    StrongLexicon,
}

impl DictionaryType {
    pub fn from_str_info(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "translator" => DictionaryType::Translator,
            "concordance" => DictionaryType::Concordance,
            "lexicon" => DictionaryType::Lexicon,
            "strong lexicon" | "strong" => DictionaryType::StrongLexicon,
            _ => DictionaryType::Explanatory,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            DictionaryType::Explanatory => "explanatory",
            DictionaryType::Translator => "translator",
            DictionaryType::Concordance => "concordance",
            DictionaryType::Lexicon => "lexicon",
            DictionaryType::StrongLexicon => "strong lexicon",
        }
    }
}

/// Matching type for standard form dictionary lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MatchingType {
    NormalizedForm1 = 1,
    #[default]
    NormalizedForm2 = 2,
    Exact = 3,
}

impl MatchingType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => MatchingType::NormalizedForm1,
            3 => MatchingType::Exact,
            _ => MatchingType::NormalizedForm2,
        }
    }

    pub fn from_str_info(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "normalized form 1" | "nf1" => MatchingType::NormalizedForm1,
            "exact" => MatchingType::Exact,
            _ => MatchingType::NormalizedForm2,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}
