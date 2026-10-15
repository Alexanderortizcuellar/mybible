"""Search Dialog for MyBible GUI
Allows searching across single, multiple, or all translations with instant passage navigation
and persistent search history.
"""

from typing import Optional
from PySide6.QtCore import Signal, QUrl
from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QLineEdit,
    QPushButton,
    QComboBox,
    QLabel,
    QTextBrowser,
    QProgressBar,
)

from gui.backend import RustBackendClient
from gui.html_renderer import render_search_results_html


class SearchDialog(QDialog):
    """Interactive Bible search dialog with persistent state."""

    verseSelected = Signal(str, int, int)  # book_name, chapter, verse

    def __init__(
        self,
        backend: RustBackendClient,
        installed_bibles: list[str],
        current_bible: str = "NLT",
        theme: str = "light",
        parent=None,
    ):
        super().__init__(parent)
        self.backend = backend
        self.installed_bibles = installed_bibles
        self.current_bible = current_bible
        self.theme = theme
        self.last_results: Optional[dict] = None

        self.setWindowTitle("Search Scripture — MyBible")
        self.resize(850, 650)
        self.init_ui()
        self.restore_last_search()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # 1. Search Query Input Row
        query_layout = QHBoxLayout()
        self.query_input = QLineEdit()
        self.query_input.setPlaceholderText(
            'Enter words or "exact phrase" (e.g. repent kingdom)...'
        )
        self.query_input.returnPressed.connect(self.perform_search)
        query_layout.addWidget(self.query_input)

        self.search_btn = QPushButton("Search")
        self.search_btn.setStyleSheet("""
            QPushButton {
                background-color: #2b6cb0;
                color: white;
                border-radius: 4px;
                padding: 6px 16px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #3182ce;
            }
        """)
        self.search_btn.clicked.connect(self.perform_search)
        query_layout.addWidget(self.search_btn)
        layout.addLayout(query_layout)

        # 2. Options Filter Row
        filter_layout = QHBoxLayout()

        filter_layout.addWidget(QLabel("<b>Mode:</b>"))
        self.mode_combo = QComboBox()
        self.mode_combo.addItems(["All Words (AND)", "Exact Phrase", "Any Word (OR)"])
        filter_layout.addWidget(self.mode_combo)

        filter_layout.addWidget(QLabel("<b>Scope:</b>"))
        self.scope_combo = QComboBox()
        self.scope_combo.addItems(
            ["Entire Bible", "Old Testament (OT)", "New Testament (NT)"]
        )
        filter_layout.addWidget(self.scope_combo)

        filter_layout.addWidget(QLabel("<b>Translation:</b>"))
        self.bible_combo = QComboBox()
        self.bible_combo.addItem("All Translations", userData="all")
        for b in self.installed_bibles:
            self.bible_combo.addItem(b, userData=b)
        idx = self.bible_combo.findData(self.current_bible)
        if idx >= 0:
            self.bible_combo.setCurrentIndex(idx)
        filter_layout.addWidget(self.bible_combo)

        layout.addLayout(filter_layout)

        # 3. Progress bar (hidden by default)
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 0)
        self.progress_bar.setVisible(False)
        layout.addWidget(self.progress_bar)

        # 4. Status Bar
        self.status_label = QLabel("Enter a search term and press Search.")
        self.status_label.setStyleSheet(
            "color: #4a5568; font-size: 10pt; margin: 2px 0;"
        )
        layout.addWidget(self.status_label)

        # 5. Results Browser
        self.results_browser = QTextBrowser()
        self.results_browser.setOpenLinks(False)
        self.results_browser.anchorClicked.connect(self.on_link_clicked)
        layout.addWidget(self.results_browser)

    def restore_last_search(self):
        """Restores cached search results from backend memory if available."""
        if self.last_results:
            return

        last = self.backend.get_last_search()
        if last and last.get("hits"):
            self.last_results = last
            q_text = last.get("query", "")
            if q_text:
                self.query_input.setText(q_text)
            total = last.get("total_matches", 0)
            self.status_label.setText(
                f"Showing previous search results for '{q_text}' ({total} matches)."
            )
            html = render_search_results_html(last, query_text=q_text, theme=self.theme)
            self.results_browser.setHtml(html)

    def perform_search(self):
        query = self.query_input.text().strip()
        if not query:
            return

        mode_text = self.mode_combo.currentText()
        if "Phrase" in mode_text:
            mode = "phrase"
        elif "Any" in mode_text:
            mode = "any"
        else:
            mode = "all"

        scope_text = self.scope_combo.currentText()
        testament = None
        if "Old" in scope_text:
            testament = "ot"
        elif "New" in scope_text:
            testament = "nt"

        target_bible = self.bible_combo.currentData()
        all_bibles = target_bible == "all"
        bible = None if all_bibles else target_bible

        self.status_label.setText(f"Searching for '{query}'...")
        self.progress_bar.setVisible(True)
        self.search_btn.setEnabled(False)

        results = self.backend.search_bible(
            query=query,
            bible=bible,
            all_bibles=all_bibles,
            testament=testament,
            mode=mode,
            limit=100,
        )

        self.progress_bar.setVisible(False)
        self.search_btn.setEnabled(True)

        if not results:
            self.status_label.setText(f"No results found for '{query}'.")
            self.results_browser.setHtml(
                f"<div style='padding: 24px; font-family: Segoe UI, sans-serif; font-size: 13pt;'>No matching verses found for <b>{query}</b>.</div>"
            )
            return

        self.last_results = results
        total = results.get("total_matches", 0)
        duration = results.get("duration_ms", 0)
        hits_count = len(results.get("hits", []))

        self.status_label.setText(
            f"Found {total} matching verses in {duration}ms (showing top {hits_count})."
        )

        html = render_search_results_html(results, query_text=query, theme=self.theme)
        self.results_browser.setHtml(html)

    def on_link_clicked(self, qurl: QUrl):
        url = qurl.toString()
        if url.startswith("goto:"):
            parts = url.replace("goto:", "").split("/")
            if len(parts) >= 3:
                book = parts[0]
                chapter = int(parts[1])
                verse = int(parts[2])
                self.verseSelected.emit(book, chapter, verse)
                self.hide()
