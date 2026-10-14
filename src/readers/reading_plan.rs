use crate::error::{MyBibleError, Result};
use crate::metadata::InfoTable;
use crate::models::reading_plan::ReadingPlanRecord;
use crate::readers::common::{open_sqlite_readonly, read_info_table, table_exists};
use crate::types::BookNumber;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Reader for a MyBible reading plan module (`.plan.SQLite3`).
pub struct ReadingPlanReader {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    abbreviation: String,
    info: InfoTable,
}

impl ReadingPlanReader {
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
        if !table_exists(&conn, "reading_plan")? {
            return Err(MyBibleError::MissingTable {
                table: "reading_plan".to_string(),
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

    /// Retrieve the maximum day number (duration of the plan in days).
    pub fn max_days(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COALESCE(MAX(day), 0) FROM reading_plan")?;
        let days: u32 = stmt.query_row([], |r| r.get(0))?;
        Ok(days)
    }

    /// Retrieve all reading items for a specific day sorted by item.
    pub fn items_for_day(&self, day: u32) -> Result<Vec<ReadingPlanRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT evening, item, book_number, start_chapter, start_verse, end_chapter, end_verse FROM reading_plan WHERE day = ?1 ORDER BY evening, item",
        )?;
        let mut rows = stmt.query([day])?;
        let mut list = Vec::new();

        while let Some(row) = rows.next()? {
            let evening_raw: i32 = row.get(0)?;
            let item: u32 = row.get(1)?;
            let book: u16 = row.get(2)?;
            let start_ch: u32 = row.get(3)?;
            let start_v: Option<u32> = row.get(4)?;
            let end_ch: u32 = row.get(5)?;
            let end_v: Option<u32> = row.get(6)?;

            list.push(ReadingPlanRecord {
                day,
                evening: evening_raw != 0,
                item,
                book_number: BookNumber(book),
                start_chapter: start_ch,
                start_verse: start_v.filter(|&v| v > 0),
                end_chapter: end_ch,
                end_verse: end_v.filter(|&v| v > 0),
            });
        }

        Ok(list)
    }
}
