use crate::error::{MyBibleError, Result};
use crate::lookup::DictionaryLookupEngine;
use crate::models::bible::{BookRecord, VerseRecord};
use crate::models::commentary::CommentaryRecord;
use crate::models::cross_reference::CrossReferenceRecord;
use crate::models::subheading::SubheadingRecord;
use crate::parsers::fragments::{expand_fragments, FragmentResolver};
use crate::parsers::html_style::{apply_theme_colors, ThemeColors};
use crate::readers::bible::BibleReader;
use crate::readers::commentary::CommentaryReader;
use crate::readers::cross_reference::CrossReferenceReader;
use crate::readers::devotion::DevotionReader;
use crate::readers::dictionary::DictionaryReader;
use crate::readers::file_detector::detect_module_file;
use crate::readers::reading_plan::ReadingPlanReader;
use crate::readers::subheading::SubheadingReader;
use crate::readers::zip_reader::ZipModuleArchive;
use crate::search::{SearchQuery, SearchResults};
use crate::types::{BookNumber, ModuleType};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// A fully aggregated verse view containing the text, subheadings, cross-references, and footnotes/commentaries.
#[derive(Debug, Clone)]
pub struct CompositeVerse {
    pub verse: VerseRecord,
    pub subheadings: Vec<SubheadingRecord>,
    pub cross_references: Vec<CrossReferenceRecord>,
    pub footnotes: Vec<CommentaryRecord>,
    pub commentaries: Vec<CommentaryRecord>,
}

/// A chapter view containing all composite verses and stories for a chapter.
#[derive(Debug, Clone)]
pub struct CompositeChapter {
    pub book: BookRecord,
    pub chapter: u32,
    pub verses: Vec<CompositeVerse>,
}

/// The core MyBible engine managing loaded modules, multi-module coordination, and HTML rendering.
pub struct MyBibleEngine {
    pub bibles: HashMap<String, Arc<BibleReader>>,
    pub commentaries: HashMap<String, Arc<CommentaryReader>>,
    pub dictionaries: HashMap<String, Arc<DictionaryReader>>,
    pub cross_references: HashMap<String, Arc<CrossReferenceReader>>,
    pub subheadings: HashMap<String, Arc<SubheadingReader>>,
    pub reading_plans: HashMap<String, Arc<ReadingPlanReader>>,
    pub devotions: HashMap<String, Arc<DevotionReader>>,

    pub active_bible: Option<String>,
    pub active_commentary: Option<String>,
    pub active_cross_references: Option<String>,
    pub active_subheadings: Option<String>,

    pub lookup_engine: DictionaryLookupEngine,
    pub theme: ThemeColors,
}

impl Default for MyBibleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MyBibleEngine {
    pub fn new() -> Self {
        Self {
            bibles: HashMap::new(),
            commentaries: HashMap::new(),
            dictionaries: HashMap::new(),
            cross_references: HashMap::new(),
            subheadings: HashMap::new(),
            reading_plans: HashMap::new(),
            devotions: HashMap::new(),

            active_bible: None,
            active_commentary: None,
            active_cross_references: None,
            active_subheadings: None,

            lookup_engine: DictionaryLookupEngine::new(),
            theme: ThemeColors::day(),
        }
    }

    /// Returns the total count of all loaded modules across all categories.
    pub fn total_modules(&self) -> usize {
        self.bibles.len()
            + self.commentaries.len()
            + self.dictionaries.len()
            + self.cross_references.len()
            + self.subheadings.len()
            + self.reading_plans.len()
            + self.devotions.len()
    }

