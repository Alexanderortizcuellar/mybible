use crate::error::{MyBibleError, Result};
use crate::metadata::InfoTable;
use crate::models::commentary::{CommentaryBookRecord, CommentaryRecord};
use crate::parsers::fragments::FragmentResolver;
use crate::readers::common::{
    column_exists, open_sqlite_readonly, read_content_fragments, read_info_table, table_exists,
};
use crate::types::BookNumber;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Reader for a MyBible commentary module (`.commentaries.SQLite3`).
pub struct CommentaryReader {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    abbreviation: String,
    info: InfoTable,
    fragments: HashMap<String, String>,
}

impl CommentaryReader {
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
        if !table_exists(&conn, "commentaries")? {
            return Err(MyBibleError::MissingTable {
                table: "commentaries".to_string(),
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

    pub fn is_footnotes(&self) -> bool {
        self.info.is_footnotes()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Retrieve all commentary records relevant to a given verse.
    pub fn commentaries_for_verse(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
        verse: u32,
    ) -> Result<Vec<CommentaryRecord>> {
        let b = book.into().0;
        let all_for_book = self.commentaries_in_book(b)?;
        let mut matches = Vec::new();
        for rec in all_for_book {
            if rec.applies_to_verse(BookNumber(b), chapter, verse) {
                matches.push(rec);
            }
        }
        Ok(matches)
    }

    /// Retrieve all commentary records relevant to a chapter.
    pub fn commentaries_for_chapter(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
    ) -> Result<Vec<CommentaryRecord>> {
        let b = book.into().0;
        let all_for_book = self.commentaries_in_book(b)?;
        let mut matches = Vec::new();
        for rec in all_for_book {
            let ch_from = rec.chapter_number_from;
            let ch_to = rec.chapter_number_to.unwrap_or(ch_from);
            if ch_from == 0 || (chapter >= ch_from && chapter <= ch_to) {
                matches.push(rec);
            }
        }
        Ok(matches)
    }

    /// Retrieve a specific footnote for a verse matching the given marker (e.g. "[1]").
    pub fn commentary_for_footnote(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
        verse: u32,
        marker: &str,
    ) -> Result<Option<CommentaryRecord>> {
        let comments = self.commentaries_for_verse(book, chapter, verse)?;
        for c in comments {
            if let Some(ref m) = c.marker {
                if m.trim() == marker.trim() {
                    return Ok(Some(c));
                }
            }
        }
        Ok(None)
    }

    /// Retrieve all commentary records for a book.
    pub fn commentaries_in_book(
        &self,
        book: impl Into<BookNumber>,
    ) -> Result<Vec<CommentaryRecord>> {
        let b = book.into().0;
        let conn = self.conn.lock().unwrap();
        let has_marker = column_exists(&conn, "commentaries", "marker")?;
        let has_preceding = column_exists(&conn, "commentaries", "is_preceding")?;

        let marker_col = if has_marker { "marker" } else { "NULL" };
        let preceding_col = if has_preceding { "is_preceding" } else { "0" };

        let sql = format!(
            "SELECT chapter_number_from, verse_number_from, chapter_number_to, verse_number_to, {}, {}, text FROM commentaries WHERE book_number = ?1 ORDER BY chapter_number_from, verse_number_from",
            preceding_col, marker_col
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([b as u32])?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let ch_from: u32 = row.get(0)?;
            let v_from: u32 = row.get(1)?;
            let ch_to: Option<u32> = row.get(2)?;
            let v_to: Option<u32> = row.get(3)?;
            let is_prec_raw: Option<i32> = row.get(4)?;
            let is_preceding = is_prec_raw.unwrap_or(0) != 0;
            let marker: Option<String> = row.get(5)?;
            let text: String = row.get(6)?;

            results.push(CommentaryRecord {
                book_number: BookNumber(b),
                chapter_number_from: ch_from,
                verse_number_from: v_from,
                chapter_number_to: ch_to,
                verse_number_to: v_to,
                is_preceding,
                marker,
                text,
            });
        }

        Ok(results)
    }

    /// Retrieve optional book name mappings defined in the commentary module.
    pub fn books(&self) -> Result<Vec<CommentaryBookRecord>> {
        let conn = self.conn.lock().unwrap();
        if !table_exists(&conn, "books")? {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare("SELECT book_number, short_name FROM books")?;
        let mut rows = stmt.query([])?;
        let mut books = Vec::new();

        while let Some(row) = rows.next()? {
            let num: u16 = row.get(0)?;
            let short_name: String = row.get(1)?;
            books.push(CommentaryBookRecord {
                book_number: BookNumber(num),
                short_name,
            });
        }

        Ok(books)
    }
}

impl FragmentResolver for CommentaryReader {
    fn get_fragment(&self, fragment_id: &str, module: Option<&str>) -> Option<String> {
        if module.is_none() || module == Some(self.abbreviation()) {
            self.fragments.get(fragment_id).cloned()
        } else {
            None
        }
    }
}
