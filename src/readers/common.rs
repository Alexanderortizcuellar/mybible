use crate::error::Result;
use crate::metadata::InfoTable;
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::Path;

/// Opens an existing SQLite database in read-only mode for thread safety.
pub fn open_sqlite_readonly(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok(conn)
}

/// Checks whether a specific table exists in the database.
pub fn table_exists(conn: &Connection, table_name: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1")?;
    let exists = stmt.exists([table_name])?;
    Ok(exists)
}

/// Checks whether a specific column exists in a table.
pub fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({})", table_name);
    let mut stmt = conn.prepare(&pragma)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name.eq_ignore_ascii_case(column_name) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Reads the `info` table into an `InfoTable` struct.
pub fn read_info_table(conn: &Connection) -> Result<InfoTable> {
    if !table_exists(conn, "info")? {
        return Ok(InfoTable::default());
    }

    let mut stmt = conn.prepare("SELECT name, value FROM info")?;
    let mut rows = stmt.query([])?;
    let mut map = HashMap::new();

    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let value: String = row.get(1)?;
        map.insert(name, value);
    }

    Ok(InfoTable::new(map))
}

/// Reads all entries from the optional `content_fragments` table.
pub fn read_content_fragments(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    if !table_exists(conn, "content_fragments")? {
        return Ok(map);
    }

    let mut stmt = conn.prepare("SELECT id, fragment FROM content_fragments")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let fragment: String = row.get(1)?;
        map.insert(id, fragment);
    }

    Ok(map)
}
