# mybible

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/github-Alexanderortizcuellar%2Fmybible-blue.svg)](https://github.com/Alexanderortizcuellar/mybible)

A fast, memory-safe, and thread-safe Rust engine and CLI tool for the **MyBible** ecosystem. It provides SQLite database readers, text parsers, morphology and Strong's concordance normalizers, a rich terminal interface, and an integrated downloader connecting directly to official MyBible repositories hosting 4,000+ modules.

---

## Features

### 📖 Complete Module Support
- **Bibles (`.SQLite3`, `.bbl.SQLite3`)**:
  - Full OT/NT and apocrypha books support with `books_all` / `books` sorting and metadata.
  - Chapter and verse lookups, range queries (e.g. `Gen 1:1-2:3`), and whole-module text search.
  - Stories and subheadings rendering.
  - Book introductions and module introductions.
  - Embedded morphology indicators and topics.
- **Commentaries & Footnotes (`.commentaries.SQLite3`)**:
  - Chapter and verse commentary aggregation.
  - Book introductions (`chapter_number_from = 0`).
  - Footnote marker alignment (`[1]`, `*`).
- **Dictionaries & Lexicons (`.dictionary.SQLite3`, `.dct.SQLite3`)**:
  - Strong's Concordance lookups (Hebrew `H...` and Greek `G...`).
  - Prefix search and full article retrieval.
  - Cognates and synonymous Strong's numbers lookup.
  - Multi-form word normalization algorithms (Form 0, Form 1, Form 2 with diacritics removal and Russian/Greek normalization).
- **Cross-References (`.crossreferences.SQLite3`)**:
  - Forward and reverse reference mapping with vote-based ranking and deduplication.
- **Reading Plans (`.plan.SQLite3`)**:
  - Daily morning and evening reading schedules.
- **Daily Devotions (`.devotions.SQLite3`)**:
  - Day-by-day devotional text reader.

### 🎨 Formatting & Rich Text Engine
- **HTML & BBCode Parsing**: Translates MyBible HTML formatting tags (`<i>`, `<t>`, `<J>` for words of Jesus, `<S>` for Strong's numbers, `<m>` for morphology) into clean, readable terminal output.
- **Hyperlinks**: Full parser for `bible://`, `dict://`, and `commentary://` URIs.
- **Fragment Expansion**: Automatically resolves `<include id="..." from="..." textual="..."/>` tags against active modules.
- **Multilingual Reference Parser**: Accepts English, Spanish (`Génesis 1:1`, `Juan 3:16`), Russian (`Бытие 1:1`), as well as standard abbreviations (`Gen`, `Jn`, `Rom`).

### 🌐 Module Downloader & Repository Client
- Built-in HTTP client that fetches the official MyBible module registry catalog (`registry.json` / `registry.json.gz`).
- Search over 4,000+ available modules by keyword, language, or type.
- One-command download, ZIP extraction, and automatic directory placement.

### ⚡ Thread-Safe Architecture (`Send + Sync`)
All readers wrap SQLite connections safely in synchronization primitives, making [`MyBibleEngine`](src/engine/manager.rs) fully `Send + Sync`. It is ready to be embedded directly into multi-threaded backends or desktop GUI frameworks such as **Tauri**, **Slint**, or **Iced**.

---

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.75+ recommended)

### Build from source
```bash
git clone https://github.com/Alexanderortizcuellar/mybible.git
cd mybible
cargo build --release
```

The compiled binary will be located at `target/release/mybible` (or `target\release\mybible.exe` on Windows).

To install globally:
```bash
cargo install --path .
```

---

## CLI Usage

### Reading Scripture
```bash
# Read a single verse
mybible read "John 3:16"

# Read a passage range
mybible read "Genesis 1:1-5"

# Specify a specific Bible version
mybible read "Rom 8:28" --bible NLT

# Include commentary, cross-references, and Strong's numbers in the output
mybible read "Jn 1:1" --with-commentary --with-crossrefs --strong
```

### Reading Commentaries & Footnotes
```bash
# Read commentary for a verse
mybible commentary "Romans 8:28"

# Read commentary using a specific module
mybible commentary "1 John 1:9" --commentary ESVstudy
```

### Looking Up Dictionaries & Strong's Concordance
```bash
# Look up a Strong's Greek entry
mybible dict "G26"

# Look up an English topic or Hebrew Strong number
mybible dict "H7225"
```

### Cross-References
```bash
# View top cross-references for a verse
mybible xref "John 1:1"
```

### Reading Plans & Devotions
```bash
# View reading items for Day 1
mybible plan 1

# Read devotional text for Day 1
mybible devotion 1
```

### Managing Modules
```bash
# List all currently installed modules
mybible modules list

# Search the remote repository for modules (over 4,000 available)
mybible modules search "Spanish" --type bible
mybible modules search "Commentary"

# Download a module directly from the repository
mybible modules download "RVA"

# Show detailed metadata about a local module
mybible modules info "NLT"
```

### Configuration
`mybible` saves configuration settings to `~/.mybible/config.toml` (or `%APPDATA%\mybible\config.toml` on Windows).

```bash
# View active configuration and paths
mybible config

# Set the default directory where modules are stored
mybible config --set-dir "C:/path/to/my/modules"

# Set default active modules
mybible config --set-bible NLT
mybible config --set-commentary ESVstudy
```

---

## Rust Library API Example

You can also use `mybible` as a library in your own Rust applications:

```rust
use mybible::engine::MyBibleEngine;
use mybible::types::BookNumber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = MyBibleEngine::new();

    // Load modules from a folder
    engine.load_directory("./modules")?;

    // Read a verse
    if let Some(verse) = engine.get_verse("NLT", BookNumber(430), 3, 16)? {
        println!("John 3:16: {}", verse.text);
    }

    // Retrieve cross-references
    if let Some(refs) = engine.get_cross_references(BookNumber(430), 1, 1)? {
        println!("Found {} cross-references for John 1:1", refs.len());
    }

    Ok(())
}
```

---

## Running Tests

All unit tests and integration tests can be run via Cargo:

```bash
cargo test
```

To run formatting and clippy verification:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets
```

---

## Contributing

Pull requests and issues are welcome! Feel free to report bugs, suggest new CLI subcommands, or contribute parsers.

---

## License

This project is licensed under the [MIT License](LICENSE).
