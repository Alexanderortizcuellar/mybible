use crate::error::{MyBibleError, Result};
use crate::metadata::InfoTable;
use crate::models::devotion::DevotionRecord;
use crate::parsers::fragments::FragmentResolver;
use crate::readers::common::{
    open_sqlite_readonly, read_content_fragments, read_info_table, table_exists,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Reader for a MyBible daily devotions module (`.devotions.SQLite3`).
pub struct DevotionReader {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    abbreviation: String,
    info: InfoTable,
    fragments: HashMap<String, String>,
}

impl DevotionReader {
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
        if !table_exists(&conn, "devotions")? {
            return Err(MyBibleError::MissingTable {
                table: "devotions".to_string(),
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

    /// Retrieve the maximum day number in the devotions module.
    pub fn max_days(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COALESCE(MAX(day), 0) FROM devotions")?;
        let days: u32 = stmt.query_row([], |r| r.get(0))?;
        Ok(days)
    }

    /// Retrieve devotion article for a given day.
    pub fn devotion_for_day(&self, day: u32) -> Result<Option<DevotionRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT devotion FROM devotions WHERE day = ?1")?;
        let mut rows = stmt.query([day])?;

        if let Some(row) = rows.next()? {
            let devotion: String = row.get(0)?;
            Ok(Some(DevotionRecord { day, devotion }))
        } else {
            Ok(None)
        }
    }
}

impl FragmentResolver for DevotionReader {
    fn get_fragment(&self, fragment_id: &str, module: Option<&str>) -> Option<String> {
        if module.is_none() || module == Some(self.abbreviation()) {
            self.fragments.get(fragment_id).cloned()
        } else {
            None
        }
    }
}
