"""Main Application Window for MyBible PySide6 GUI
Features a flexible dock-based study workspace, rich Scripture reader,
parallel translation comparisons, dictionary lookups, and commentary verse hover tooltips.
"""

from typing import Optional

from PySide6.QtCore import Qt, QUrl
from PySide6.QtGui import QAction, QCursor, QFont, QKeySequence
from PySide6.QtWidgets import (
    QCheckBox,
    QComboBox,
    QDockWidget,
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QMainWindow,
    QMenu,
    QMenuBar,
    QMessageBox,
    QPushButton,
    QStatusBar,
    QTextBrowser,
    QToolBar,
    QToolTip,
    QTreeWidget,
    QTreeWidgetItem,
    QVBoxLayout,
    QWidget,
)

from gui.backend import RustBackendClient
from gui.html_renderer import (
    clean_verse_tags_to_html,
    render_chapter_html,
    render_commentary_html,
    render_compare_html,
    render_dict_html,
)
from gui.modules_dialog import ModulesDialog
from gui.search_dialog import SearchDialog


class MainWindow(QMainWindow):
    """Dock-based Scripture Study Main Window."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("MyBible — Scripture Study Engine")
        self.resize(1360, 850)

        # Apply global stylesheet fixing ComboBox baseline and polishing docks
        self.apply_base_stylesheet()

        # Backend client
        self.backend = RustBackendClient(self)
        self.modules_info = self.backend.get_modules()
        self.current_theme = "light"
        self.font_size = 15

        self.current_book_number = 470  # Matthew by default
        self.current_book_name = "Matthew"
        self.current_chapter = 1
        self.books_data: list[dict] = []

        # Persistent dialog references
        self.search_dialog: Optional[SearchDialog] = None
        self.modules_dialog: Optional[ModulesDialog] = None

        # Comparison translation checkboxes
        self.compare_checkboxes: dict[str, QCheckBox] = {}

        self.init_ui()
        self.load_translations()
        self.load_books()
        self.load_active_chapter()

    def apply_base_stylesheet(self):
        self.setStyleSheet("""
            QComboBox {
                padding: 4px 26px 4px 10px;
                min-height: 26px;
                font-size: 13px;
                font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif;
                border: 1px solid #cbd5e0;
                border-radius: 4px;
                background: #ffffff;
                color: #2d3748;
            }
            QComboBox:hover {
                border-color: #3182ce;
            }
            QComboBox:focus {
                border-color: #2b6cb0;
            }
            QComboBox::drop-down {
                subcontrol-origin: padding;
                subcontrol-position: top right;
                width: 22px;
                border-left: 1px solid #e2e8f0;
                border-top-right-radius: 4px;
                border-bottom-right-radius: 4px;
            }
            QComboBox QAbstractItemView {
                border: 1px solid #cbd5e0;
                background: #ffffff;
                selection-background-color: #2b6cb0;
                selection-color: #ffffff;
                padding: 4px;
            }
            QDockWidget {
                font-family: 'Segoe UI', sans-serif;
                font-weight: bold;
            }
            QDockWidget::title {
                background: #edf2f7;
                padding: 6px 10px;
                border-bottom: 1px solid #cbd5e0;
                font-size: 12px;
                color: #2d3748;
            }
            QToolBar {
                spacing: 6px;
                padding: 4px;
                border-bottom: 1px solid #e2e8f0;
                background: #f7fafc;
            }
        """)

    def init_ui(self):
        # 1. Menu Bar
        self.init_menu_bar()

        # 2. Navigation & Study Toolbar
        self.init_toolbar()

        # 3. Central Widget: Scripture Reader
        self.reader_browser = QTextBrowser()
        self.reader_browser.setOpenLinks(False)
        self.reader_browser.setMouseTracking(True)
        self.reader_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.reader_browser.highlighted.connect(self.on_link_highlighted)
        self.setCentralWidget(self.reader_browser)

        # 4. Dockable Panels
        self.init_docks()

        # Status Bar
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)
        self.statusBar.showMessage("MyBible Engine initialized.")

    def init_menu_bar(self):
        menubar = self.menuBar()

        # File Menu
        file_menu = menubar.addMenu("&File")

        modules_action = QAction("📦 Manage Modules...", self)
        modules_action.setShortcut(QKeySequence("Ctrl+M"))
        modules_action.triggered.connect(self.open_modules_dialog)
        file_menu.addAction(modules_action)

        file_menu.addSeparator()

        exit_action = QAction("Exit", self)
        exit_action.setShortcut(QKeySequence("Ctrl+Q"))
        exit_action.triggered.connect(self.close)
        file_menu.addAction(exit_action)

        # View Menu (Dock Toggles & Layout)
        self.view_menu = menubar.addMenu("&View")

        # Theme Sub-menu
        theme_menu = self.view_menu.addMenu("Reading Theme")
        for theme_key, theme_label in [("light", "Light (Day)"), ("sepia", "Sepia (Reading)"), ("dark", "Dark (Night)")]:
            act = QAction(theme_label, self)
            act.triggered.connect(lambda _, t=theme_key: self.set_theme(t))
            theme_menu.addAction(act)

        self.view_menu.addSeparator()

        # Font Zoom Actions
        zoom_in_act = QAction("Zoom In", self)
        zoom_in_act.setShortcut(QKeySequence.ZoomIn)
        zoom_in_act.triggered.connect(self.increase_font_size)
        self.view_menu.addAction(zoom_in_act)

        zoom_out_act = QAction("Zoom Out", self)
        zoom_out_act.setShortcut(QKeySequence.ZoomOut)
        zoom_out_act.triggered.connect(self.decrease_font_size)
        self.view_menu.addAction(zoom_out_act)

        self.view_menu.addSeparator()

        reset_layout_act = QAction("Reset Default Dock Layout", self)
        reset_layout_act.triggered.connect(self.reset_default_layout)
        self.view_menu.addAction(reset_layout_act)

        self.view_menu.addSeparator()

        # Tools Menu
        tools_menu = menubar.addMenu("&Tools")
        search_action = QAction("🔍 Search Scripture...", self)
        search_action.setShortcut(QKeySequence("Ctrl+F"))
        search_action.triggered.connect(self.open_search_dialog)
        tools_menu.addAction(search_action)

        reload_action = QAction("🔄 Reload Modules", self)
        reload_action.setShortcut(QKeySequence("F5"))
        reload_action.triggered.connect(self.reload_all_modules)
        tools_menu.addAction(reload_action)

    def init_toolbar(self):
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
        self.subheadings_combo.addItem("Default (Stories / Embedded)", userData="default")
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
        self.theme_combo.addItems(["Light", "Sepia", "Dark"])
        self.theme_combo.currentIndexChanged.connect(self.on_theme_changed)
        toolbar.addWidget(self.theme_combo)

        # Font Size Controls
        self.btn_font_minus = QPushButton("A-")
        self.btn_font_minus.setFixedWidth(30)
        self.btn_font_minus.clicked.connect(self.decrease_font_size)
        toolbar.addWidget(self.btn_font_minus)

        self.btn_font_plus = QPushButton("A+")
        self.btn_font_plus.setFixedWidth(30)
        self.btn_font_plus.clicked.connect(self.increase_font_size)
        toolbar.addWidget(self.btn_font_plus)

    def init_docks(self):
        # Configure docking corners
        self.setCorner(Qt.TopLeftCorner, Qt.LeftDockWidgetArea)
        self.setCorner(Qt.BottomLeftCorner, Qt.LeftDockWidgetArea)
        self.setCorner(Qt.TopRightCorner, Qt.RightDockWidgetArea)
        self.setCorner(Qt.BottomRightCorner, Qt.RightDockWidgetArea)

        # --- 1. Left Dock: Books & Chapters Tree ---
        self.nav_dock = QDockWidget("📖 Books & Navigation", self)
        self.nav_dock.setObjectName("nav_dock")
        self.nav_dock.setAllowedAreas(Qt.LeftDockWidgetArea | Qt.RightDockWidgetArea)

        self.book_tree = QTreeWidget()
        self.book_tree.setHeaderLabel("Biblical Canon")
        self.book_tree.itemClicked.connect(self.on_book_tree_clicked)
        self.nav_dock.setWidget(self.book_tree)
        self.addDockWidget(Qt.LeftDockWidgetArea, self.nav_dock)

        # --- 2. Right Dock: Commentary & Footnotes ---
        self.comm_dock = QDockWidget("📝 Commentary & Footnotes", self)
        self.comm_dock.setObjectName("comm_dock")

        comm_widget = QWidget()
        comm_layout = QVBoxLayout(comm_widget)
        comm_layout.setContentsMargins(6, 6, 6, 6)

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
        self.comm_browser.setMouseTracking(True)
        self.comm_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.comm_browser.highlighted.connect(self.on_link_highlighted)
        comm_layout.addWidget(self.comm_browser)

        self.comm_dock.setWidget(comm_widget)
        self.addDockWidget(Qt.RightDockWidgetArea, self.comm_dock)

        # --- 3. Right Dock: Cross-References ---
        self.xref_dock = QDockWidget("🔗 Cross-References", self)
        self.xref_dock.setObjectName("xref_dock")

        self.xref_browser = QTextBrowser()
        self.xref_browser.setOpenLinks(False)
        self.xref_browser.setMouseTracking(True)
        self.xref_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.xref_browser.highlighted.connect(self.on_link_highlighted)
        self.xref_dock.setWidget(self.xref_browser)
        self.addDockWidget(Qt.RightDockWidgetArea, self.xref_dock)

        # Tabify Cross-Refs with Commentary
        self.tabifyDockWidget(self.comm_dock, self.xref_dock)

        # --- 4. Right Dock: Dictionary & Concordance ---
        self.dict_dock = QDockWidget("📚 Dictionary & Concordance", self)
        self.dict_dock.setObjectName("dict_dock")

        dict_widget = QWidget()
        dict_layout = QVBoxLayout(dict_widget)
        dict_layout.setContentsMargins(6, 6, 6, 6)

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
        self.dict_browser.setMouseTracking(True)
        self.dict_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.dict_browser.highlighted.connect(self.on_link_highlighted)
        dict_layout.addWidget(self.dict_browser)

        self.dict_dock.setWidget(dict_widget)
        self.addDockWidget(Qt.RightDockWidgetArea, self.dict_dock)

        # Tabify Dictionary with Cross-References
        self.tabifyDockWidget(self.xref_dock, self.dict_dock)

        # --- 5. Bottom Dock: Multi-Version Comparison ---
        self.compare_dock = QDockWidget("⚖️ Parallel Bible Comparison", self)
        self.compare_dock.setObjectName("compare_dock")

        compare_widget = QWidget()
        comp_layout = QVBoxLayout(compare_widget)
        comp_layout.setContentsMargins(6, 6, 6, 6)

        # Translation Selector Row
        self.comp_hdr = QHBoxLayout()
        self.comp_hdr.addWidget(QLabel("<b>Compare Versions:</b>"))

        self.comp_checkbox_container = QWidget()
        self.comp_checkbox_layout = QHBoxLayout(self.comp_checkbox_container)
        self.comp_checkbox_layout.setContentsMargins(0, 0, 0, 0)
        self.update_compare_checkboxes()
        self.comp_hdr.addWidget(self.comp_checkbox_container)

        self.comp_hdr.addStretch()

        comp_refresh_btn = QPushButton("Refresh Comparison")
        comp_refresh_btn.clicked.connect(self.load_active_comparison)
        self.comp_hdr.addWidget(comp_refresh_btn)
        comp_layout.addLayout(self.comp_hdr)

        self.compare_browser = QTextBrowser()
        self.compare_browser.setOpenLinks(False)
        self.compare_browser.setMouseTracking(True)
        self.compare_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.compare_browser.highlighted.connect(self.on_link_highlighted)
        comp_layout.addWidget(self.compare_browser)

        self.compare_dock.setWidget(compare_widget)
        self.addDockWidget(Qt.BottomDockWidgetArea, self.compare_dock)

        # Activate commentary tab by default on right
        self.comm_dock.raise_()

        # Add View Menu Toggles for all docks
        for dock in [self.nav_dock, self.comm_dock, self.xref_dock, self.dict_dock, self.compare_dock]:
            self.view_menu.addAction(dock.toggleViewAction())

    def update_compare_checkboxes(self):
        """Populates checkboxes for all installed Bibles."""
        # Clear existing
        for cb in self.compare_checkboxes.values():
            cb.deleteLater()
        self.compare_checkboxes.clear()

        bibles = self.modules_info.get("bibles", [])
        for b in bibles:
            cb = QCheckBox(b)
            # Default check active bible and first 1-2 other bibles
            cb.setChecked(True)
            cb.stateChanged.connect(self.load_active_comparison)
            self.comp_checkbox_layout.addWidget(cb)
            self.compare_checkboxes[b] = cb

    def reset_default_layout(self):
        """Restores dock positions to optimal study arrangement."""
        self.addDockWidget(Qt.LeftDockWidgetArea, self.nav_dock)
        self.addDockWidget(Qt.RightDockWidgetArea, self.comm_dock)
        self.addDockWidget(Qt.RightDockWidgetArea, self.xref_dock)
        self.addDockWidget(Qt.RightDockWidgetArea, self.dict_dock)
        self.tabifyDockWidget(self.comm_dock, self.xref_dock)
        self.tabifyDockWidget(self.xref_dock, self.dict_dock)
        self.addDockWidget(Qt.BottomDockWidgetArea, self.compare_dock)
        self.nav_dock.show()
        self.comm_dock.show()
        self.xref_dock.show()
        self.dict_dock.show()
        self.compare_dock.show()
        self.comm_dock.raise_()
        self.statusBar.showMessage("Dock layout restored to default.")

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

        # Find and select current book
        for i in range(self.book_combo.count()):
            data = self.book_combo.itemData(i)
            if data and data.get("book_number") == self.current_book_number:
                self.book_combo.setCurrentIndex(i)
                break
        self.book_combo.blockSignals(False)

        # 2. Populate Chapters Dropdown
        self.update_chapters_for_current_book()

        # 3. Populate Book Navigation Tree
        self.book_tree.clear()
        ot_root = QTreeWidgetItem(self.book_tree, ["Old Testament (Tanakh)"])
        nt_root = QTreeWidgetItem(self.book_tree, ["New Testament (B'rit Chadashah)"])
        ot_root.setExpanded(True)
        nt_root.setExpanded(True)

        for b in self.books_data:
            item = QTreeWidgetItem([f"{b.get('long_name', '')} ({b.get('chapter_count', 1)})"])
            item.setData(0, Qt.UserRole, b)
            if b.get("testament") == "ot":
                ot_root.addChild(item)
            else:
                nt_root.addChild(item)

    def update_chapters_for_current_book(self):
        """Updates chapter combo for the currently selected book."""
        data = self.book_combo.currentData()
        max_ch = data.get("chapter_count", 1) if data else 1

        self.chapter_combo.blockSignals(True)
        self.chapter_combo.clear()
        for ch in range(1, max_ch + 1):
            self.chapter_combo.addItem(str(ch))

        if self.current_chapter > max_ch:
            self.current_chapter = 1

        self.chapter_combo.setCurrentText(str(self.current_chapter))
        self.chapter_combo.blockSignals(False)

    def load_active_chapter(self):
        """Loads and renders the current passage in the Scripture Browser and inspector docks."""
        bible = self.bible_combo.currentText()
        if not bible or bible == "None installed":
            self.reader_browser.setHtml("<h2 style='padding: 20px;'>No Bible modules installed.</h2>")
            return

        ref = f"{self.current_book_name} {self.current_chapter}"

        sub_opt = self.subheadings_combo.currentData()
        subheadings = sub_opt != "none"
        sub_mod = None if sub_opt in ("default", "none") else sub_opt

        read_data = self.backend.get_passage(
            reference=ref,
            bible=bible,
            subheadings=subheadings,
            footnotes=True,
            xrefs=True,
            subheadings_module=sub_mod,
        )

        if not read_data:
            self.reader_browser.setHtml(f"<p>Could not load {ref}.</p>")
            self.statusBar.showMessage(f"Error loading {ref}")
            return

        html = render_chapter_html(read_data, theme=self.current_theme, font_size=self.font_size)
        self.reader_browser.setHtml(html)

        self.load_active_commentary()
        self.load_active_xrefs()
        self.load_active_comparison()
        self.statusBar.showMessage(f"Displaying {ref} ({bible})")

    def load_active_commentary(self):
        """Loads commentary notes for current chapter into side panel."""
        comm_module = self.comm_module_combo.currentText()
        if not comm_module:
            self.comm_browser.setHtml("<p style='padding: 10px;'>No commentary module installed.</p>")
            return

        ref = f"{self.current_book_name} {self.current_chapter}"
        data = self.backend.get_commentary(ref, module=comm_module)

        html = render_commentary_html(data or {}, theme=self.current_theme, font_size=self.font_size - 1)
        self.comm_browser.setHtml(html)

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
                <a href="goto:{target}" class="verse-hover" style="color: #2b6cb0; font-weight: bold; text-decoration: none;">🔗 {target}</a>
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

    def load_active_comparison(self):
        """Loads parallel verses for the current chapter in all checked Bible versions."""
        active_bibles = [b for b, cb in self.compare_checkboxes.items() if cb.isChecked()]
        if not active_bibles:
            cur = self.bible_combo.currentText()
            if cur:
                active_bibles = [cur]

        ref = f"{self.current_book_name} {self.current_chapter}"
        data = self.backend.get_compare(ref, bibles=active_bibles)
        html = render_compare_html(data or {}, theme=self.current_theme, font_size=self.font_size - 1)
        self.compare_browser.setHtml(html)

    # --- User Navigation Handlers ---

    def on_translation_changed(self):
        self.load_books()
        self.load_active_chapter()

    def on_book_combo_changed(self):
        data = self.book_combo.currentData()
        if data:
            self.current_book_name = data.get("long_name", "")
            self.current_book_number = data.get("book_number", 470)
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
                self.chapter_combo.setCurrentText(str(self.current_chapter))

    def set_active_book(self, book_name_or_prefix: str):
        target = book_name_or_prefix.strip().lower()
        for i in range(self.book_combo.count()):
            item_text = self.book_combo.itemText(i).lower()
            data = self.book_combo.itemData(i)
            short_name = (data.get("short_name", "") if data else "").lower()
            if target == item_text or target == short_name or item_text.startswith(target):
                self.book_combo.setCurrentIndex(i)
                break

    # --- Tooltips & Hyperlink Interception ---

    def on_link_highlighted(self, url: str):
        """Displays rich instant hover tooltips for Scripture references in commentaries and articles."""
        if not url:
            QToolTip.hideText()
            return

        if url.startswith("goto:") or url.startswith("xref:"):
            ref = url.replace("goto:", "").replace("xref:", "").strip()
            # Clean separators like "Exodus/6/20" -> "Exodus 6:20"
            ref = ref.replace("/", " ")
            cur_bible = self.bible_combo.currentText()
            preview = self.backend.get_verse_preview(ref, bible=cur_bible)
            if preview:
                tooltip_html = f"""
                <div style="font-family: Segoe UI, sans-serif; font-size: 11pt; padding: 6px; max-width: 440px;">
                    <b style="color: #2b6cb0;">📖 {ref}</b> <span style="color: #718096; font-size: 9pt;">({cur_bible})</span><br/><br/>
                    <span>{preview}</span>
                </div>
                """
                QToolTip.showText(QCursor.pos(), tooltip_html)
            else:
                QToolTip.showText(QCursor.pos(), f"📖 {ref}")

        elif url.startswith("dict:"):
            term = url.replace("dict:", "").strip()
            dict_mod = self.modules_info.get("dictionaries", [None])[0]
            entry = self.backend.get_dict_entry(term, module=dict_mod)
            if entry and entry.get("found"):
                topic = entry.get("topic", term)
                plain = (entry.get("plain_text", "") or "")[:200]
                tooltip_html = f"""
                <div style="font-family: Segoe UI, sans-serif; font-size: 11pt; padding: 6px; max-width: 440px;">
                    <b style="color: #805ad5;">📚 {topic}</b><br/><br/>
                    <span>{plain}...</span>
                </div>
                """
                QToolTip.showText(QCursor.pos(), tooltip_html)
            else:
                QToolTip.showText(QCursor.pos(), f"📚 Lookup '{term}' in dictionary")

        elif url.startswith("fn:"):
            marker = url.replace("fn:", "").strip()
            QToolTip.showText(QCursor.pos(), f"Footnote [{marker}]")

    def on_reader_link_clicked(self, qurl: QUrl):
        """Handles user clicking interactive hyperlinks in browsers."""
        url = qurl.toString()
        if url.startswith("fn:"):
            marker = url.replace("fn:", "")
            self.statusBar.showMessage(f"Footnote [{marker}] selected")
            self.comm_dock.raise_()

        elif url.startswith("dict:"):
            term = url.replace("dict:", "")
            self.dict_input.setText(term)
            self.lookup_dictionary_term()
            self.dict_dock.raise_()

        elif url.startswith("goto:") or url.startswith("xref:"):
            target = url.split(":", 1)[1]
            self.navigate_to_reference(target)

        elif url.startswith("verse:"):
            v_num = url.replace("verse:", "")
            self.statusBar.showMessage(f"Verse {v_num} selected")

    def navigate_to_reference(self, target_ref: str):
        """Parses reference and navigates to target book and chapter."""
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
            self.dict_dock.raise_()
            self.statusBar.showMessage(f"Looked up '{term}' in {dict_module}")
        else:
            self.dict_browser.setHtml(f"<p>No definition found for '{term}'.</p>")

    # --- Dialogs & Tools ---

    def open_search_dialog(self):
        if self.search_dialog is None:
            bibles = self.modules_info.get("bibles", [])
            current = self.bible_combo.currentText()
            self.search_dialog = SearchDialog(
                backend=self.backend,
                installed_bibles=bibles,
                current_bible=current,
                theme=self.current_theme,
                parent=self,
            )
            self.search_dialog.verseSelected.connect(self.on_search_verse_selected)

        self.search_dialog.show()
        self.search_dialog.raise_()
        self.search_dialog.activateWindow()

    def on_search_verse_selected(self, book: str, chapter: int, verse: int):
        self.set_active_book(book)
        self.current_chapter = chapter
        self.chapter_combo.setCurrentText(str(chapter))
        self.statusBar.showMessage(f"Navigated from search to {book} {chapter}:{verse}")

    def open_modules_dialog(self):
        if self.modules_dialog is None:
            self.modules_dialog = ModulesDialog(self.backend, self)
            self.modules_dialog.modulesUpdated.connect(self.reload_all_modules)
        self.modules_dialog.show()
        self.modules_dialog.raise_()
        self.modules_dialog.activateWindow()

    def reload_all_modules(self):
        self.statusBar.showMessage("Reloading installed modules...")
        self.backend.reload_modules()
        self.modules_info = self.backend.get_modules()
        self.load_translations()
        self.update_compare_checkboxes()
        self.load_books()
        self.load_active_chapter()
        self.statusBar.showMessage("Modules successfully reloaded.")

    # --- Themes & Typography ---

    def on_theme_changed(self):
        text = self.theme_combo.currentText().lower()
        if "sepia" in text:
            self.current_theme = "sepia"
        elif "dark" in text:
            self.current_theme = "dark"
        else:
            self.current_theme = "light"
        self.load_active_chapter()

    def set_theme(self, theme_key: str):
        self.current_theme = theme_key
        idx = 0 if theme_key == "light" else (1 if theme_key == "sepia" else 2)
        self.theme_combo.setCurrentIndex(idx)

    def increase_font_size(self):
        if self.font_size < 28:
            self.font_size += 1
            self.load_active_chapter()

    def decrease_font_size(self):
        if self.font_size > 10:
            self.font_size -= 1
            self.load_active_chapter()

    def closeEvent(self, event):
        """Clean up backend child process on exit."""
        if hasattr(self, "backend") and self.backend:
            self.backend.close()
        event.accept()
