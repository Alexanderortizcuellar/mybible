"""
Main Application Window for MyBible PySide6 GUI
"""

import re
from typing import Optional

from PySide6.QtCore import Qt, QUrl
from PySide6.QtGui import QAction, QFont, QIcon
from PySide6.QtWidgets import (
    QMainWindow,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QSplitter,
    QComboBox,
    QPushButton,
    QLabel,
    QTextBrowser,
    QTreeWidget,
    QTreeWidgetItem,
    QTabWidget,
    QLineEdit,
    QToolBar,
    QStatusBar,
    QMessageBox,
)

from gui.backend import RustBackendClient
from gui.html_renderer import (
    render_chapter_html,
    render_dict_html,
    clean_verse_tags_to_html,
)
from gui.search_dialog import SearchDialog
from gui.modules_dialog import ModulesDialog


class MainWindow(QMainWindow):
    """Main window for MyBible desktop study application."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("MyBible — Scripture Study Engine")
        self.resize(1280, 800)

        self.backend = RustBackendClient(self)
        self.modules_info = self.backend.get_modules()
        self.current_theme = "light"
        self.font_size = 15

        self.current_book_number = 470  # Matthew by default
        self.current_book_name = "Matthew"
        self.current_chapter = 1
        self.books_data: list[dict] = []

        self.init_ui()
        self.load_translations()
        self.load_books()
        self.load_active_chapter()

    def init_ui(self):
        # 1. Top Navigation Toolbar
        toolbar = QToolBar("Navigation", self)
        toolbar.setMovable(False)
        self.addToolBar(toolbar)

        # Translation dropdown
        toolbar.addWidget(QLabel(" Translation: "))
        self.bible_combo = QComboBox()
        self.bible_combo.currentIndexChanged.connect(self.on_translation_changed)
        toolbar.addWidget(self.bible_combo)

        toolbar.addSeparator()

        # Book dropdown
        toolbar.addWidget(QLabel(" Book: "))
        self.book_combo = QComboBox()
        self.book_combo.currentIndexChanged.connect(self.on_book_combo_changed)
        toolbar.addWidget(self.book_combo)

        # Chapter dropdown
        toolbar.addWidget(QLabel(" Chapter: "))
        self.chapter_combo = QComboBox()
        self.chapter_combo.currentIndexChanged.connect(self.on_chapter_combo_changed)
        toolbar.addWidget(self.chapter_combo)

        # Previous / Next Chapter buttons
        self.prev_btn = QPushButton("◀ Prev")
        self.prev_btn.clicked.connect(self.go_prev_chapter)
        toolbar.addWidget(self.prev_btn)

        self.next_btn = QPushButton("Next ▶")
        self.next_btn.clicked.connect(self.go_next_chapter)
        toolbar.addWidget(self.next_btn)

        toolbar.addSeparator()

        # Subheadings selector
        toolbar.addWidget(QLabel(" Subheadings: "))
        self.subheadings_combo = QComboBox()
        self.subheadings_combo.addItem("Default (Module / Stories)", userData="default")
        self.subheadings_combo.addItem("None (Suppress)", userData="none")
        for s in self.modules_info.get("subheadings", []):
            self.subheadings_combo.addItem(f"{s}", userData=s)
        self.subheadings_combo.currentIndexChanged.connect(self.load_active_chapter)
        toolbar.addWidget(self.subheadings_combo)

        toolbar.addSeparator()

        # Search Action Button
        self.search_btn = QPushButton("🔍 Search")
        self.search_btn.clicked.connect(self.open_search_dialog)
        toolbar.addWidget(self.search_btn)

        # Module Manager Button
        self.modules_btn = QPushButton("📦 Modules")
        self.modules_btn.clicked.connect(self.open_modules_dialog)
        toolbar.addWidget(self.modules_btn)

        toolbar.addSeparator()

        # Theme selector
        toolbar.addWidget(QLabel(" Theme: "))
        self.theme_combo = QComboBox()
        self.theme_combo.addItems(["Light (Day)", "Sepia (Reading)", "Dark (Night)"])
        self.theme_combo.currentIndexChanged.connect(self.on_theme_changed)
        toolbar.addWidget(self.theme_combo)

        # Font Size Controls
        self.btn_font_minus = QPushButton("A-")
        self.btn_font_minus.setFixedWidth(32)
        self.btn_font_minus.clicked.connect(self.decrease_font_size)
        toolbar.addWidget(self.btn_font_minus)

        self.btn_font_plus = QPushButton("A+")
        self.btn_font_plus.setFixedWidth(32)
        self.btn_font_plus.clicked.connect(self.increase_font_size)
        toolbar.addWidget(self.btn_font_plus)

        # 2. Main Central Splitter Layout
        main_splitter = QSplitter(Qt.Horizontal)
        self.setCentralWidget(main_splitter)

        # Left Sidebar: Book Navigation Tree
        self.book_tree = QTreeWidget()
        self.book_tree.setHeaderLabel("Scripture Books")
        self.book_tree.setMaximumWidth(220)
        self.book_tree.itemClicked.connect(self.on_book_tree_clicked)
        main_splitter.addWidget(self.book_tree)

        # Center Widget: Scripture Browser
        self.reader_browser = QTextBrowser()
        self.reader_browser.setOpenLinks(False)
        self.reader_browser.anchorClicked.connect(self.on_reader_link_clicked)
        main_splitter.addWidget(self.reader_browser)

        # Right Sidebar: Commentary / Footnotes / Dictionary Inspector
        self.side_tabs = QTabWidget()
        self.side_tabs.setMinimumWidth(320)
        self.side_tabs.setMaximumWidth(450)

        # Tab 1: Footnotes & Commentary
        self.tab_commentary = QWidget()
        comm_layout = QVBoxLayout(self.tab_commentary)

        comm_hdr = QHBoxLayout()
        comm_hdr.addWidget(QLabel("Module:"))
        self.comm_module_combo = QComboBox()
        for c in self.modules_info.get("commentaries", []):
            self.comm_module_combo.addItem(c)
        self.comm_module_combo.currentIndexChanged.connect(self.load_active_commentary)
        comm_hdr.addWidget(self.comm_module_combo)
        comm_layout.addLayout(comm_hdr)

        self.comm_browser = QTextBrowser()
        self.comm_browser.setOpenLinks(False)
        self.comm_browser.anchorClicked.connect(self.on_reader_link_clicked)
        comm_layout.addWidget(self.comm_browser)

        self.side_tabs.addTab(self.tab_commentary, "Commentary")

        # Tab 2: Cross References
        self.tab_xref = QWidget()
        xref_layout = QVBoxLayout(self.tab_xref)
        self.xref_browser = QTextBrowser()
        self.xref_browser.setOpenLinks(False)
        self.xref_browser.anchorClicked.connect(self.on_reader_link_clicked)
        xref_layout.addWidget(self.xref_browser)
        self.side_tabs.addTab(self.tab_xref, "Cross-Refs")

        # Tab 3: Dictionary & Strong's
        self.tab_dict = QWidget()
        dict_layout = QVBoxLayout(self.tab_dict)

        dict_search_row = QHBoxLayout()
        self.dict_input = QLineEdit()
        self.dict_input.setPlaceholderText("Lookup Strong # (e.g. G26, H7225) or word...")
        self.dict_input.returnPressed.connect(self.lookup_dictionary_term)
        dict_search_row.addWidget(self.dict_input)

        self.dict_btn = QPushButton("Lookup")
        self.dict_btn.clicked.connect(self.lookup_dictionary_term)
        dict_search_row.addWidget(self.dict_btn)
        dict_layout.addLayout(dict_search_row)

        self.dict_browser = QTextBrowser()
        self.dict_browser.setOpenLinks(False)
        self.dict_browser.anchorClicked.connect(self.on_reader_link_clicked)
        dict_layout.addWidget(self.dict_browser)

        self.side_tabs.addTab(self.tab_dict, "Dictionary")

        main_splitter.addWidget(self.side_tabs)

        # Set default splitter proportions (20% tree, 55% reader, 25% side tabs)
        main_splitter.setSizes([200, 680, 400])

        # Status Bar
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)
        self.statusBar.showMessage("MyBible Engine initialized.")

    # --- Loading & Data Handling ---

    def load_translations(self):
        """Populates installed Bible translations."""
        self.bible_combo.blockSignals(True)
        self.bible_combo.clear()
        bibles = self.modules_info.get("bibles", [])
        if not bibles:
            self.bible_combo.addItem("None installed")
        else:
            for b in bibles:
                self.bible_combo.addItem(b)
        self.bible_combo.blockSignals(False)

    def load_books(self):
        """Fetches books from backend for active translation and populates dropdown and tree."""
        active_bible = self.bible_combo.currentText()
        if not active_bible or active_bible == "None installed":
            return

        self.books_data = self.backend.get_books(active_bible)

        # 1. Populate Book Dropdown
        self.book_combo.blockSignals(True)
        self.book_combo.clear()
        for b in self.books_data:
            self.book_combo.addItem(b.get("long_name", ""), userData=b)
        self.book_combo.blockSignals(False)

        # 2. Populate Tree
        self.book_tree.clear()
        ot_group = QTreeWidgetItem(self.book_tree, ["Old Testament"])
        nt_group = QTreeWidgetItem(self.book_tree, ["New Testament"])

        for b in self.books_data:
            num = b.get("book_number", 0)
            name = b.get("long_name", "")
            item = QTreeWidgetItem([name])
            item.setData(0, Qt.UserRole, b)

            if num < 470:
                ot_group.addChild(item)
            else:
                nt_group.addChild(item)

        ot_group.setExpanded(True)
        nt_group.setExpanded(True)

        # Select default Matthew
        self.set_active_book("Matthew")

    def set_active_book(self, book_name: str):
        """Selects book in combo box and loads chapters."""
        idx = self.book_combo.findText(book_name, Qt.MatchContains)
        if idx >= 0:
            self.book_combo.setCurrentIndex(idx)
        else:
            self.update_chapters_for_current_book()

    def update_chapters_for_current_book(self):
        """Populates chapter combo based on currently selected book."""
        data = self.book_combo.currentData()
        if not data:
            return

        self.current_book_number = data.get("book_number", 10)
        self.current_book_name = data.get("long_name", "")
        max_ch = data.get("chapter_count", 1)

        self.chapter_combo.blockSignals(True)
        self.chapter_combo.clear()
        for ch in range(1, max_ch + 1):
            self.chapter_combo.addItem(str(ch))

        if self.current_chapter > max_ch:
            self.current_chapter = 1

        self.chapter_combo.setCurrentText(str(self.current_chapter))
        self.chapter_combo.blockSignals(False)

    def load_active_chapter(self):
        """Queries backend for the active chapter and renders it."""
        bible = self.bible_combo.currentText()
        if not bible or bible == "None installed":
            return

        ref = f"{self.current_book_name} {self.current_chapter}"
        sub_mode = self.subheadings_combo.currentData()
        subheadings = None
        no_sub = False
        if sub_mode == "none":
            no_sub = True
        elif sub_mode != "default":
            subheadings = sub_mode

        self.statusBar.showMessage(f"Loading {ref} ({bible})...")

        read_data = self.backend.read_passage(
            reference=ref,
            bible=bible,
            subheadings=subheadings,
            no_subheadings=no_sub,
            footnotes=True,
            xrefs=True,
        )

        if not read_data:
            self.reader_browser.setHtml(f"<p>Could not load {ref}.</p>")
            self.statusBar.showMessage(f"Error loading {ref}")
            return

        html = render_chapter_html(read_data, theme=self.current_theme, font_size=self.font_size)
        self.reader_browser.setHtml(html)

        self.load_active_commentary()
        self.load_active_xrefs()
        self.statusBar.showMessage(f"Displaying {ref} ({bible})")

    def load_active_commentary(self):
        """Loads commentary notes for current chapter into side panel."""
        comm_module = self.comm_module_combo.currentText()
        if not comm_module:
            self.comm_browser.setHtml("<p style='padding: 10px;'>No commentary module installed.</p>")
            return

        ref = f"{self.current_book_name} {self.current_chapter}"
        data = self.backend.get_commentary(ref, module=comm_module)

        if not data or not data.get("notes"):
            self.comm_browser.setHtml(
                f"<div style='padding: 12px; font-family: Segoe UI, sans-serif;'><b>No commentary notes found for {ref} in {comm_module}.</b></div>"
            )
            return

        notes_html = []
        for n in data.get("notes", []):
            v_from = n.get("verse_from", 1)
            v_to = n.get("verse_to", v_from)
            v_badge = f"{v_from}" if v_from == v_to else f"{v_from}-{v_to}"
            formatted_text = clean_verse_tags_to_html(n.get("text", ""))
            notes_html.append(f"""
            <div style="border-bottom: 1px solid #ccc; padding: 10px 0;">
                <div style="color: #2b6cb0; font-weight: bold; margin-bottom: 4px;">Verse {v_badge}</div>
                <div style="font-size: {self.font_size - 1}pt; line-height: 1.5;">{formatted_text}</div>
            </div>
            """)

        self.comm_browser.setHtml(f"""
        <html>
        <body style="font-family: Segoe UI, sans-serif; padding: 10px;">
            <h3>📝 {ref} — {comm_module}</h3>
            {"".join(notes_html)}
        </body>
        </html>
        """)

    def load_active_xrefs(self):
        """Loads cross-references for chapter start into side panel."""
        ref = f"{self.current_book_name} {self.current_chapter}:1"
        data = self.backend.get_cross_references(ref)
        if not data or not data.get("references"):
            self.xref_browser.setHtml(
                f"<div style='padding: 12px; font-family: Segoe UI, sans-serif;'><b>No cross-references found for {ref}.</b></div>"
            )
            return

        items_html = []
        for r in data.get("references", []):
            target = r.get("target", "")
            preview = r.get("preview", "")
            items_html.append(f"""
            <div style="margin-bottom: 8px;">
                <a href="goto:{target}" style="color: #2b6cb0; font-weight: bold; text-decoration: none;">🔗 {target}</a>
                <div style="color: #666; font-size: {self.font_size - 2}pt;">{preview}</div>
            </div>
            """)

        self.xref_browser.setHtml(f"""
        <html>
        <body style="font-family: Segoe UI, sans-serif; padding: 10px;">
            <h4>🔗 Cross-References for {ref}</h4>
            {"".join(items_html)}
        </body>
        </html>
        """)

    # --- User Navigation Handlers ---

    def on_translation_changed(self):
        self.load_books()
        self.load_active_chapter()

    def on_book_combo_changed(self):
        self.update_chapters_for_current_book()
        self.load_active_chapter()

    def on_chapter_combo_changed(self):
        ch_text = self.chapter_combo.currentText()
        if ch_text.isdigit():
            self.current_chapter = int(ch_text)
            self.load_active_chapter()

    def on_book_tree_clicked(self, item: QTreeWidgetItem):
        data = item.data(0, Qt.UserRole)
        if data:
            name = data.get("long_name", "")
            self.set_active_book(name)

    def go_prev_chapter(self):
        if self.current_chapter > 1:
            self.current_chapter -= 1
            self.chapter_combo.setCurrentText(str(self.current_chapter))
        else:
            cur_idx = self.book_combo.currentIndex()
            if cur_idx > 0:
                self.book_combo.setCurrentIndex(cur_idx - 1)
                data = self.book_combo.currentData()
                if data:
                    self.current_chapter = data.get("chapter_count", 1)
                    self.chapter_combo.setCurrentText(str(self.current_chapter))

    def go_next_chapter(self):
        data = self.book_combo.currentData()
        max_ch = data.get("chapter_count", 1) if data else 1
        if self.current_chapter < max_ch:
            self.current_chapter += 1
            self.chapter_combo.setCurrentText(str(self.current_chapter))
        else:
            cur_idx = self.book_combo.currentIndex()
            if cur_idx < self.book_combo.count() - 1:
                self.book_combo.setCurrentIndex(cur_idx + 1)
                self.current_chapter = 1
                self.chapter_combo.setCurrentText("1")

    def on_theme_changed(self):
        text = self.theme_combo.currentText()
        if "Sepia" in text:
            self.current_theme = "sepia"
        elif "Dark" in text:
            self.current_theme = "dark"
        else:
            self.current_theme = "light"
        self.load_active_chapter()

    def increase_font_size(self):
        self.font_size = min(28, self.font_size + 1)
        self.load_active_chapter()

    def decrease_font_size(self):
        self.font_size = max(10, self.font_size - 1)
        self.load_active_chapter()

    def on_reader_link_clicked(self, qurl: QUrl):
        """Handles clicks on footnote, Strong's number, cross-reference, and verse links."""
        url = qurl.toString()

        if url.startswith("fn:"):
            # Footnote clicked: switch to commentary tab and query note
            marker = url.replace("fn:", "")
            self.side_tabs.setCurrentWidget(self.tab_commentary)
            self.statusBar.showMessage(f"Footnote [{marker}] selected")

        elif url.startswith("dict:"):
            # Strong's code clicked: switch to dictionary tab and lookup
            term = url.replace("dict:", "")
            self.side_tabs.setCurrentWidget(self.tab_dict)
            self.dict_input.setText(term)
            self.lookup_dictionary_term()

        elif url.startswith("goto:") or url.startswith("xref:"):
            target = url.split(":", 1)[1]
            self.navigate_to_reference(target)

        elif url.startswith("verse:"):
            v_num = url.replace("verse:", "")
            self.statusBar.showMessage(f"Verse {v_num} selected")

    def navigate_to_reference(self, target_ref: str):
        """Parses reference and navigates to target book and chapter."""
        # e.g. "Romans 8:28" or "Gen 1:1" or "Matthew/3/2"
        clean = target_ref.replace("/", " ")
        parts = clean.split()
        if len(parts) >= 2:
            book_cand = parts[0]
            ch_cand = parts[1].split(":")[0]
            if ch_cand.isdigit():
                self.set_active_book(book_cand)
                self.current_chapter = int(ch_cand)
                self.chapter_combo.setCurrentText(str(self.current_chapter))

    def lookup_dictionary_term(self):
        term = self.dict_input.text().strip()
        if not term:
            return

        dict_module = self.modules_info.get("dictionaries", [None])[0]
        data = self.backend.get_dict_entry(term, module=dict_module)
        if data:
            html = render_dict_html(data, theme=self.current_theme, font_size=self.font_size)
            self.dict_browser.setHtml(html)
            self.statusBar.showMessage(f"Looked up '{term}' in {dict_module}")
        else:
            self.dict_browser.setHtml(f"<p>No definition found for '{term}'.</p>")

    # --- Dialogs ---

    def open_search_dialog(self):
        bibles = self.modules_info.get("bibles", [])
        current = self.bible_combo.currentText()
        dialog = SearchDialog(
            backend=self.backend,
            installed_bibles=bibles,
            current_bible=current,
            theme=self.current_theme,
            parent=self,
        )
        dialog.verseSelected.connect(self.on_search_verse_selected)
        dialog.exec()

    def on_search_verse_selected(self, book_name: str, chapter: int, verse: int):
        self.set_active_book(book_name)
        self.current_chapter = chapter
        self.chapter_combo.setCurrentText(str(chapter))
        self.statusBar.showMessage(f"Jumped to {book_name} {chapter}:{verse}")

    def open_modules_dialog(self):
        dialog = ModulesDialog(backend=self.backend, parent=self)
        dialog.modulesUpdated.connect(self.on_modules_updated)
        dialog.exec()

    def on_modules_updated(self):
        self.modules_info = self.backend.get_modules()
        self.load_translations()
        self.load_active_chapter()
