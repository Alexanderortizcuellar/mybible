use crate::error::{MyBibleError, Result};
use crate::metadata::InfoTable;
use crate::models::subheading::SubheadingRecord;
use crate::readers::common::{open_sqlite_readonly, read_info_table, table_exists};
use crate::types::BookNumber;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Reader for a MyBible subheadings module (`.subheadings.SQLite3`).
pub struct SubheadingReader {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    abbreviation: String,
    info: InfoTable,
}

impl SubheadingReader {
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
        if !table_exists(&conn, "subheadings")? {
            return Err(MyBibleError::MissingTable {
                table: "subheadings".to_string(),
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

    /// Retrieve subheadings that precede the given verse.
    pub fn subheadings_for_verse(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
        verse: u32,
    ) -> Result<Vec<SubheadingRecord>> {
        let b = book.into().0;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT order_if_several, subheading FROM subheadings WHERE book_number = ?1 AND chapter = ?2 AND verse = ?3 ORDER BY order_if_several",
        )?;
        let mut rows = stmt.query([b as u32, chapter, verse])?;
        let mut list = Vec::new();

        while let Some(row) = rows.next()? {
            let order: Option<u32> = row.get(0)?;
            let text: String = row.get(1)?;
            list.push(SubheadingRecord {
                book_number: BookNumber(b),
                chapter,
                verse,
                order_if_several: order,
                subheading: text,
            });
        }

        Ok(list)
    }

    /// Retrieve all subheadings in a given chapter.
    pub fn subheadings_in_chapter(
        &self,
        book: impl Into<BookNumber>,
        chapter: u32,
    ) -> Result<Vec<SubheadingRecord>> {
        let b = book.into().0;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT verse, order_if_several, subheading FROM subheadings WHERE book_number = ?1 AND chapter = ?2 ORDER BY verse, order_if_several",
        )?;
        let mut rows = stmt.query([b as u32, chapter])?;
        let mut list = Vec::new();

        while let Some(row) = rows.next()? {
            let verse: u32 = row.get(0)?;
            let order: Option<u32> = row.get(1)?;
            let text: String = row.get(2)?;
            list.push(SubheadingRecord {
                book_number: BookNumber(b),
                chapter,
                verse,
                order_if_several: order,
                subheading: text,
            });
        }

        Ok(list)
    }
}
