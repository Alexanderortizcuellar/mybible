use crate::error::Result;
use crate::metadata::InfoTable;
use crate::models::dictionary::{
    DictionaryRecord, WordRecord, WordsProcessingRecord, WordsProcessingType,
};
use crate::models::morphology::{MorphologyIndicationRecord, MorphologyTopicRecord};
use crate::parsers::fragments::FragmentResolver;
use crate::readers::common::{
    column_exists, open_sqlite_readonly, read_content_fragments, read_info_table, table_exists,
};
use crate::types::{BookNumber, DictionaryType, MatchingType};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Reader for a MyBible dictionary or lexicon module (`.dictionary.SQLite3`).
pub struct DictionaryReader {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    abbreviation: String,
    info: InfoTable,
    fragments: HashMap<String, String>,
}

impl DictionaryReader {
    pub fn open(path: impl AsRef<Path>, abbreviation: &str) -> Result<Self> {
        let p = path.as_ref();
        let conn = open_sqlite_readonly(p)?;
        Self::from_connection(conn, Some(p.to_path_buf()), abbreviation)
    }

    pub fn from_connection(
        conn: Connection,
        path: Option<PathBuf>,
        abbreviation: &str,
    ) -> Result<Self> {
        if !table_exists(&conn, "dictionary")? {
            return Err(crate::error::MyBibleError::MissingTable {
                table: "dictionary".to_string(),
                module: abbreviation.to_string(),
            });
        }

        let info = read_info_table(&conn)?;
        let fragments = read_content_fragments(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            path,
            abbreviation: abbreviation.to_string(),
            info,
            fragments,
        })
    }

    pub fn abbreviation(&self) -> &str {
        &self.abbreviation
    }

    pub fn info(&self) -> &InfoTable {
        &self.info
    }

    pub fn dictionary_type(&self) -> DictionaryType {
        self.info.dictionary_type()
    }

    pub fn matching_type(&self) -> MatchingType {
        self.info.standard_form_matching_type()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Check if the `dictionary` table exists in this module.
    pub fn has_dictionary_table(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        table_exists(&conn, "dictionary")
    }

    /// Retrieve an article by topic exact match.
    pub fn get_topic(&self, topic: &str) -> Result<Option<DictionaryRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "dictionary")? {
            return Ok(None);
        }

        let has_short = column_exists(&conn, "dictionary", "short_definition")?;
        let has_lexeme = column_exists(&conn, "dictionary", "lexeme")?;
        let has_trans = column_exists(&conn, "dictionary", "transliteration")?;
        let has_pron = column_exists(&conn, "dictionary", "pronunciation")?;

        let sql = format!(
            "SELECT definition, {}, {}, {}, {} FROM dictionary WHERE topic = ?1",
            if has_short {
                "short_definition"
            } else {
                "NULL"
            },
            if has_lexeme { "lexeme" } else { "NULL" },
            if has_trans { "transliteration" } else { "NULL" },
            if has_pron { "pronunciation" } else { "NULL" },
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([topic])?;

        if let Some(row) = rows.next()? {
            let definition: String = row.get(0)?;
            let short_definition: Option<String> = row.get(1)?;
            let lexeme: Option<String> = row.get(2)?;
            let transliteration: Option<String> = row.get(3)?;
            let pronunciation: Option<String> = row.get(4)?;

            Ok(Some(DictionaryRecord {
                topic: topic.to_string(),
                definition,
                short_definition,
                lexeme,
                transliteration,
                pronunciation,
            }))
        } else {
            Ok(None)
        }
    }

    /// Find topics starting with the given prefix.
    pub fn search_topics_prefix(&self, prefix: &str, limit: usize) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "dictionary")? {
            return Ok(Vec::new());
        }
        let pattern = format!("{}%", prefix);
        let mut stmt = conn
            .prepare("SELECT topic FROM dictionary WHERE topic LIKE ?1 ORDER BY topic LIMIT ?2")?;
        let mut rows = stmt.query([&pattern, &(limit as i64).to_string()])?;
        let mut topics = Vec::new();

        while let Some(row) = rows.next()? {
            let t: String = row.get(0)?;
            topics.push(t);
        }

        Ok(topics)
    }

    /// Retrieve all topics from the dictionary.
    pub fn all_topics(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "dictionary")? {
            return Ok(Vec::new());
        }
        let mut stmt = conn.prepare("SELECT topic FROM dictionary ORDER BY topic")?;
        let mut rows = stmt.query([])?;
        let mut topics = Vec::new();
        while let Some(row) = rows.next()? {
            topics.push(row.get(0)?);
        }
        Ok(topics)
    }

    /// Retrieve all word variation records from the `words` table.
    pub fn words(&self) -> Result<Vec<WordRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "words")? {
            return Ok(Vec::new());
        }

        let has_book = column_exists(&conn, "words", "book_number")?;
        let sql = if has_book {
            "SELECT standard_form, variation, book_number, chapter_number, verse_number FROM words"
        } else {
            "SELECT standard_form, variation, 0, 0, 0 FROM words"
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query([])?;
        let mut words = Vec::new();

        while let Some(row) = rows.next()? {
            let standard_form: String = row.get(0)?;
            let variation: String = row.get(1)?;
            let book: u16 = row.get(2)?;
            let ch: u32 = row.get(3)?;
            let v: u32 = row.get(4)?;

            words.push(WordRecord {
                standard_form,
                variation,
                book_number: if book > 0 {
                    Some(BookNumber(book))
                } else {
                    None
                },
                chapter_number: if ch > 0 { Some(ch) } else { None },
                verse_number: if v > 0 { Some(v) } else { None },
            });
        }

        Ok(words)
    }

    /// Retrieve word processing rules from `words_processing` table.
    pub fn words_processing(&self) -> Result<Vec<WordsProcessingRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "words_processing")? {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare("SELECT type, input, output FROM words_processing")?;
        let mut rows = stmt.query([])?;
        let mut list = Vec::new();

        while let Some(row) = rows.next()? {
            let t: String = row.get(0)?;
            if let Some(proc_type) = WordsProcessingType::from_str_info(&t) {
                let input: String = row.get(1)?;
                let output: String = row.get(2)?;
                list.push(WordsProcessingRecord {
                    proc_type,
                    input,
                    output,
                });
            }
        }

        Ok(list)
    }

    /// Retrieve cognate Strong's numbers for the given Strong number.
    pub fn cognates(&self, strong_number: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "cognate_strong_numbers")? {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare(
            "SELECT strong_number FROM cognate_strong_numbers WHERE group_id IN (SELECT group_id FROM cognate_strong_numbers WHERE strong_number = ?1) AND strong_number <> ?1",
        )?;
        let mut rows = stmt.query([strong_number])?;
        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            list.push(row.get(0)?);
        }
        Ok(list)
    }

    /// Retrieve synonymous Strong's numbers for the given Strong number.
    pub fn synonyms(&self, strong_number: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "synonymous_strong_numbers")? {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare(
            "SELECT strong_number FROM synonymous_strong_numbers WHERE group_id IN (SELECT group_id FROM synonymous_strong_numbers WHERE strong_number = ?1) AND strong_number <> ?1",
        )?;
        let mut rows = stmt.query([strong_number])?;
        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            list.push(row.get(0)?);
        }
        Ok(list)
    }

    /// Retrieve morphology indications from `morphology_indications` table.
    pub fn morphology_indications(&self) -> Result<Vec<MorphologyIndicationRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "morphology_indications")? {
            return Ok(Vec::new());
        }

        let has_lang = column_exists(&conn, "morphology_indications", "language")?;
        let sql = if has_lang {
            "SELECT indication, applicable_to, language, meaning FROM morphology_indications"
        } else {
            "SELECT indication, applicable_to, NULL, meaning FROM morphology_indications"
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query([])?;
        let mut records = Vec::new();

        while let Some(row) = rows.next()? {
            records.push(MorphologyIndicationRecord {
                indication: row.get(0)?,
                applicable_to: row.get(1)?,
                language: row.get(2)?,
                meaning: row.get(3)?,
            });
        }

        Ok(records)
    }

    /// Retrieve morphology topic mapping from `morphology_topics` table.
    pub fn morphology_topics(&self) -> Result<Vec<MorphologyTopicRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "morphology_topics")? {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare("SELECT indication, topic FROM morphology_topics")?;
        let mut rows = stmt.query([])?;
        let mut records = Vec::new();

        while let Some(row) = rows.next()? {
            records.push(MorphologyTopicRecord {
                indication: row.get(0)?,
                topic: row.get(1)?,
            });
        }

        Ok(records)
    }
}

impl FragmentResolver for DictionaryReader {
    fn get_fragment(&self, fragment_id: &str, module: Option<&str>) -> Option<String> {
        if module.is_none() || module == Some(self.abbreviation()) {
            self.fragments.get(fragment_id).cloned()
        } else {
            None
        }
    }
}
