use crate::error::{MyBibleError, Result};
use crate::metadata::InfoTable;
use crate::models::bible::{BookRecord, StoryRecord, VerseRecord};
use crate::models::morphology::{MorphologyIndicationRecord, MorphologyTopicRecord};
use crate::parsers::fragments::FragmentResolver;
use crate::readers::common::{
    column_exists, open_sqlite_readonly, read_content_fragments, read_info_table, table_exists,
};
use crate::types::{BookNumber, VerseRange};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Reader for a MyBible Bible module (`.SQLite3`).
pub struct BibleReader {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    abbreviation: String,
    info: InfoTable,
    fragments: HashMap<String, String>,
}

impl BibleReader {
    /// Opens a Bible module database from a file path.
    pub fn open(path: impl AsRef<Path>, abbreviation: &str) -> Result<Self> {
        let p = path.as_ref();
        let conn = open_sqlite_readonly(p)?;
        Self::from_connection(conn, Some(p.to_path_buf()), abbreviation)
    }

    /// Initializes a Bible module from an in-memory SQLite connection.
    pub fn from_connection(
        conn: Connection,
        path: Option<PathBuf>,
        abbreviation: &str,
    ) -> Result<Self> {
        if !table_exists(&conn, "verses")? {
            return Err(MyBibleError::MissingTable {
                table: "verses".to_string(),
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

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Return all books defined in the module.
    /// Prefers the `books_all` table if present; falls back to `books`.
    pub fn books(&self) -> Result<Vec<BookRecord>> {
        let (has_all, has_b) = {
            let conn = self.conn.lock().unwrap();
            (
                table_exists(&conn, "books_all")?,
                table_exists(&conn, "books")?,
            )
        };

        if has_all {
            self.read_books_all()
        } else if has_b {
            self.read_books()
        } else {
            Ok(Vec::new())
        }
    }

    /// Return only books that are marked as present (or all in `books`).
    pub fn present_books(&self) -> Result<Vec<BookRecord>> {
        let all = self.books()?;
        Ok(all.into_iter().filter(|b| b.is_present).collect())
    }

    /// Lookup a single book record by its number.
    pub fn book(&self, book_number: impl Into<BookNumber>) -> Result<Option<BookRecord>> {
        let num: u16 = book_number.into().0;
        let books = self.books()?;
        Ok(books.into_iter().find(|b| b.book_number.0 == num))
    }

    fn read_books_all(&self) -> Result<Vec<BookRecord>> {
        let conn = self.conn.lock().unwrap();
        let has_sorting = column_exists(&conn, "books_all", "sorting_order")?;
        let has_title = column_exists(&conn, "books_all", "title")?;
        let has_color = column_exists(&conn, "books_all", "book_color")?;
        let has_present = column_exists(&conn, "books_all", "is_present")?;

        let sql =
            format!(
            "SELECT book_number, {}, short_name, {}, long_name, {}, {} FROM books_all ORDER BY {}",
            if has_color { "book_color" } else { "''" },
            if has_title { "title" } else { "NULL" },
            if has_present { "is_present" } else { "1" },
            if has_sorting { "sorting_order" } else { "NULL" },
            if has_sorting { "COALESCE(sorting_order, book_number)" } else { "book_number" }
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut books = Vec::new();

        while let Some(row) = rows.next()? {
            let num: u16 = row.get(0)?;
            let book_color: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            let short_name: String = row.get(2)?;
            let title: Option<String> = row.get(3)?;
            let long_name: String = row.get(4)?;
            let is_present_raw: Option<i32> = row.get(5)?;
            let is_present = is_present_raw.unwrap_or(1) != 0;
            let sorting_order: Option<u32> = row.get(6)?;

            books.push(BookRecord {
                book_number: BookNumber(num),
                book_color,
                short_name,
                title,
                long_name,
                is_present,
                sorting_order,
            });
        }

        Ok(books)
    }

    fn read_books(&self) -> Result<Vec<BookRecord>> {
        let conn = self.conn.lock().unwrap();
        let has_sorting = column_exists(&conn, "books", "sorting_order")?;
        let has_color = column_exists(&conn, "books", "book_color")?;

        let sql = format!(
            "SELECT book_number, {}, short_name, long_name, {} FROM books ORDER BY {}",
            if has_color { "book_color" } else { "''" },
            if has_sorting { "sorting_order" } else { "NULL" },
            if has_sorting {
                "COALESCE(sorting_order, book_number)"
            } else {
                "book_number"
            }
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut books = Vec::new();

        while let Some(row) = rows.next()? {
            let num: u16 = row.get(0)?;
            let book_color: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            let short_name: String = row.get(2)?;
            let long_name: String = row.get(3)?;
            let sorting_order: Option<u32> = row.get(4)?;

            books.push(BookRecord {
                book_number: BookNumber(num),
                book_color,
                short_name,
                title: None,
                long_name,
                is_present: true,
                sorting_order,
            });
        }

        Ok(books)
    }

    /// Retrieve a specific verse.
    pub fn verse(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
        verse: u32,
    ) -> Result<Option<VerseRecord>> {
        let b = book.into().0;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT text FROM verses WHERE book_number = ?1 AND chapter = ?2 AND verse = ?3",
        )?;
        let mut rows = stmt.query([b as u32, chapter, verse])?;

        if let Some(row) = rows.next()? {
            let text: String = row.get(0)?;
            Ok(Some(VerseRecord {
                book_number: BookNumber(b),
                chapter,
                verse,
                text,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieve all verses in a given chapter.
    pub fn verses_in_chapter(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
    ) -> Result<Vec<VerseRecord>> {
        let b = book.into().0;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT verse, text FROM verses WHERE book_number = ?1 AND chapter = ?2 ORDER BY verse",
        )?;
        let mut rows = stmt.query([b as u32, chapter])?;
        let mut verses = Vec::new();

        while let Some(row) = rows.next()? {
            let verse_num: u32 = row.get(0)?;
            let text: String = row.get(1)?;
            verses.push(VerseRecord {
                book_number: BookNumber(b),
                chapter,
                verse: verse_num,
                text,
            });
        }

        Ok(verses)
    }

    /// Retrieve verses within a specified range.
    pub fn verses_in_range(&self, range: &VerseRange) -> Result<Vec<VerseRecord>> {
        let b = range.book.0;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT chapter, verse, text FROM verses WHERE book_number = ?1 AND ((chapter = ?2 AND verse >= ?3) OR (chapter > ?2 AND chapter < ?4) OR (chapter = ?4 AND verse <= ?5)) ORDER BY chapter, verse"
        )?;
        let mut rows = stmt.query([
            b as u32,
            range.start_chapter,
            range.start_verse,
            range.end_chapter,
            range.end_verse,
        ])?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let ch: u32 = row.get(0)?;
            let v: u32 = row.get(1)?;
            let text: String = row.get(2)?;
            results.push(VerseRecord {
                book_number: BookNumber(b),
                chapter: ch,
                verse: v,
                text,
            });
        }
        Ok(results)
    }

    /// Get total number of chapters in a book.
    pub fn chapter_count(&self, book: impl Into<BookNumber>) -> Result<u32> {
        let b = book.into().0;
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT COALESCE(MAX(chapter), 0) FROM verses WHERE book_number = ?1")?;
        let count: u32 = stmt.query_row([b as u32], |r| r.get(0))?;
        Ok(count)
    }

    /// Get total number of verses in a book and chapter.
    pub fn verse_count(&self, book: impl Into<BookNumber>, chapter: u32) -> Result<u32> {
        let b = book.into().0;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COALESCE(MAX(verse), 0) FROM verses WHERE book_number = ?1 AND chapter = ?2",
        )?;
        let count: u32 = stmt.query_row([b as u32, chapter], |r| r.get(0))?;
        Ok(count)
    }

    /// Retrieve module introduction (book_number == 0).
    pub fn module_introduction(&self) -> Result<Option<String>> {
        self.book_introduction(0)
    }

    /// Retrieve introduction for a book (or module if 0).
    pub fn book_introduction(&self, book: impl Into<BookNumber>) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "introductions")? {
            return Ok(None);
        }
        let b = book.into().0;
        let mut stmt =
            conn.prepare("SELECT introduction FROM introductions WHERE book_number = ?1")?;
        let mut rows = stmt.query([b as u32])?;
        if let Some(row) = rows.next()? {
            let intro: String = row.get(0)?;
            Ok(Some(intro))
        } else {
            Ok(None)
        }
    }

    /// Retrieve subheadings/stories from the `stories` table for a verse.
    pub fn stories_for_verse(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
        verse: u32,
    ) -> Result<Vec<StoryRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "stories")? {
            return Ok(Vec::new());
        }
        let b = book.into().0;
        let mut stmt = conn.prepare(
            "SELECT order_if_several, title FROM stories WHERE book_number = ?1 AND chapter = ?2 AND verse = ?3 ORDER BY order_if_several",
        )?;
        let mut rows = stmt.query([b as u32, chapter, verse])?;
        let mut stories = Vec::new();
        while let Some(row) = rows.next()? {
            let order: Option<u32> = row.get(0)?;
            let title: String = row.get(1)?;
            stories.push(StoryRecord {
                book_number: BookNumber(b),
                chapter,
                verse,
                order_if_several: order,
                title,
            });
        }
        Ok(stories)
    }

    /// Retrieve subheadings/stories for an entire chapter.
    pub fn stories_in_chapter(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
    ) -> Result<Vec<StoryRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "stories")? {
            return Ok(Vec::new());
        }
        let b = book.into().0;
        let mut stmt = conn.prepare(
            "SELECT verse, order_if_several, title FROM stories WHERE book_number = ?1 AND chapter = ?2 ORDER BY verse, order_if_several",
        )?;
        let mut rows = stmt.query([b as u32, chapter])?;
        let mut stories = Vec::new();
        while let Some(row) = rows.next()? {
            let verse: u32 = row.get(0)?;
            let order: Option<u32> = row.get(1)?;
            let title: String = row.get(2)?;
            stories.push(StoryRecord {
                book_number: BookNumber(b),
                chapter,
                verse,
                order_if_several: order,
                title,
            });
        }
        Ok(stories)
    }

