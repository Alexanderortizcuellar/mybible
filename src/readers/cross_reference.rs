use crate::error::{MyBibleError, Result};
use crate::metadata::InfoTable;
use crate::models::cross_reference::CrossReferenceRecord;
use crate::readers::common::{open_sqlite_readonly, read_info_table, table_exists};
use crate::types::BookNumber;
use rusqlite::Connection;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Reader for a MyBible cross references module (`.crossreferences.SQLite3`).
pub struct CrossReferenceReader {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    abbreviation: String,
    info: InfoTable,
}

impl CrossReferenceReader {
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
        if !table_exists(&conn, "cross_references")? {
            return Err(MyBibleError::MissingTable {
                table: "cross_references".to_string(),
                module: abbreviation.to_string(),
            });
        }

        let info = read_info_table(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            path,
            abbreviation: abbreviation.to_string(),
            info,
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

    /// Retrieve all cross references for a given verse.
    /// If `requires_reverse_processing` is enabled, merges forward and reversed references,
    /// deduplicates, and sorts by votes descending.
    pub fn references_for_verse(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
        verse: u32,
    ) -> Result<Vec<CrossReferenceRecord>> {
        let b = book.into().0;
        let mut results = self.forward_references(b, chapter, verse)?;

        if self.info.requires_reverse_processing() {
            let reverse_refs = self.reverse_references(b, chapter, verse)?;
            results.extend(reverse_refs);

            // Deduplicate based on (book_to, chapter_to, verse_to_start, verse_to_end)
            let mut seen = HashSet::new();
            results.retain(|r| {
                let key = (
                    r.book_to.0,
                    r.chapter_to,
                    r.verse_to_start,
                    r.verse_to_end.unwrap_or(0),
                );
                seen.insert(key)
            });

            // Sort by votes descending (highest relevance first)
            results.sort_by_key(|a| std::cmp::Reverse(a.votes.unwrap_or(0)));
        }

        Ok(results)
    }

    fn forward_references(
        &self,
        book: u16,
        chapter: u32,
        verse: u32,
    ) -> Result<Vec<CrossReferenceRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT book, chapter, verse, verse_end, book_to, chapter_to, verse_to_start, verse_to_end, votes FROM cross_references WHERE book = ?1 AND chapter = ?2 AND (verse = ?3 OR (verse <= ?3 AND verse_end >= ?3)) ORDER BY votes DESC",
        )?;
        let mut rows = stmt.query([book as u32, chapter, verse])?;
        let mut list = Vec::new();

        while let Some(row) = rows.next()? {
            let b: u16 = row.get(0)?;
            let ch: u32 = row.get(1)?;
            let v: u32 = row.get(2)?;
            let v_end: Option<u32> = row.get(3)?;
            let b_to: u16 = row.get(4)?;
            let ch_to: u32 = row.get(5)?;
            let v_to_start: u32 = row.get(6)?;
            let v_to_end: Option<u32> = row.get(7)?;
            let votes: Option<i32> = row.get(8)?;

            list.push(CrossReferenceRecord {
                book: BookNumber(b),
                chapter: ch,
                verse: v,
                verse_end: v_end.filter(|&x| x > 0),
                book_to: BookNumber(b_to),
                chapter_to: ch_to,
                verse_to_start: v_to_start,
                verse_to_end: v_to_end.filter(|&x| x > 0),
                votes,
            });
        }

        Ok(list)
    }

    fn reverse_references(
        &self,
        book: u16,
        chapter: u32,
        verse: u32,
    ) -> Result<Vec<CrossReferenceRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT book, chapter, verse, verse_end, book_to, chapter_to, verse_to_start, verse_to_end, votes FROM cross_references WHERE book_to = ?1 AND chapter_to = ?2 AND (verse_to_start = ?3 OR (verse_to_start <= ?3 AND (verse_to_end >= ?3 OR verse_to_end = 0 OR verse_to_end IS NULL)))",
        )?;
        let mut rows = stmt.query([book as u32, chapter, verse])?;
        let mut list = Vec::new();

        while let Some(row) = rows.next()? {
            let b: u16 = row.get(0)?;
            let ch: u32 = row.get(1)?;
            let v: u32 = row.get(2)?;
            let v_end: Option<u32> = row.get(3)?;
            let b_to: u16 = row.get(4)?;
            let ch_to: u32 = row.get(5)?;
            let v_to_start: u32 = row.get(6)?;
            let v_to_end: Option<u32> = row.get(7)?;
            let votes: Option<i32> = row.get(8)?;

            let orig = CrossReferenceRecord {
                book: BookNumber(b),
                chapter: ch,
                verse: v,
                verse_end: v_end.filter(|&x| x > 0),
                book_to: BookNumber(b_to),
                chapter_to: ch_to,
                verse_to_start: v_to_start,
                verse_to_end: v_to_end.filter(|&x| x > 0),
                votes,
            };
            list.push(orig.reversed());
        }

        Ok(list)
    }
}
