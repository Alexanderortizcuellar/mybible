use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "mybible",
    about = "Modern, high-performance MyBible study engine & CLI",
    version = "0.1.0"
)]
pub struct Cli {
    /// Custom modules data directory path (overrides default)
    #[arg(short = 'd', long = "dir", alias = "data-dir", global = true)]
    pub data_dir: Option<PathBuf>,

    /// Disable ANSI colored output in terminal
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,

    /// Output results as structured JSON (for GUIs, scripts, and frontends)
    #[arg(long = "json", global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Read Bible passage or chapter (e.g. 'mybible read "John 3:16"')
    Read {
        /// Bible passage reference (e.g. "John 3:16", "Gen 1:1-5", "Psalm 23")
        reference: String,

        /// Bible translation abbreviation (defaults to active or first loaded Bible)
        #[arg(short = 'b', long = "bible")]
        bible: Option<String>,

        /// Display footnotes if available
        #[arg(long = "footnotes", default_value_t = true)]
        footnotes: bool,

        /// Display cross-references below verses
        #[arg(long = "xrefs")]
        xrefs: bool,

        /// Subheadings module to use (defaults to active subheadings module or embedded stories)
        #[arg(short = 's', long = "subheadings")]
        subheadings: Option<String>,

        /// Disable section subheadings
        #[arg(long = "no-subheadings")]
        no_subheadings: bool,
    },

    /// List books and chapters in a Bible translation (e.g. 'mybible books -b NLT')
    Books {
        /// Bible translation abbreviation (defaults to active or first loaded Bible)
        #[arg(short = 'b', long = "bible")]
        bible: Option<String>,
    },

    /// Search for words, phrases, or Strong's numbers in Bible translations (e.g. 'mybible search "kingdom of God"')
    #[command(alias = "find", alias = "s")]
    Search {
        /// The search query (words or "exact phrase")
        query: String,

        /// Bible module(s) to search in (e.g. -b NLT or -b NLT,KJV). Defaults to active Bible.
        #[arg(short = 'b', long = "bible")]
        bible: Option<String>,

        /// Search across all loaded Bible modules
        #[arg(short = 'a', long = "all")]
        all: bool,

        /// Filter search by book name or number (e.g. "Genesis", "John", "430")
        #[arg(short = 'B', long = "book")]
        book: Option<String>,

        /// Filter search by testament: "ot" (Old Testament) or "nt" (New Testament)
        #[arg(short = 't', long = "testament")]
        testament: Option<String>,

        /// Search mode: "all" (all words), "any" (any word), "phrase" (exact phrase)
        #[arg(short = 'm', long = "mode", default_value = "all")]
        mode: String,

        /// Maximum number of search results to display (0 for unlimited)
        #[arg(short = 'l', long = "limit", default_value_t = 50)]
        limit: usize,

        /// Match case sensitively (default is case-insensitive)
        #[arg(long = "case-sensitive")]
        case_sensitive: bool,

        /// Strict accent matching (default ignores accents/diacritics)
        #[arg(long = "match-accents")]
        match_accents: bool,
    },

    /// Read commentary notes for a passage (e.g. 'mybible commentary "Gen 1:1"')
    #[command(alias = "notes", alias = "comm")]
    Commentary {
        /// Bible passage reference (e.g. "Gen 1:1", "John 3:16")
        reference: String,

        /// Commentary module abbreviation (defaults to active commentary)
        #[arg(short = 'm', long = "module")]
        module: Option<String>,
    },

    /// Lookup a term or Strong's number in dictionary (e.g. 'mybible dict "Aaron"')
    #[command(alias = "lookup")]
    Dict {
        /// Word, topic, or Strong's number (e.g. "Aaron", "H1234", "G2424")
        term: String,

        /// Dictionary module abbreviation (defaults to first loaded dictionary)
        #[arg(short = 'm', long = "module")]
        module: Option<String>,

        /// Suggest closest topics if term is not found
        #[arg(short = 's', long = "suggest", default_value_t = true)]
        suggest: bool,
    },

    /// Show cross-references for a verse (e.g. 'mybible xref "John 3:16"')
    #[command(alias = "cross")]
    Xref {
        /// Bible verse reference (e.g. "John 3:16")
        reference: String,

        /// Preview target verse text using active Bible
        #[arg(short = 'p', long = "preview", default_value_t = true)]
        preview: bool,
    },

    /// Read a reading plan for a specific day (e.g. 'mybible plan 100EJ-p --day 1')
    Plan {
        /// Reading plan module abbreviation
        plan: String,

        /// Day number (1 to max days)
        #[arg(short = 'd', long = "day", default_value_t = 1)]
        day: u32,
    },

    /// Read daily devotional for a day (e.g. 'mybible devotion --day 1')
    Devotion {
        /// Devotions module abbreviation (defaults to first loaded)
        #[arg(short = 'm', long = "module")]
        module: Option<String>,

        /// Day number (1 to 366, defaults to 1)
        #[arg(short = 'd', long = "day", default_value_t = 1)]
        day: u32,
    },

    /// Manage installed and online modules (list, search, download, delete, info)
    #[command(subcommand, alias = "mod")]
    Modules(ModulesCommands),

    /// View or edit CLI configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Run an interactive JSON-RPC server on stdin/stdout for desktop GUIs and applications
    Server,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ModulesCommands {
    /// List all installed modules in the active data directory
    List,

    /// Search the online repository of 4,000+ modules (e.g. 'mybible modules search "esv"')
    Search {
        /// Search query (abbreviation, description, or language)
        query: String,

        /// Filter by 2-letter language code (e.g. "en", "es", "ru")
        #[arg(short = 'l', long = "lang")]
        language: Option<String>,
    },

    /// Download and install a module from the official repository (e.g. 'mybible modules download ESV')
    Download {
        /// Module abbreviation to download (e.g. "ESV", "RV1960", "Strong")
        abbreviation: String,
    },

    /// Delete an installed module from disk
    Delete {
        /// Module abbreviation to delete
        abbreviation: String,
    },

    /// Show detailed metadata and statistics for an installed module
    Info {
        /// Module abbreviation
        abbreviation: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Show current configuration and active data directory
    Get,

    /// Set default data directory
    SetDir { path: PathBuf },

    /// Set default Bible translation
    SetBible { abbreviation: String },

    /// Set default Commentary
    SetCommentary { abbreviation: String },

    /// Set default Dictionary
    SetDictionary { abbreviation: String },

    /// Set default Subheadings module
    SetSubheadings { abbreviation: String },
}
