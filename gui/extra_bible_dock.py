"""Parallel Bible View Dock Widget for MyBible Desktop.
Allows opening multiple independent or synchronized Bible reader panels
to compare translations side-by-side.
"""

from PySide6.QtCore import Qt, Signal
from PySide6.QtWidgets import (
    QCheckBox,
    QComboBox,
    QDockWidget,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QTextBrowser,
    QVBoxLayout,
    QWidget,
)

from gui.html_renderer import render_chapter_html


class BibleDockWidget(QDockWidget):
    """An independent dockable Bible reader widget."""

    referenceRequested = Signal(str)  # emitted when user clicks an anchor to navigate

    def __init__(
        self,
        backend,
        available_bibles: list[str],
        default_bible: str,
        book_num: int,
        book_name: str,
        chapter: int,
        theme: str = "light",
        font_size: int = 15,
        dock_id: int = 1,
        parent=None,
    ):
        super().__init__(f"📖 Bible View {dock_id} ({default_bible})", parent)
        self.setObjectName(f"extra_bible_dock_{dock_id}")
        self.setAttribute(Qt.WA_DeleteOnClose, True)

        self.backend = backend
        self.available_bibles = available_bibles
        self.current_bible = default_bible
        self.current_book_num = book_num
        self.current_book_name = book_name
        self.current_chapter = chapter
        self.current_theme = theme
        self.font_size = font_size
        self.dock_id = dock_id

        self.init_ui()
        self.load_chapter()

    def init_ui(self):
        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(6, 6, 6, 6)
        layout.setSpacing(6)

        # Controls Header
        hdr = QHBoxLayout()
        hdr.setSpacing(8)

        hdr.addWidget(QLabel("<b>Bible:</b>"))
        self.bible_combo = QComboBox()
        self.bible_combo.setMinimumWidth(85)
        for b in self.available_bibles:
            self.bible_combo.addItem(b)
        idx = self.bible_combo.findText(self.current_bible)
        if idx >= 0:
            self.bible_combo.setCurrentIndex(idx)
        self.bible_combo.currentIndexChanged.connect(self.on_bible_changed)
        hdr.addWidget(self.bible_combo)

        self.sync_checkbox = QCheckBox("Sync Passage")
        self.sync_checkbox.setChecked(True)
        self.sync_checkbox.setToolTip("Synchronize with main Scripture navigation")
        hdr.addWidget(self.sync_checkbox)

        hdr.addStretch()

        self.prev_btn = QPushButton("◀")
        self.prev_btn.setFixedWidth(26)
        self.prev_btn.setToolTip("Previous Chapter")
        self.prev_btn.clicked.connect(self.go_prev)
        hdr.addWidget(self.prev_btn)

        self.next_btn = QPushButton("▶")
        self.next_btn.setFixedWidth(26)
        self.next_btn.setToolTip("Next Chapter")
        self.next_btn.clicked.connect(self.go_next)
        hdr.addWidget(self.next_btn)

        layout.addLayout(hdr)

        # Scripture Text Browser
        self.browser = QTextBrowser()
        self.browser.setOpenLinks(False)
        self.browser.setMouseTracking(True)
        self.browser.anchorClicked.connect(self.on_anchor_clicked)
        layout.addWidget(self.browser)

        self.setWidget(container)

    def on_bible_changed(self):
        new_bible = self.bible_combo.currentText()
        if new_bible and new_bible != self.current_bible:
            self.current_bible = new_bible
            self.setWindowTitle(f"📖 Bible View {self.dock_id} ({self.current_bible})")
            self.load_chapter()

    def set_passage(self, book_num: int, book_name: str, chapter: int):
        """Called by MainWindow when the central passage changes."""
        if not self.sync_checkbox.isChecked():
            return
        self.current_book_num = book_num
        self.current_book_name = book_name
        self.current_chapter = chapter
        self.load_chapter()

    def set_theme_and_font(self, theme: str, font_size: int):
        self.current_theme = theme
        self.font_size = font_size
        self.load_chapter()

    def load_chapter(self):
        """Retrieves chapter content for this dock's active translation."""
        if not self.current_bible:
            return
        ref = f"{self.current_book_name} {self.current_chapter}"
        data = self.backend.get_passage(
            reference=ref,
            bible=self.current_bible,
            subheadings=False,
            footnotes=True,
            xrefs=True,
        )
        if data:
            html = render_chapter_html(
                data, theme=self.current_theme, font_size=self.font_size
            )
            self.browser.setHtml(html)
        else:
            self.browser.setHtml(
                f"<div style='padding: 20px;'><h3>{ref} ({self.current_bible})</h3>"
                f"<p style='opacity: 0.7;'>No content available for this chapter.</p></div>"
            )

    def go_prev(self):
        if self.current_chapter > 1:
            self.current_chapter -= 1
            self.load_chapter()

    def go_next(self):
        self.current_chapter += 1
        self.load_chapter()

    def on_anchor_clicked(self, url):
        target = url.toString() if hasattr(url, "toString") else str(url)
        if target.startswith("goto:") or target.startswith("xref:"):
            ref = target.split(":", 1)[1]
            self.referenceRequested.emit(ref)
