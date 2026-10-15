"""
Search Dialog for MyBible GUI
Allows searching across single, multiple, or all translations with instant passage navigation.
"""

from typing import Optional
from PySide6.QtCore import Qt, Signal, QUrl
from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QLineEdit,
    QPushButton,
    QComboBox,
    QLabel,
    QTextBrowser,
    QCheckBox,
    QProgressBar,
)

from gui.backend import RustBackendClient
from gui.html_renderer import render_search_results_html


class SearchDialog(QDialog):
    """Interactive Bible search dialog."""

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

        self.setWindowTitle("Search Scripture — MyBible")
        self.resize(800, 650)
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # 1. Search Query Input Row
        query_layout = QHBoxLayout()
        self.query_input = QLineEdit()
        self.query_input.setPlaceholderText("Enter words or \"exact phrase\" (e.g. repent kingdom)...")
        self.query_input.returnPressed.connect(self.perform_search)
        query_layout.addWidget(self.query_input)

        self.search_btn = QPushButton("Search")
        self.search_btn.clicked.connect(self.perform_search)
        query_layout.addWidget(self.search_btn)
        layout.addLayout(query_layout)

        # 2. Options Filter Row
        filter_layout = QHBoxLayout()

        filter_layout.addWidget(QLabel("Mode:"))
        self.mode_combo = QComboBox()
        self.mode_combo.addItems(["All Words (AND)", "Exact Phrase", "Any Word (OR)"])
        filter_layout.addWidget(self.mode_combo)

        filter_layout.addWidget(QLabel("Scope:"))
        self.scope_combo = QComboBox()
        self.scope_combo.addItems(["Entire Bible", "Old Testament (OT)", "New Testament (NT)"])
        filter_layout.addWidget(self.scope_combo)

        filter_layout.addWidget(QLabel("Translation:"))
        self.bible_combo = QComboBox()
        self.bible_combo.addItem("All Translations", userData="all")
        for b in self.installed_bibles:
            self.bible_combo.addItem(b, userData=b)
        # Select current bible by default
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
        layout.addWidget(self.status_label)

        # 5. Results Browser
        self.results_browser = QTextBrowser()
        self.results_browser.setOpenLinks(False)
        self.results_browser.anchorClicked.connect(self.on_link_clicked)
        layout.addWidget(self.results_browser)

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

        # Execute search via QProcess wrapper
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
                f"<p style='padding: 20px; font-size: 14pt;'>No matching verses found for <b>{query}</b>.</p>"
            )
            return

        total = results.get("total_matches", 0)
        duration = results.get("duration_ms", 0)
        hits_count = len(results.get("hits", []))

        self.status_label.setText(
            f"Found {total} matching verses in {duration}ms (showing {hits_count})."
        )

        html = render_search_results_html(results, theme=self.theme)
        self.results_browser.setHtml(html)

    def on_link_clicked(self, qurl: QUrl):
        url = qurl.toString()
        if url.startswith("goto:"):
            # Format: goto:BookName/Chapter/Verse
            parts = url.replace("goto:", "").split("/")
            if len(parts) >= 3:
                book = parts[0]
                chapter = int(parts[1])
                verse = int(parts[2])
                self.verseSelected.emit(book, chapter, verse)
                self.accept()
