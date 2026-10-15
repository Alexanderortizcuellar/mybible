"""User Study Notes Database Manager for MyBible Desktop
Stores personal reflections, sermon notes, and chapter commentaries persistently in SQLite.
"""

import os
import sqlite3
from datetime import datetime
from typing import Optional


class NotesManager:
    """Manages persistent SQLite storage for user study notes."""

    def __init__(self, db_dir: str = "modules"):
        os.makedirs(db_dir, exist_ok=True)
        self.db_path = os.path.join(db_dir, "user_notes.db")
        self._init_db()

    def _get_connection(self) -> sqlite3.Connection:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        return conn

    def _init_db(self):
        with self._get_connection() as conn:
            conn.execute("""
            CREATE TABLE IF NOT EXISTS study_notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                book_number INTEGER,
                chapter INTEGER,
                verse INTEGER,
                passage_ref TEXT,
                title TEXT,
                content TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            """)
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_notes_passage ON study_notes (book_number, chapter);"
            )
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_notes_updated ON study_notes (updated_at DESC);"
            )
            conn.commit()

    def get_passage_note(self, book_number: int, chapter: int) -> Optional[dict]:
        """Fetches personal note for a specific book and chapter."""
        with self._get_connection() as conn:
            cur = conn.execute(
                """
                SELECT id, book_number, chapter, verse, passage_ref, title, content, created_at, updated_at
                FROM study_notes
                WHERE book_number = ? AND chapter = ?
                ORDER BY updated_at DESC
                LIMIT 1
                """,
                (book_number, chapter),
            )
            row = cur.fetchone()
            if row:
                return dict(row)
        return None

    def save_passage_note(
        self,
        book_number: int,
        chapter: int,
        passage_ref: str,
        content: str,
        title: Optional[str] = None,
    ) -> int:
        """Saves or updates note for active book and chapter."""
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        note_title = title or f"Notes on {passage_ref}"
        existing = self.get_passage_note(book_number, chapter)

        with self._get_connection() as conn:
            if existing:
                note_id = existing["id"]
                conn.execute(
                    """
                    UPDATE study_notes
                    SET passage_ref = ?, title = ?, content = ?, updated_at = ?
                    WHERE id = ?
                    """,
                    (passage_ref, note_title, content, now, note_id),
                )
            else:
                cur = conn.execute(
                    """
                    INSERT INTO study_notes (book_number, chapter, passage_ref, title, content, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    """,
                    (book_number, chapter, passage_ref, note_title, content, now, now),
                )
                note_id = cur.lastrowid
            conn.commit()
            return note_id

    def get_all_notes(self, search_query: str = "") -> list[dict]:
        """Lists all saved notes with optional search keyword filtering."""
        with self._get_connection() as conn:
            if search_query.strip():
                pattern = f"%{search_query.strip()}%"
                cur = conn.execute(
                    """
                    SELECT id, book_number, chapter, verse, passage_ref, title, content, created_at, updated_at
                    FROM study_notes
                    WHERE title LIKE ? OR content LIKE ? OR passage_ref LIKE ?
                    ORDER BY updated_at DESC
                    """,
                    (pattern, pattern, pattern),
                )
            else:
                cur = conn.execute(
                    """
                    SELECT id, book_number, chapter, verse, passage_ref, title, content, created_at, updated_at
                    FROM study_notes
                    ORDER BY updated_at DESC
                    """
                )
            return [dict(r) for r in cur.fetchall()]

    def get_note_by_id(self, note_id: int) -> Optional[dict]:
        with self._get_connection() as conn:
            cur = conn.execute(
                "SELECT id, book_number, chapter, verse, passage_ref, title, content, created_at, updated_at FROM study_notes WHERE id = ?",
                (note_id,),
            )
            row = cur.fetchone()
            if row:
                return dict(row)
        return None

    def save_topic_note(
        self,
        note_id: Optional[int],
        title: str,
        passage_ref: str,
        content: str,
        book_number: Optional[int] = None,
        chapter: Optional[int] = None,
    ) -> int:
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        with self._get_connection() as conn:
            if note_id:
                conn.execute(
                    """
                    UPDATE study_notes
                    SET title = ?, passage_ref = ?, content = ?, updated_at = ?
                    WHERE id = ?
                    """,
                    (title, passage_ref, content, now, note_id),
                )
                res_id = note_id
            else:
                cur = conn.execute(
                    """
                    INSERT INTO study_notes (book_number, chapter, passage_ref, title, content, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    """,
                    (book_number, chapter, passage_ref, title, content, now, now),
                )
                res_id = cur.lastrowid
            conn.commit()
            return res_id

    def delete_note(self, note_id: int):
        with self._get_connection() as conn:
            conn.execute("DELETE FROM study_notes WHERE id = ?", (note_id,))
            conn.commit()