    /// Retrieve morphology indications embedded in this Bible module.
    pub fn morphology_indications(&self) -> Result<Vec<MorphologyIndicationRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "morphology_indications")? {
            return Ok(Vec::new());
        }
        let mut stmt = conn.prepare(
            "SELECT indication, applicable_to, language, meaning FROM morphology_indications",
        )?;
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

    /// Retrieve morphology topics mapping from this Bible module.
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

    /// Search for text substring across all verses.
    pub fn search_text(&self, query: &str, limit: usize) -> Result<Vec<VerseRecord>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT book_number, chapter, verse, text FROM verses WHERE text LIKE ?1 LIMIT ?2",
        )?;
        let mut rows = stmt.query([&pattern, &(limit as i64).to_string()])?;
        let mut verses = Vec::new();
        while let Some(row) = rows.next()? {
            let b: u16 = row.get(0)?;
            let ch: u32 = row.get(1)?;
            let v: u32 = row.get(2)?;
            let text: String = row.get(3)?;
            verses.push(VerseRecord {
                book_number: BookNumber(b),
                chapter: ch,
                verse: v,
                text,
            });
        }
        Ok(verses)
    }
}

impl FragmentResolver for BibleReader {
    fn get_fragment(&self, fragment_id: &str, module: Option<&str>) -> Option<String> {
        if module.is_none() || module == Some(self.abbreviation()) {
            self.fragments.get(fragment_id).cloned()
        } else {
            None
        }
    }
}
