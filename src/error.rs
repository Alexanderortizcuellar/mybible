use std::path::PathBuf;
use thiserror::Error;

/// Core error types for the MyBible engine.
#[derive(Debug, Error)]
pub enum MyBibleError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Invalid module format: {0}")]
    InvalidFormat(String),

    #[error("Missing required table '{table}' in module '{module}'")]
    MissingTable { table: String, module: String },

    #[error("Unknown or unsupported module file: {0}")]
    UnknownModuleFile(PathBuf),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Book number {0} is invalid or unrecognized")]
    InvalidBookNumber(u16),

    #[error("Verse not found: book {book}, chapter {chapter}, verse {verse}")]
    VerseNotFound { book: u16, chapter: u32, verse: u32 },

    #[error("Fragment '{fragment_id}' not found (module: {module_name:?})")]
    FragmentNotFound {
        fragment_id: String,
        module_name: Option<String>,
    },

    #[error("Fragment circular inclusion detected for id '{0}'")]
    CircularFragmentInclusion(String),

    #[error("Dictionary topic '{0}' not found")]
    TopicNotFound(String),

    #[error("Module '{0}' not found")]
    ModuleNotFound(String),

    #[error("HTTP request error: {0}")]
    Http(String),
}

pub type Result<T> = std::result::Result<T, MyBibleError>;