    /// Load all modules found in a directory (recursively scanning subdirectories).
    pub fn load_directory(&mut self, dir_path: impl AsRef<Path>) -> Result<usize> {
        let dir = dir_path.as_ref();
        if !dir.is_dir() {
            return Ok(0);
        }

        let mut loaded_count = 0;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                loaded_count += self.load_directory(&path)?;
            } else if path.is_file() && self.load_module_file(&path).is_ok() {
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    /// Downloads a module from the repository catalog and immediately loads it into the engine.
    pub fn download_and_load(
        &mut self,
        client: &crate::repository::RepositoryClient,
        entry: &crate::repository::ModuleDownloadEntry,
        registry: &crate::repository::ModuleRegistry,
        dest_dir: impl AsRef<Path>,
    ) -> Result<ModuleType> {
        let extracted_path = client.download_module(entry, registry, dest_dir)?;
        self.load_module_file(&extracted_path)
    }

    /// Loads a single module file (.SQLite3, .zip, etc.).
    pub fn load_module_file(&mut self, path: impl AsRef<Path>) -> Result<ModuleType> {
        let p = path.as_ref();
        let info = detect_module_file(p)
            .ok_or_else(|| MyBibleError::UnknownModuleFile(p.to_path_buf()))?;

        if info.is_zipped {
            let archive = ZipModuleArchive::open(p)?;
            // If it's a single zipped module, extract or load in-memory
            for entry in archive.entries() {
                let bytes = archive.read_bytes(&entry.inner_path)?;
                let mem_conn = rusqlite::Connection::open_in_memory()?;
                // Use SQLite backup API or execute batch
                // For bundled modules in zip, we deserialize SQLite database directly into memory
                mem_conn.execute_batch("PRAGMA synchronous = OFF;")?;
                // rusqlite allows opening or deserializing
                // We can use rusqlite's deserialize feature if available, or write to a temp file
                let temp_dir = std::env::temp_dir().join("mybible_extracted");
                let _ = fs::create_dir_all(&temp_dir);
                let target_file = temp_dir.join(format!("{}.SQLite3", entry.abbreviation));
                fs::write(&target_file, bytes)?;
                self.load_unzipped_module(&target_file, &entry.abbreviation, entry.module_type)?;
            }
            return Ok(info.module_type);
        }

        self.load_unzipped_module(p, &info.abbreviation, info.module_type)?;
        Ok(info.module_type)
    }

    fn load_unzipped_module(&mut self, path: &Path, abbrev: &str, mtype: ModuleType) -> Result<()> {
        match mtype {
            ModuleType::Bible => match BibleReader::open(path, abbrev) {
                Ok(reader) => {
                    let arc = Arc::new(reader);
                    if self.active_bible.is_none() {
                        self.active_bible = Some(abbrev.to_string());
                    }
                    self.bibles.insert(abbrev.to_string(), arc);
                }
                Err(MyBibleError::MissingTable { .. }) => {
                    if let Ok(c_reader) = CommentaryReader::open(path, abbrev) {
                        let arc = Arc::new(c_reader);
                        if self.active_commentary.is_none() {
                            self.active_commentary = Some(abbrev.to_string());
                        }
                        self.commentaries.insert(abbrev.to_string(), arc);
                    } else if let Ok(d_reader) = DictionaryReader::open(path, abbrev) {
                        let arc = Arc::new(d_reader);
                        self.lookup_engine.add_dictionary(arc.clone());
                        self.dictionaries.insert(abbrev.to_string(), arc);
                    } else if let Ok(x_reader) = CrossReferenceReader::open(path, abbrev) {
                        let arc = Arc::new(x_reader);
                        if self.active_cross_references.is_none() {
                            self.active_cross_references = Some(abbrev.to_string());
                        }
                        self.cross_references.insert(abbrev.to_string(), arc);
                    } else {
                        return Err(MyBibleError::MissingTable {
                            table: "verses".to_string(),
                            module: abbrev.to_string(),
                        });
                    }
                }
                Err(e) => return Err(e),
            },
            ModuleType::Commentaries => {
                let reader = Arc::new(CommentaryReader::open(path, abbrev)?);
                if self.active_commentary.is_none() {
                    self.active_commentary = Some(abbrev.to_string());
                }
                self.commentaries.insert(abbrev.to_string(), reader);
            }
            ModuleType::Dictionary => {
                let reader = Arc::new(DictionaryReader::open(path, abbrev)?);
                self.lookup_engine.add_dictionary(reader.clone());
                self.dictionaries.insert(abbrev.to_string(), reader);
            }
            ModuleType::CrossReferences => {
                let reader = Arc::new(CrossReferenceReader::open(path, abbrev)?);
                if self.active_cross_references.is_none() {
                    self.active_cross_references = Some(abbrev.to_string());
                }
                self.cross_references.insert(abbrev.to_string(), reader);
            }
            ModuleType::Subheadings => {
                let reader = Arc::new(SubheadingReader::open(path, abbrev)?);
                if self.active_subheadings.is_none() {
                    self.active_subheadings = Some(abbrev.to_string());
                }
                self.subheadings.insert(abbrev.to_string(), reader);
            }
            ModuleType::ReadingPlan => {
                let reader = Arc::new(ReadingPlanReader::open(path, abbrev)?);
                self.reading_plans.insert(abbrev.to_string(), reader);
            }
            ModuleType::Devotions => {
                let reader = Arc::new(DevotionReader::open(path, abbrev)?);
                self.devotions.insert(abbrev.to_string(), reader);
            }
            ModuleType::Bundle => {
                // Handled in ZIP archive extraction
            }
        }
        Ok(())
    }

    pub fn get_bible(&self, abbrev: &str) -> Option<Arc<BibleReader>> {
        self.bibles.get(abbrev).cloned()
    }

    pub fn get_active_bible(&self) -> Option<Arc<BibleReader>> {
        let name = self.active_bible.as_deref()?;
        self.bibles.get(name).cloned()
    }

    pub fn get_commentary(&self, abbrev: &str) -> Option<Arc<CommentaryReader>> {
        self.commentaries.get(abbrev).cloned()
    }

    pub fn get_active_commentary(&self) -> Option<Arc<CommentaryReader>> {
        let name = self.active_commentary.as_deref()?;
        self.commentaries.get(name).cloned()
    }

    /// Retrieve a comprehensive composite verse combining text, active subheadings, cross-references, and footnotes.
    pub fn read_composite_verse(
        &self,
        bible_abbrev: Option<&str>,
        book: impl Into<BookNumber>,
        chapter: u32,
        verse: u32,
    ) -> Result<CompositeVerse> {
        let b = book.into();
        let bible = match bible_abbrev {
            Some(abbrev) => self.get_bible(abbrev).ok_or_else(|| {
                MyBibleError::InvalidFormat(format!("Bible '{}' not loaded", abbrev))
            })?,
            None => self
                .get_active_bible()
                .ok_or_else(|| MyBibleError::InvalidFormat("No active Bible loaded".to_string()))?,
        };

        let verse_rec =
            bible
                .verse(b, chapter, verse)?
                .ok_or_else(|| MyBibleError::VerseNotFound {
                    book: b.0,
                    chapter,
                    verse,
                })?;

        // 1. Subheadings: from active subheadings module, or from bible module's stories table
        let mut subheadings = Vec::new();
        if let Some(ref sub_name) = self.active_subheadings {
            if let Some(sub_reader) = self.subheadings.get(sub_name) {
                subheadings = sub_reader.subheadings_for_verse(b, chapter, verse)?;
            }
        }
        if subheadings.is_empty() {
            let stories = bible.stories_for_verse(b, chapter, verse)?;
            for s in stories {
                subheadings.push(SubheadingRecord {
                    book_number: s.book_number,
                    chapter: s.chapter,
                    verse: s.verse,
                    order_if_several: s.order_if_several,
                    subheading: s.title,
                });
            }
        }

        // 2. Cross references
        let mut cross_refs = Vec::new();
        if let Some(ref cr_name) = self.active_cross_references {
            if let Some(cr_reader) = self.cross_references.get(cr_name) {
                cross_refs = cr_reader.references_for_verse(b, chapter, verse)?;
            }
        }

        // 3. Commentaries & Footnotes
        let mut footnotes = Vec::new();
        let mut commentaries = Vec::new();

        if let Some(ref comm_name) = self.active_commentary {
            if let Some(comm_reader) = self.commentaries.get(comm_name) {
                let recs = comm_reader.commentaries_for_verse(b, chapter, verse)?;
                if comm_reader.is_footnotes() {
                    footnotes = recs;
                } else {
                    commentaries = recs;
                }
            }
        }

        // Also check if there is a same-abbreviation commentary carrying footnotes (e.g. KJV and KJV.commentaries)
        if footnotes.is_empty() {
            if let Some(comm_reader) = self.commentaries.get(bible.abbreviation()) {
                if comm_reader.is_footnotes() {
                    footnotes = comm_reader.commentaries_for_verse(b, chapter, verse)?;
                }
            }
        }

        Ok(CompositeVerse {
            verse: verse_rec,
            subheadings,
            cross_references: cross_refs,
            footnotes,
            commentaries,
        })
    }

    /// Retrieve all composite verses in a chapter.
    pub fn read_composite_chapter(
        &self,
        bible_abbrev: Option<&str>,
        book: impl Into<BookNumber>,
        chapter: u32,
    ) -> Result<CompositeChapter> {
        let b = book.into();
        let bible = match bible_abbrev {
            Some(abbrev) => self.get_bible(abbrev).ok_or_else(|| {
                MyBibleError::InvalidFormat(format!("Bible '{}' not loaded", abbrev))
            })?,
            None => self
                .get_active_bible()
                .ok_or_else(|| MyBibleError::InvalidFormat("No active Bible loaded".to_string()))?,
        };

        let book_rec = bible.book(b)?.unwrap_or_else(|| BookRecord {
            book_number: b,
            book_color: "#ccccff".to_string(),
            short_name: format!("Book{}", b.0),
            title: None,
            long_name: format!("Book {}", b.0),
            is_present: true,
            sorting_order: None,
        });

        let verses = bible.verses_in_chapter(b, chapter)?;
        let mut composite_verses = Vec::with_capacity(verses.len());

        for v in verses {
            let comp = self.read_composite_verse(bible_abbrev, b, chapter, v.verse)?;
            composite_verses.push(comp);
        }

        Ok(CompositeChapter {
            book: book_rec,
            chapter,
            verses: composite_verses,
        })
    }

    /// Render HTML content by expanding fragments across modules and applying theme colors.
    pub fn render_html(&self, html: &str, current_module: Option<&str>) -> Result<String> {
        let expanded = expand_fragments(html, current_module, self)?;
        Ok(apply_theme_colors(&expanded, &self.theme))
    }

    /// Performs an advanced Bible search across one, multiple, or all loaded Bible modules.
    ///
    /// - If `modules` is empty: searches the active Bible, or first available Bible if none is active.
    /// - If `modules` contains "all" or "*": searches all loaded Bibles.
    /// - Otherwise: searches the specific Bible abbreviations specified in `modules`.
    pub fn search(&self, query: &SearchQuery, modules: &[&str]) -> Result<SearchResults> {
        let start = std::time::Instant::now();

        // 1. Resolve which Bible modules to search
        let target_bibles: Vec<Arc<BibleReader>> = if modules
            .iter()
            .any(|&m| m.eq_ignore_ascii_case("all") || m == "*")
        {
            let mut list: Vec<_> = self.bibles.values().cloned().collect();
            list.sort_by(|a, b| a.abbreviation().cmp(b.abbreviation()));
            list
        } else if !modules.is_empty() {
            let mut list = Vec::new();
            for &m in modules {
                let trimmed = m.trim();
                let reader = self.get_bible(trimmed).ok_or_else(|| {
                    MyBibleError::ModuleNotFound(format!(
                        "Bible module '{}' is not loaded",
                        trimmed
                    ))
                })?;
                list.push(reader);
            }
            list
        } else if let Some(active) = self.get_active_bible() {
            vec![active]
        } else if let Some(first) = self.bibles.values().next() {
            vec![first.clone()]
        } else {
            return Err(MyBibleError::ModuleNotFound(
                "No Bible modules are loaded in the engine".to_string(),
            ));
        };

        // 2. Perform search across target Bibles
        let mut all_hits = Vec::new();
        let mut total_matches = 0;
        let mut matches_by_module = HashMap::new();

        for bible in &target_bibles {
            let (hits, count) = bible.search_with_total(query)?;
            matches_by_module.insert(bible.abbreviation().to_string(), count);
            total_matches += count;
            all_hits.extend(hits);
        }

        // 3. Sort hits in canonical order: (book_number, chapter, verse, bible_module)
        all_hits.sort_by(|a, b| {
            a.book_number
                .0
                .cmp(&b.book_number.0)
                .then_with(|| a.chapter.cmp(&b.chapter))
                .then_with(|| a.verse.cmp(&b.verse))
                .then_with(|| a.bible_module.cmp(&b.bible_module))
        });

        // 4. Truncate to limit if specified
        if let Some(limit) = query.limit {
            if all_hits.len() > limit {
                all_hits.truncate(limit);
            }
        }

        let duration_ms = start.elapsed().as_millis();

        Ok(SearchResults {
            query: query.clone(),
            total_matches,
            matches_by_module,
            hits: all_hits,
            duration_ms,
        })
    }
}

impl FragmentResolver for MyBibleEngine {
    fn get_fragment(&self, fragment_id: &str, module: Option<&str>) -> Option<String> {
        if let Some(mod_name) = module {
            let clean_name = mod_name
                .trim_end_matches(".SQLite3")
                .trim_end_matches(".sqlite3");
            let base_abbrev = clean_name
                .strip_suffix(".commentaries")
                .or_else(|| clean_name.strip_suffix(".dictionary"))
                .or_else(|| clean_name.strip_suffix(".subheadings"))
                .or_else(|| clean_name.strip_suffix(".crossreferences"))
                .or_else(|| clean_name.strip_suffix(".plan"))
                .or_else(|| clean_name.strip_suffix(".devotions"))
                .unwrap_or(clean_name);

            for name in [clean_name, base_abbrev] {
                if let Some(c) = self.commentaries.get(name) {
                    if let Some(frag) = c.get_fragment(fragment_id, None) {
                        return Some(frag);
                    }
                }
                if let Some(b) = self.bibles.get(name) {
                    if let Some(frag) = b.get_fragment(fragment_id, None) {
                        return Some(frag);
                    }
                }
                if let Some(d) = self.dictionaries.get(name) {
                    if let Some(frag) = d.get_fragment(fragment_id, None) {
                        return Some(frag);
                    }
                }
                if let Some(dev) = self.devotions.get(name) {
                    if let Some(frag) = dev.get_fragment(fragment_id, None) {
                        return Some(frag);
                    }
                }
            }
        }

        // Fallback: search across all loaded modules
        for b in self.bibles.values() {
            if let Some(frag) = b.get_fragment(fragment_id, None) {
                return Some(frag);
            }
        }
        for c in self.commentaries.values() {
            if let Some(frag) = c.get_fragment(fragment_id, None) {
                return Some(frag);
            }
        }
        for d in self.dictionaries.values() {
            if let Some(frag) = d.get_fragment(fragment_id, None) {
                return Some(frag);
            }
        }
        for dev in self.devotions.values() {
            if let Some(frag) = dev.get_fragment(fragment_id, None) {
                return Some(frag);
            }
        }

        None
    }
}
