//! # MyBible Engine
//!
//! A comprehensive, high-performance Rust engine and parser for MyBible modules.
//!
//! Recreates the MyBible format specification and runtime logic:
//! - Full support for all MyBible module types (Bible, Dictionary/Lexicon, Commentaries/Footnotes,
//!   Cross References, Subheadings, Reading Plans, Daily Devotions, Bundles, and Zipped archives).
//! - SQLite database readers adhering strictly to MyBible's schema rules and conventions.
//! - Parsers for embedded verse tags (`<S>`, `<m>`, `<i>`, `<J>`, `<n>`, `<e>`, `<t>`, `<br/>`, `<pb/>`, `<f>`, `<h>`).
//! - Hyperlink target parsers (`B:...`, `S:...`, `C:...`, `http(s)://`, `<x>...</x>`).
//! - HTML fragment inclusion engine (`<!-- INCLUDE(...) -->`) with cycle and recursion protection.
//! - Theme color and style resolvers (`%COLOR_TEXT%`, `%COLOR_RED%`, `<font color='0..5'>`, etc.).
//! - Dictionary lookup engine supporting NF0, NF1, NF2 normalizations, Java `String.hashCode()` compatibility,
//!   and multi-dictionary fallback matching.

pub mod books;
pub mod cli;
pub mod engine;
pub mod error;
pub mod lookup;
pub mod metadata;
pub mod models;
pub mod parsers;
pub mod readers;
pub mod repository;
pub mod terminal;
pub mod types;

pub use books::{find_book_by_name, find_book_by_number, ALL_BOOKS};
pub use engine::{CompositeChapter, CompositeVerse, MyBibleEngine};
pub use error::{MyBibleError, Result};
pub use lookup::{
    java_hash_code, normalize_form_0, normalize_form_1, normalize_form_2, DictionaryLookupEngine,
    LookupMatch, LookupMode,
};
pub use metadata::InfoTable;
pub use models::*;
pub use parsers::{
    apply_theme_colors, expand_fragments, nodes_to_html, nodes_to_plain_text, parse_bible_location,
    parse_fragment_includes, parse_hyperlink, parse_verse_text, BibleLocation, FragmentInclude,
    HyperlinkTarget, ThemeColors, VerseNode, VerseRenderOptions,
};
pub use readers::{
    detect_module_file, BibleReader, CommentaryReader, CrossReferenceReader, DevotionReader,
    DictionaryReader, ReadingPlanReader, SubheadingReader, ZipModuleArchive,
};
pub use repository::{
    LanguageDescription, ModuleDownloadEntry, ModuleRegistry, RegistryHost, RepositoryClient,
    OFFICIAL_REGISTRY_URL,
};
pub use types::{
    BookNumber, DictionaryType, MatchingType, ModuleType, Testament, VerseRange, VerseRef,
};

/// Convenient prelude module importing all primary types and functions.
pub mod prelude {
    pub use crate::books::{find_book_by_name, find_book_by_number, ALL_BOOKS};
    pub use crate::engine::{CompositeChapter, CompositeVerse, MyBibleEngine};
    pub use crate::error::{MyBibleError, Result};
    pub use crate::lookup::{
        java_hash_code, normalize_form_0, normalize_form_1, normalize_form_2,
        DictionaryLookupEngine, LookupMatch, LookupMode,
    };
    pub use crate::metadata::InfoTable;
    pub use crate::models::{
        BookRecord, CommentaryBookRecord, CommentaryRecord, ContentFragmentRecord,
        CrossReferenceRecord, DevotionRecord, DictionaryRecord, IntroductionRecord,
        LookupReferenceRecord, LookupTopicRecord, LookupWordRecord, MorphologyIndicationRecord,
        MorphologyTopicRecord, ReadingPlanRecord, StoryRecord, SubheadingRecord, VerseRecord,
        WordRecord, WordsProcessingRecord, WordsProcessingType,
    };
    pub use crate::parsers::{
        apply_theme_colors, expand_fragments, nodes_to_html, nodes_to_plain_text,
        parse_bible_location, parse_fragment_includes, parse_hyperlink, parse_verse_text,
        BibleLocation, FragmentInclude, HyperlinkTarget, ThemeColors, VerseNode,
        VerseRenderOptions,
    };
    pub use crate::readers::{
        detect_module_file, BibleReader, CommentaryReader, CrossReferenceReader, DevotionReader,
        DictionaryReader, ReadingPlanReader, SubheadingReader, ZipModuleArchive,
    };
    pub use crate::repository::{
        LanguageDescription, ModuleDownloadEntry, ModuleRegistry, RegistryHost, RepositoryClient,
        OFFICIAL_REGISTRY_URL,
    };
    pub use crate::types::*;
}
