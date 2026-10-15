"""Main Application Window for MyBible PySide6 GUI
Features a flexible dock-based study workspace, rich Scripture reader,
parallel translation comparisons, dictionary lookups, and commentary verse hover tooltips.
"""

import re
from typing import Optional

from PySide6.QtCore import QRect, QSettings, Qt, QTimer, Signal
from PySide6.QtGui import QAction, QCursor, QKeySequence
from PySide6.QtWidgets import (
    QApplication,
    QCheckBox,
    QComboBox,
    QDockWidget,
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QListWidget,
    QListWidgetItem,
    QMainWindow,
    QMessageBox,
    QPushButton,
    QSizePolicy,
    QSpinBox,
    QStackedWidget,
    QStatusBar,
    QTextBrowser,
    QTextEdit,
    QToolBar,
    QToolTip,
    QTreeWidget,
    QTreeWidgetItem,
    QVBoxLayout,
    QWidget,
)

from gui.backend import RustBackendClient
from gui.extra_bible_dock import BibleDockWidget
from gui.html_renderer import (
    render_chapter_html,
    render_commentary_html,
    render_compare_html,
    render_devotion_html,
    render_dict_html,
)
from gui.modules_dialog import ModulesDialog
from gui.notes_manager import NotesManager
from gui.search_dialog import SearchDialog
from gui.styles import get_theme_palette, get_theme_stylesheet


def is_system_dark_mode() -> bool:
    """Detects whether system OS theme is dark."""
    pal = QApplication.palette()
    return pal.color(pal.ColorRole.Window).value() < 128


class ScriptureBrowser(QTextBrowser):
    """Rich interactive text browser with double-click word lookup support."""

    wordDoubleClicked = Signal(str)

    def mouseDoubleClickEvent(self, event):
        super().mouseDoubleClickEvent(event)
        if event.button() == Qt.LeftButton:
            cursor = self.textCursor()
            selected = cursor.selectedText().strip()
            clean = re.sub(r"^[^\w]+|[^\w]+$", "", selected)
            if clean:
                self.wordDoubleClicked.emit(clean)


class MainWindow(QMainWindow):
    """Professional Dock-based Scripture Study Main Window."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("MyBible — Professional Scripture Study Engine")
        self.resize(1400, 880)

        # Backend client
        self.backend = RustBackendClient(self)
        self.modules_info = self.backend.get_modules()

        # Persistent application settings
        self.settings = QSettings("MyBible", "MyBibleDesktop")

        self.lookup_on_double_click = bool(
            self.settings.value("lookupOnDoubleClick", True, type=bool)
        )
        self._current_hover_url: str = ""

        # Initial theme detection (restore saved or match OS default if dark)
        saved_theme = self.settings.value("theme")
        if saved_theme and isinstance(saved_theme, str):
            self.current_theme = saved_theme
        else:
            self.current_theme = "dark" if is_system_dark_mode() else "light"

        saved_font = self.settings.value("fontSize")
        if saved_font:
            try:
                self.font_size = int(saved_font)
            except (ValueError, TypeError):
                self.font_size = 15
        else:
            self.font_size = 15

        saved_book = self.settings.value("activeBook")
        if saved_book:
            try:
                self.current_book_number = int(saved_book)
            except (ValueError, TypeError):
                self.current_book_number = 470
        else:
            self.current_book_number = 470  # Matthew by default

        saved_ch = self.settings.value("activeChapter")
        if saved_ch:
            try:
                self.current_chapter = int(saved_ch)
            except (ValueError, TypeError):
                self.current_chapter = 1
        else:
            self.current_chapter = 1

        self.current_book_name = "Matthew"
        self.books_data: list[dict] = []

        # Persistent dialog references
        self.search_dialog: Optional[SearchDialog] = None
        self.modules_dialog: Optional[ModulesDialog] = None

        # Dynamically opened parallel Bible docks
        self.extra_bible_docks: list[BibleDockWidget] = []

        # Comparison translation checkboxes
        self.compare_checkboxes: dict[str, QCheckBox] = {}

        import datetime

        self.current_devotion_day = datetime.date.today().timetuple().tm_yday
        self.notes_manager = NotesManager(self.modules_info.get("data_dir", "modules"))
        self._active_note_book = self.current_book_number
        self._active_note_chapter = self.current_chapter

        self.init_ui()
        self.load_translations()
        self.load_books()

        # Apply cohesive styling and palette
        self.apply_theme(self.current_theme)
        self.load_active_chapter()
        self.load_active_devotion()
        self.load_active_passage_note()

    def apply_theme(self, theme_name: str):
        """Applies full palette, stylesheet, and renders reader content for the chosen theme."""
        self.current_theme = theme_name

        # 1. Update application palette so native controls inherit correct colors
        pal = get_theme_palette(theme_name)
        app = QApplication.instance()
        if app:
            app.setPalette(pal)
        self.setPalette(pal)

        # 2. Apply comprehensive stylesheet
        self.setStyleSheet(get_theme_stylesheet(theme_name))

        # 3. Update active theme combo without triggering loop
        idx = 0 if theme_name == "light" else (1 if theme_name == "sepia" else 2)
        if hasattr(self, "theme_combo") and self.theme_combo.currentIndex() != idx:
            self.theme_combo.blockSignals(True)
            self.theme_combo.setCurrentIndex(idx)
            self.theme_combo.blockSignals(False)

        # 4. Re-render web content if already initialized
        if hasattr(self, "reader_browser") and self.books_data:
            self.load_active_chapter()
        if hasattr(self, "devotion_browser"):
            self.load_active_devotion()

    def init_ui(self):
        # 1. Menu Bar
        self.init_menu_bar()

        # 2. Navigation & Study Toolbar
        self.init_toolbar()

        # 3. Central Widget: Scripture Reader
        self.reader_browser = ScriptureBrowser()
        self.reader_browser.setOpenLinks(False)
        self.reader_browser.setMouseTracking(True)
        self.reader_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.reader_browser.highlighted.connect(self.on_link_highlighted)
        self.reader_browser.wordDoubleClicked.connect(self.on_word_double_clicked)
        self.setCentralWidget(self.reader_browser)

        # 4. Dockable Panels
        self.init_docks()

        # Status Bar
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)
        self.statusBar.showMessage("MyBible Engine initialized.")

        # Restore saved geometry and dock layout
        saved_geo = self.settings.value("geometry")
        saved_state = self.settings.value("windowState")
        if saved_geo:
            self.restoreGeometry(saved_geo)
        if saved_state:
            self.restoreState(saved_state)
        else:
            self.compare_dock.hide()
            self.compare_toggle_btn.setChecked(False)

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
        for theme_key, theme_label in [
            ("light", "Light (Day)"),
            ("sepia", "Sepia (Reading)"),
            ("dark", "Dark (Night)"),
        ]:
            act = QAction(theme_label, self)
            act.triggered.connect(lambda _, t=theme_key: self.apply_theme(t))
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

        self.compare_menu_action = QAction("⚖️ Compare Bible Versions", self)
        self.compare_menu_action.setCheckable(True)
        self.compare_menu_action.setChecked(False)
        self.compare_menu_action.triggered.connect(self.toggle_compare_dock)
        tools_menu.addAction(self.compare_menu_action)

        new_bible_action = QAction("➕ Open Parallel Bible View", self)
        new_bible_action.triggered.connect(self.create_extra_bible_dock)
        tools_menu.addAction(new_bible_action)

        tools_menu.addSeparator()

        self.double_click_action = QAction(
            "📖 Double-Click Word to Lookup Dictionary", self
        )
        self.double_click_action.setCheckable(True)
        self.double_click_action.setChecked(self.lookup_on_double_click)
        self.double_click_action.triggered.connect(self.toggle_double_click_lookup)
        tools_menu.addAction(self.double_click_action)

        tools_menu.addSeparator()

        reload_action = QAction("🔄 Reload Modules", self)
        reload_action.setShortcut(QKeySequence("F5"))
        reload_action.triggered.connect(self.reload_all_modules)
        tools_menu.addAction(reload_action)

    def init_toolbar(self):
        toolbar = QToolBar("Navigation", self)
        toolbar.setObjectName("nav_toolbar")
        toolbar.setMovable(False)
        self.addToolBar(toolbar)

        # Translation dropdown
        toolbar.addWidget(QLabel("Bible:"))
        self.bible_combo = QComboBox()
        self.bible_combo.setFixedWidth(85)
        self.bible_combo.currentIndexChanged.connect(self.on_translation_changed)
        toolbar.addWidget(self.bible_combo)

        toolbar.addSeparator()

        # Book dropdown
        toolbar.addWidget(QLabel("Book:"))
        self.book_combo = QComboBox()
        self.book_combo.setFixedWidth(135)
        self.book_combo.currentIndexChanged.connect(self.on_book_combo_changed)
        toolbar.addWidget(self.book_combo)

        # Chapter dropdown
        toolbar.addWidget(QLabel("Ch:"))
        self.chapter_combo = QComboBox()
        self.chapter_combo.setFixedWidth(65)
        self.chapter_combo.currentIndexChanged.connect(self.on_chapter_combo_changed)
        toolbar.addWidget(self.chapter_combo)

        # Previous / Next Chapter buttons
        self.prev_btn = QPushButton("◀")
        self.prev_btn.setObjectName("navBtn")
        self.prev_btn.setToolTip("Previous Chapter")
        self.prev_btn.setFixedWidth(28)
        self.prev_btn.clicked.connect(self.go_prev_chapter)
        toolbar.addWidget(self.prev_btn)

        self.next_btn = QPushButton("▶")
        self.next_btn.setObjectName("navBtn")
        self.next_btn.setToolTip("Next Chapter")
        self.next_btn.setFixedWidth(28)
        self.next_btn.clicked.connect(self.go_next_chapter)
        toolbar.addWidget(self.next_btn)

        toolbar.addSeparator()

        # Subheadings selector
        toolbar.addWidget(QLabel("Headings:"))
        self.subheadings_combo = QComboBox()
        self.subheadings_combo.setFixedWidth(150)
        self.subheadings_combo.addItem("Default (Stories)", userData="default")
        self.subheadings_combo.addItem("None (Suppress)", userData="none")
        for s in self.modules_info.get("subheadings", []):
            self.subheadings_combo.addItem(f"{s}", userData=s)
        self.subheadings_combo.currentIndexChanged.connect(self.load_active_chapter)
        toolbar.addWidget(self.subheadings_combo)

        toolbar.addSeparator()

        # Action Buttons
        self.search_btn = QPushButton("🔍 Search")
        self.search_btn.setObjectName("primaryBtn")
        self.search_btn.clicked.connect(self.open_search_dialog)
        toolbar.addWidget(self.search_btn)

        self.compare_toggle_btn = QPushButton("⚖️ Compare")
        self.compare_toggle_btn.setCheckable(True)
        self.compare_toggle_btn.setChecked(False)
        self.compare_toggle_btn.clicked.connect(self.toggle_compare_dock)
        toolbar.addWidget(self.compare_toggle_btn)

        # Expanding spacer
        spacer = QWidget()
        spacer.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Preferred)
        toolbar.addWidget(spacer)

        # Theme selector
        toolbar.addWidget(QLabel("Theme:"))
        self.theme_combo = QComboBox()
        self.theme_combo.setFixedWidth(90)
        self.theme_combo.addItems(["Light", "Sepia", "Dark"])
        initial_idx = (
            0
            if self.current_theme == "light"
            else (1 if self.current_theme == "sepia" else 2)
        )
        self.theme_combo.setCurrentIndex(initial_idx)
        self.theme_combo.currentIndexChanged.connect(self.on_theme_changed)
        toolbar.addWidget(self.theme_combo)

        # Font Size Controls
        self.btn_font_minus = QPushButton("A-")
        self.btn_font_minus.setObjectName("fontBtn")
        self.btn_font_minus.setFixedWidth(30)
        self.btn_font_minus.setToolTip("Decrease Font Size")
        self.btn_font_minus.clicked.connect(self.decrease_font_size)
        toolbar.addWidget(self.btn_font_minus)

        self.btn_font_plus = QPushButton("A+")
        self.btn_font_plus.setObjectName("fontBtn")
        self.btn_font_plus.setFixedWidth(30)
        self.btn_font_plus.setToolTip("Increase Font Size")
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
        self.nav_dock.setMinimumWidth(260)
        self.nav_dock.setAllowedAreas(Qt.LeftDockWidgetArea | Qt.RightDockWidgetArea)

        self.book_tree = QTreeWidget()
        self.book_tree.setColumnCount(2)
        self.book_tree.setHeaderLabels(["Book", "Ch"])
        self.book_tree.header().setStretchLastSection(False)
        self.book_tree.header().setSectionResizeMode(0, QHeaderView.Stretch)
        self.book_tree.header().setSectionResizeMode(1, QHeaderView.Fixed)
        self.book_tree.setColumnWidth(1, 42)
        self.book_tree.itemClicked.connect(self.on_book_tree_clicked)
        self.nav_dock.setWidget(self.book_tree)
        self.addDockWidget(Qt.LeftDockWidgetArea, self.nav_dock)

        # --- 2. Right Dock: Commentary & Footnotes ---
        self.comm_dock = QDockWidget("📝 Commentary & Footnotes", self)
        self.comm_dock.setObjectName("comm_dock")
        self.comm_dock.setMinimumWidth(320)

        comm_widget = QWidget()
        comm_layout = QVBoxLayout(comm_widget)
        comm_layout.setContentsMargins(8, 8, 8, 8)
        comm_layout.setSpacing(6)

        comm_hdr = QHBoxLayout()
        comm_hdr.addWidget(QLabel("<b>Commentary:</b>"))
        self.comm_module_combo = QComboBox()
        self.comm_module_combo.setMinimumWidth(140)
        for c in self.modules_info.get("commentaries", []):
            self.comm_module_combo.addItem(c)
        self.comm_module_combo.currentIndexChanged.connect(self.load_active_commentary)
        comm_hdr.addWidget(self.comm_module_combo)
        comm_hdr.addStretch()
        comm_layout.addLayout(comm_hdr)

        self.comm_browser = ScriptureBrowser()
        self.comm_browser.setOpenLinks(False)
        self.comm_browser.setMouseTracking(True)
        self.comm_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.comm_browser.highlighted.connect(self.on_link_highlighted)
        self.comm_browser.wordDoubleClicked.connect(self.on_word_double_clicked)
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
        self.dict_dock = QDockWidget("📚 Dictionary & Lexicon", self)
        self.dict_dock.setObjectName("dict_dock")

        dict_widget = QWidget()
        dict_layout = QVBoxLayout(dict_widget)
        dict_layout.setContentsMargins(8, 8, 8, 8)
        dict_layout.setSpacing(6)

        dict_hdr = QHBoxLayout()
        dict_hdr.addWidget(QLabel("<b>Dictionary:</b>"))
        self.dict_module_combo = QComboBox()
        self.dict_module_combo.setMinimumWidth(140)
        self.update_dictionary_combo()
        self.dict_module_combo.currentIndexChanged.connect(self.on_dict_module_changed)
        dict_hdr.addWidget(self.dict_module_combo)
        dict_hdr.addStretch()
        dict_layout.addLayout(dict_hdr)

        dict_search_row = QHBoxLayout()
        self.dict_input = QLineEdit()
        self.dict_input.setPlaceholderText("Strong # (G26, H7225) or word...")
        self.dict_input.returnPressed.connect(self.lookup_dictionary_term)
        dict_search_row.addWidget(self.dict_input)

        self.dict_btn = QPushButton("Lookup")
        self.dict_btn.setObjectName("primaryBtn")
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

        # --- 5. Right Dock: Daily Devotional ---
        self.devotion_dock = QDockWidget("🌅 Daily Devotional", self)
        self.devotion_dock.setObjectName("devotion_dock")

        dev_widget = QWidget()
        dev_layout = QVBoxLayout(dev_widget)
        dev_layout.setContentsMargins(8, 8, 8, 8)
        dev_layout.setSpacing(6)

        # Devotion module selection row
        dev_hdr = QHBoxLayout()
        dev_hdr.addWidget(QLabel("<b>Devotional:</b>"))
        self.devotion_module_combo = QComboBox()
        self.devotion_module_combo.setMinimumWidth(140)
        self.update_devotion_combo()
        self.devotion_module_combo.currentIndexChanged.connect(
            self.on_devotion_module_changed
        )
        dev_hdr.addWidget(self.devotion_module_combo)
        dev_hdr.addStretch()
        dev_layout.addLayout(dev_hdr)

        # Day navigation controls row
        day_row = QHBoxLayout()
        day_row.setSpacing(6)

        self.dev_prev_btn = QPushButton("◀")
        self.dev_prev_btn.setFixedWidth(28)
        self.dev_prev_btn.setToolTip("Previous Day")
        self.dev_prev_btn.clicked.connect(self.go_prev_devotion_day)
        day_row.addWidget(self.dev_prev_btn)

        day_row.addWidget(QLabel("Day:"))
        self.dev_day_spin = QSpinBox()
        self.dev_day_spin.setRange(1, 366)
        self.dev_day_spin.setValue(self.current_devotion_day)
        self.dev_day_spin.setFixedWidth(70)
        self.dev_day_spin.valueChanged.connect(self.on_devotion_day_changed)
        day_row.addWidget(self.dev_day_spin)

        self.dev_next_btn = QPushButton("▶")
        self.dev_next_btn.setFixedWidth(28)
        self.dev_next_btn.setToolTip("Next Day")
        self.dev_next_btn.clicked.connect(self.go_next_devotion_day)
        day_row.addWidget(self.dev_next_btn)

        day_row.addStretch()

        self.dev_today_btn = QPushButton("Today")
        self.dev_today_btn.setToolTip("Jump to Today's Devotional")
        self.dev_today_btn.clicked.connect(self.go_today_devotion)
        day_row.addWidget(self.dev_today_btn)

        dev_layout.addLayout(day_row)

        self.devotion_browser = ScriptureBrowser()
        self.devotion_browser.setOpenLinks(False)
        self.devotion_browser.setMouseTracking(True)
        self.devotion_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.devotion_browser.highlighted.connect(self.on_link_highlighted)
        self.devotion_browser.wordDoubleClicked.connect(self.on_word_double_clicked)
        dev_layout.addWidget(self.devotion_browser)

        self.devotion_dock.setWidget(dev_widget)
        self.addDockWidget(Qt.RightDockWidgetArea, self.devotion_dock)
        self.tabifyDockWidget(self.dict_dock, self.devotion_dock)

        # --- 6. Right Dock: Personal Study Notes ---
        self.notes_dock = QDockWidget("📝 Study Notes", self)
        self.notes_dock.setObjectName("notes_dock")

        notes_widget = QWidget()
        notes_layout = QVBoxLayout(notes_widget)
        notes_layout.setContentsMargins(8, 8, 8, 8)
        notes_layout.setSpacing(6)

        # Header controls
        notes_hdr = QHBoxLayout()
        self.notes_mode_btn = QPushButton("📑 All Notes")
        self.notes_mode_btn.setToolTip(
            "Toggle between Passage Notes and All Notes Notebook"
        )
        self.notes_mode_btn.clicked.connect(self.on_notes_mode_toggled)
        notes_hdr.addWidget(self.notes_mode_btn)

        notes_hdr.addStretch()

        self.notes_status_lbl = QLabel("")
        self.notes_status_lbl.setStyleSheet("color: gray; font-size: 11px;")
        notes_hdr.addWidget(self.notes_status_lbl)

        self.notes_save_btn = QPushButton("💾 Save")
        self.notes_save_btn.setObjectName("primaryBtn")
        self.notes_save_btn.setToolTip("Save Note (Ctrl+S or Auto-saved)")
        self.notes_save_btn.clicked.connect(self.save_current_passage_note)
        notes_hdr.addWidget(self.notes_save_btn)

        notes_layout.addLayout(notes_hdr)

        # Stacked pages: 0 = Passage Note Editor, 1 = Notebook List
        self.notes_stack = QStackedWidget()

        # Page 0: Passage Note Editor
        page_editor = QWidget()
        pe_layout = QVBoxLayout(page_editor)
        pe_layout.setContentsMargins(0, 0, 0, 0)
        pe_layout.setSpacing(4)

        self.notes_passage_lbl = QLabel(
            f"<b>Notes for {self.current_book_name} {self.current_chapter}:</b>"
        )
        pe_layout.addWidget(self.notes_passage_lbl)

        self.notes_editor = QTextEdit()
        self.notes_editor.setPlaceholderText(
            "Write personal study thoughts, sermon outlines, or verse reflections here...\n"
            "(Notes are automatically saved to your local study database)"
        )
        self.notes_editor.textChanged.connect(self.on_notes_text_changed)
        pe_layout.addWidget(self.notes_editor)

        ed_bottom_row = QHBoxLayout()
        self.notes_clear_btn = QPushButton("Clear Note")
        self.notes_clear_btn.clicked.connect(self.clear_current_note)
        ed_bottom_row.addWidget(self.notes_clear_btn)
        ed_bottom_row.addStretch()
        pe_layout.addLayout(ed_bottom_row)

        self.notes_stack.addWidget(page_editor)

        # Page 1: All Notes Notebook List
        page_list = QWidget()
        pl_layout = QVBoxLayout(page_list)
        pl_layout.setContentsMargins(0, 0, 0, 0)
        pl_layout.setSpacing(4)

        self.notes_search_input = QLineEdit()
        self.notes_search_input.setPlaceholderText(
            "Search all notes by title, passage, or keyword..."
        )
        self.notes_search_input.textChanged.connect(self.refresh_all_notes_list)
        pl_layout.addWidget(self.notes_search_input)

        self.notes_list = QListWidget()
        self.notes_list.itemDoubleClicked.connect(self.on_note_item_double_clicked)
        pl_layout.addWidget(self.notes_list)

        list_btn_row = QHBoxLayout()
        self.notes_open_btn = QPushButton("Open Note")
        self.notes_open_btn.clicked.connect(self.open_selected_note_from_list)
        list_btn_row.addWidget(self.notes_open_btn)

        self.notes_del_btn = QPushButton("Delete Note")
        self.notes_del_btn.clicked.connect(self.delete_selected_note_from_list)
        list_btn_row.addWidget(self.notes_del_btn)

        list_btn_row.addStretch()
        pl_layout.addLayout(list_btn_row)

        self.notes_stack.addWidget(page_list)
        notes_layout.addWidget(self.notes_stack)

        # Auto-save timer (1500 ms debounced)
        self.notes_autosave_timer = QTimer(self)
        self.notes_autosave_timer.setSingleShot(True)
        self.notes_autosave_timer.timeout.connect(self.save_current_passage_note)

        self.notes_dock.setWidget(notes_widget)
        self.addDockWidget(Qt.RightDockWidgetArea, self.notes_dock)
        self.tabifyDockWidget(self.devotion_dock, self.notes_dock)

        # --- 7. Bottom Dock: Multi-Version Comparison ---
        self.compare_dock = QDockWidget("⚖️ Parallel Bible Comparison", self)
        self.compare_dock.setObjectName("compare_dock")

        compare_widget = QWidget()
        comp_layout = QVBoxLayout(compare_widget)
        comp_layout.setContentsMargins(8, 8, 8, 8)
        comp_layout.setSpacing(6)

        # Translation Selector Row
        self.comp_hdr = QHBoxLayout()
        self.comp_hdr.setSpacing(12)
        self.comp_hdr.addWidget(QLabel("<b>Compare Versions:</b>"))

        self.comp_checkbox_container = QWidget()
        self.comp_checkbox_layout = QHBoxLayout(self.comp_checkbox_container)
        self.comp_checkbox_layout.setContentsMargins(0, 0, 0, 0)
        self.comp_checkbox_layout.setSpacing(10)
        self.update_compare_checkboxes()
        self.comp_hdr.addWidget(self.comp_checkbox_container)

        self.comp_hdr.addStretch()

        comp_refresh_btn = QPushButton("Refresh Comparison")
        comp_refresh_btn.clicked.connect(self.load_active_comparison)
        self.comp_hdr.addWidget(comp_refresh_btn)
        comp_layout.addLayout(self.comp_hdr)

        self.compare_browser = ScriptureBrowser()
        self.compare_browser.setOpenLinks(False)
        self.compare_browser.setMouseTracking(True)
        self.compare_browser.anchorClicked.connect(self.on_reader_link_clicked)
        self.compare_browser.highlighted.connect(self.on_link_highlighted)
        self.compare_browser.wordDoubleClicked.connect(self.on_word_double_clicked)
        comp_layout.addWidget(self.compare_browser)

        self.compare_dock.setWidget(compare_widget)
        self.addDockWidget(Qt.BottomDockWidgetArea, self.compare_dock)
        self.compare_dock.hide()

        # Activate commentary tab by default on right
        self.comm_dock.raise_()
        self.resizeDocks([self.nav_dock], [270], Qt.Horizontal)

        # Connect compare dock visibility to toolbar button and menu action
        self.compare_dock.visibilityChanged.connect(self.on_compare_visibility_changed)

        # Add View Menu Toggles for all docks
        for dock in [
            self.nav_dock,
            self.comm_dock,
            self.xref_dock,
            self.dict_dock,
            self.devotion_dock,
            self.notes_dock,
            self.compare_dock,
        ]:
            self.view_menu.addAction(dock.toggleViewAction())

    def on_compare_visibility_changed(self, visible: bool):
        self.compare_toggle_btn.setChecked(visible)
        if hasattr(self, "compare_menu_action"):
            self.compare_menu_action.setChecked(visible)

    def toggle_compare_dock(self, checked: Optional[bool] = None):
        if checked is None or isinstance(checked, bool) is False:
            visible = not self.compare_dock.isVisible()
        else:
            visible = bool(checked)
        self.compare_dock.setVisible(visible)
        if visible:
            self.compare_dock.raise_()
            self.load_active_comparison()

    def create_extra_bible_dock(self):
        """Creates an additional dockable Bible reader panel for parallel study."""
        bibles = self.modules_info.get("bibles", [])
        if not bibles:
            QMessageBox.information(
                self,
                "No Bibles Available",
                "Please download at least one Bible module via File -> Manage Modules.",
            )
            return

        cur_bible = self.bible_combo.currentText()
        default_b = cur_bible
        for b in bibles:
            if b != cur_bible:
                default_b = b
                break

        dock_num = len(self.extra_bible_docks) + 1
        dock = BibleDockWidget(
            backend=self.backend,
            available_bibles=bibles,
            default_bible=default_b,
            book_num=self.current_book_number,
            book_name=self.current_book_name,
            chapter=self.current_chapter,
            theme=self.current_theme,
            font_size=self.font_size,
            dock_id=dock_num,
            parent=self,
        )
        dock.referenceRequested.connect(self.navigate_to_reference)
        self.addDockWidget(Qt.RightDockWidgetArea, dock)
        dock.show()
        dock.raise_()
        self.extra_bible_docks.append(dock)
        self.view_menu.addAction(dock.toggleViewAction())
        self.statusBar.showMessage(f"Opened parallel Bible view for {default_b}")

    def update_compare_checkboxes(self):
        """Populates checkboxes for all installed Bibles."""
        for cb in self.compare_checkboxes.values():
            cb.deleteLater()
        self.compare_checkboxes.clear()

        bibles = self.modules_info.get("bibles", [])
        for b in bibles:
            cb = QCheckBox(b)
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
        self.addDockWidget(Qt.RightDockWidgetArea, self.devotion_dock)
        self.addDockWidget(Qt.RightDockWidgetArea, self.notes_dock)
        self.tabifyDockWidget(self.comm_dock, self.xref_dock)
        self.tabifyDockWidget(self.xref_dock, self.dict_dock)
        self.tabifyDockWidget(self.dict_dock, self.devotion_dock)
        self.tabifyDockWidget(self.devotion_dock, self.notes_dock)
        self.addDockWidget(Qt.BottomDockWidgetArea, self.compare_dock)
        self.nav_dock.show()
        self.comm_dock.show()
        self.xref_dock.show()
        self.dict_dock.show()
        self.devotion_dock.show()
        self.notes_dock.show()
        self.compare_dock.hide()
        self.comm_dock.raise_()
        self.resizeDocks([self.nav_dock], [270], Qt.Horizontal)
        self.compare_toggle_btn.setChecked(False)
        if hasattr(self, "compare_menu_action"):
            self.compare_menu_action.setChecked(False)
        self.statusBar.showMessage("Dock layout restored to default.")

    # --- Loading & Data Handling ---

    def update_dictionary_combo(self):
        """Populates dictionary module dropdown."""
        if not hasattr(self, "dict_module_combo"):
            return
        current = self.dict_module_combo.currentText()
        self.dict_module_combo.blockSignals(True)
        self.dict_module_combo.clear()
        dicts = self.modules_info.get("dictionaries", [])
        if not dicts:
            self.dict_module_combo.addItem("None installed")
        else:
            for d in dicts:
                self.dict_module_combo.addItem(d)
            saved_dict = self.settings.value("activeDictionary")
            preferred = current or saved_dict
            if preferred and self.dict_module_combo.findText(preferred) >= 0:
                self.dict_module_combo.setCurrentText(preferred)
        self.dict_module_combo.blockSignals(False)

    def on_dict_module_changed(self):
        if hasattr(self, "dict_input"):
            term = self.dict_input.text().strip()
            if term:
                self.lookup_dictionary_term(term)

    # --- Devotional Dock Handlers ---

    def update_devotion_combo(self):
        """Populates devotional module dropdown."""
        if not hasattr(self, "devotion_module_combo"):
            return
        current = self.devotion_module_combo.currentText()
        self.devotion_module_combo.blockSignals(True)
        self.devotion_module_combo.clear()
        devotions = self.modules_info.get("devotions", [])
        if not devotions:
            self.devotion_module_combo.addItem("None installed")
        else:
            for d in devotions:
                self.devotion_module_combo.addItem(d)
            saved_dev = self.settings.value("activeDevotion")
            preferred = current or saved_dev
            if preferred and self.devotion_module_combo.findText(preferred) >= 0:
                self.devotion_module_combo.setCurrentText(preferred)
        self.devotion_module_combo.blockSignals(False)

    def on_devotion_module_changed(self):
        self.load_active_devotion()

    def on_devotion_day_changed(self, day: int):
        self.current_devotion_day = day
        self.load_active_devotion()

    def go_prev_devotion_day(self):
        if hasattr(self, "dev_day_spin"):
            cur = self.dev_day_spin.value()
            if cur > 1:
                self.dev_day_spin.setValue(cur - 1)

    def go_next_devotion_day(self):
        if hasattr(self, "dev_day_spin"):
            cur = self.dev_day_spin.value()
            if cur < self.dev_day_spin.maximum():
                self.dev_day_spin.setValue(cur + 1)

    def go_today_devotion(self):
        import datetime

        today_day = datetime.date.today().timetuple().tm_yday
        if hasattr(self, "dev_day_spin"):
            self.dev_day_spin.setValue(today_day)

    def load_active_devotion(self):
        if not hasattr(self, "devotion_browser"):
            return
        mod = (
            self.devotion_module_combo.currentText()
            if hasattr(self, "devotion_module_combo")
            else None
        )
        if not mod or mod == "None installed":
            self.devotion_browser.setHtml(
                "<div style='padding: 16px; font-family: Segoe UI, sans-serif;'>"
                "<h3>🌅 Daily Devotional</h3>"
                "<p style='opacity: 0.8;'>No devotional module installed.</p>"
                "<p>Go to <b>File -> Manage Modules...</b> to download devotional modules like <code>DBEE-d</code>.</p>"
                "</div>"
            )
            return

        data = self.backend.get_devotion(module=mod, day=self.current_devotion_day)
        if data:
            max_days = data.get("max_days", 365)
            if hasattr(self, "dev_day_spin"):
                self.dev_day_spin.blockSignals(True)
                self.dev_day_spin.setMaximum(max_days)
                self.dev_day_spin.setValue(data.get("day", self.current_devotion_day))
                self.dev_day_spin.blockSignals(False)
            html = render_devotion_html(
                data, theme=self.current_theme, font_size=self.font_size - 1
            )
            self.devotion_browser.setHtml(html)
        else:
            self.devotion_browser.setHtml(
                f"<div style='padding: 16px;'>No devotion found for Day {self.current_devotion_day} in {mod}.</div>"
            )

    # --- Personal Study Notes Handlers ---

    def load_active_passage_note(self):
        """Loads personal study notes for the active book and chapter."""
        if not hasattr(self, "notes_editor"):
            return

        # Save previous note if modified and chapter changed
        if (
            self._active_note_book != self.current_book_number
            or self._active_note_chapter != self.current_chapter
        ):
            self.save_current_passage_note()

        self._active_note_book = self.current_book_number
        self._active_note_chapter = self.current_chapter
        passage_str = f"{self.current_book_name} {self.current_chapter}"

        if hasattr(self, "notes_passage_lbl"):
            self.notes_passage_lbl.setText(f"<b>Notes for {passage_str}:</b>")

        note = self.notes_manager.get_passage_note(
            self.current_book_number, self.current_chapter
        )
        content = note.get("content", "") if note else ""

        self.notes_editor.blockSignals(True)
        self.notes_editor.setPlainText(content)
        self.notes_editor.blockSignals(False)

        if hasattr(self, "notes_status_lbl"):
            if note and note.get("updated_at"):
                self.notes_status_lbl.setText(f"Saved: {note.get('updated_at')}")
            else:
                self.notes_status_lbl.setText("Ready")

    def save_current_passage_note(self):
        """Saves current editor contents into SQLite study notes database."""
        if not hasattr(self, "notes_editor"):
            return
        if (
            hasattr(self, "notes_autosave_timer")
            and self.notes_autosave_timer.isActive()
        ):
            self.notes_autosave_timer.stop()

        content = self.notes_editor.toPlainText().strip()
        b_num = self._active_note_book or self.current_book_number
        ch_num = self._active_note_chapter or self.current_chapter
        passage_str = f"{self.current_book_name} {ch_num}"

        # If empty and no prior note exists, skip inserting blank rows
        existing = self.notes_manager.get_passage_note(b_num, ch_num)
        if not content and not existing:
            if hasattr(self, "notes_status_lbl"):
                self.notes_status_lbl.setText("No notes")
            return

        self.notes_manager.save_passage_note(
            book_number=b_num,
            chapter=ch_num,
            passage_ref=passage_str,
            content=content,
            title=f"Study Notes — {passage_str}",
        )
        import datetime

        now_time = datetime.datetime.now().strftime("%H:%M:%S")
        if hasattr(self, "notes_status_lbl"):
            self.notes_status_lbl.setText(f"Saved at {now_time}")

    def on_notes_text_changed(self):
        if hasattr(self, "notes_status_lbl"):
            self.notes_status_lbl.setText("Editing...")
        if hasattr(self, "notes_autosave_timer"):
            self.notes_autosave_timer.start(1500)

    def clear_current_note(self):
        reply = QMessageBox.question(
            self,
            "Clear Study Note",
            f"Are you sure you want to clear your notes for {self.current_book_name} {self.current_chapter}?",
            QMessageBox.Yes | QMessageBox.No,
            QMessageBox.No,
        )
        if reply == QMessageBox.Yes:
            self.notes_editor.setPlainText("")
            self.save_current_passage_note()

    def on_notes_mode_toggled(self):
        current_page = self.notes_stack.currentIndex()
        if current_page == 0:
            # Switching to All Notes list
            self.save_current_passage_note()
            self.refresh_all_notes_list()
            self.notes_stack.setCurrentIndex(1)
            self.notes_mode_btn.setText("📖 Current Chapter")
            self.notes_save_btn.setEnabled(False)
        else:
            # Switching back to Editor
            self.notes_stack.setCurrentIndex(0)
            self.notes_mode_btn.setText("📑 All Notes")
            self.notes_save_btn.setEnabled(True)

    def refresh_all_notes_list(self):
        if not hasattr(self, "notes_list"):
            return
        query = (
            self.notes_search_input.text()
            if hasattr(self, "notes_search_input")
            else ""
        )
        notes = self.notes_manager.get_all_notes(query)

        self.notes_list.clear()
        for n in notes:
            ref = n.get("passage_ref") or "General"
            title = n.get("title") or ref
            date = n.get("updated_at", "")
            raw = n.get("content", "").replace("\n", " ").strip()
            snippet = (raw[:60] + "...") if len(raw) > 60 else raw

            item_text = f"📖 {ref}: {title}\n  {date} — {snippet}"
            item = QListWidgetItem(item_text)
            item.setData(Qt.UserRole, n)
            self.notes_list.addItem(item)

    def on_note_item_double_clicked(self, item: QListWidgetItem):
        self.open_selected_note_from_list()

    def open_selected_note_from_list(self):
        cur_item = self.notes_list.currentItem()
        if not cur_item:
            return
        note = cur_item.data(Qt.UserRole)
        if not note:
            return

        b_num = note.get("book_number")
        ch_num = note.get("chapter")

        if b_num is not None and ch_num is not None:
            # Find book name
            book_item = None
            for b in self.books_data:
                if b.get("book_number") == b_num:
                    book_item = b
                    break
            if book_item:
                self.set_active_book(book_item.get("long_name", ""))
            self.current_chapter = ch_num
            self.chapter_combo.setCurrentText(str(ch_num))
            self.load_active_chapter()

        # Switch stack back to editor
        self.notes_stack.setCurrentIndex(0)
        self.notes_mode_btn.setText("📑 All Notes")
        self.notes_save_btn.setEnabled(True)

    def delete_selected_note_from_list(self):
        cur_item = self.notes_list.currentItem()
        if not cur_item:
            return
        note = cur_item.data(Qt.UserRole)
        if not note:
            return

        ref = note.get("passage_ref") or "this note"
        reply = QMessageBox.question(
            self,
            "Delete Note",
            f"Are you sure you want to permanently delete notes for {ref}?",
            QMessageBox.Yes | QMessageBox.No,
            QMessageBox.No,
        )
        if reply == QMessageBox.Yes:
            self.notes_manager.delete_note(note["id"])
            self.refresh_all_notes_list()
            # If the deleted note was for current passage, clear editor too
            if (
                note.get("book_number") == self.current_book_number
                and note.get("chapter") == self.current_chapter
            ):
                self.notes_editor.blockSignals(True)
                self.notes_editor.setPlainText("")
                self.notes_editor.blockSignals(False)
                self.notes_status_lbl.setText("Note deleted")

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
        ot_root = QTreeWidgetItem(self.book_tree, ["Old Testament", "39"])
        nt_root = QTreeWidgetItem(self.book_tree, ["New Testament", "27"])
        ot_root.setExpanded(True)
        nt_root.setExpanded(True)

        for b in self.books_data:
            name = b.get("long_name", "")
            ch_count = str(b.get("chapter_count", 1))
            item = QTreeWidgetItem([name, ch_count])
            item.setTextAlignment(1, Qt.AlignRight)
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
            self.reader_browser.setHtml(
                "<h2 style='padding: 20px;'>No Bible modules installed.</h2>"
            )
            return

        ref = f"{self.current_book_name} {self.current_chapter}"

        sub_opt = self.subheadings_combo.currentData()
        subheadings = sub_opt != "none"
        sub_mod = None if sub_opt in ("default", "none") else sub_opt

        read_data = self.backend.get_passage(
            reference=ref,
            bible=bible,
            book_number=self.current_book_number,
            chapter=self.current_chapter,
            subheadings=subheadings,
            footnotes=True,
            xrefs=True,
            subheadings_module=sub_mod,
        )

        if not read_data:
            self.reader_browser.setHtml(f"<p>Could not load {ref}.</p>")
            self.statusBar.showMessage(f"Error loading {ref}")
            return

        html = render_chapter_html(
            read_data, theme=self.current_theme, font_size=self.font_size
        )
        self.reader_browser.setHtml(html)

        self.load_active_commentary()
        self.load_active_xrefs()
        self.load_active_comparison()
        self.load_active_passage_note()

        # Update any open parallel Bible docks
        for extra in list(self.extra_bible_docks):
            try:
                if extra and extra.isVisible():
                    extra.set_passage(
                        self.current_book_number,
                        self.current_book_name,
                        self.current_chapter,
                    )
            except RuntimeError:
                if extra in self.extra_bible_docks:
                    self.extra_bible_docks.remove(extra)

        self.statusBar.showMessage(f"Displaying {ref} ({bible})")

    def load_active_commentary(self):
        """Loads commentary notes for current chapter into side panel."""
        comm_module = self.comm_module_combo.currentText()
        if not comm_module:
            self.comm_browser.setHtml(
                "<p style='padding: 10px;'>No commentary module installed.</p>"
            )
            return

        ref = f"{self.current_book_name} {self.current_chapter}"
        data = self.backend.get_commentary(
            ref,
            module=comm_module,
            book_number=self.current_book_number,
            chapter=self.current_chapter,
        )

        html = render_commentary_html(
            data or {}, theme=self.current_theme, font_size=self.font_size - 1
        )
        self.comm_browser.setHtml(html)

    def load_active_xrefs(self):
        """Loads cross-references for chapter start into side panel."""
        ref = f"{self.current_book_name} {self.current_chapter}:1"
        data = self.backend.get_cross_references(
            ref,
            book_number=self.current_book_number,
            chapter=self.current_chapter,
        )
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
                <a href="goto:{target}" class="verse-hover" style="font-weight: bold; text-decoration: none;">🔗 {target}</a>
                <div style="opacity: 0.8; font-size: {self.font_size - 2}pt;">{preview}</div>
            </div>
            """)

        self.xref_browser.setHtml(f"""
        <html>
        <head>
            <style>
                body {{ font-family: 'Segoe UI', sans-serif; padding: 10px; }}
                a {{ text-decoration: none; }}
            </style>
        </head>
        <body>
            <h4>🔗 Cross-References for {ref}</h4>
            {"".join(items_html)}
        </body>
        </html>
        """)

    def load_active_comparison(self):
        """Loads parallel verses for the current chapter in all checked Bible versions."""
        active_bibles = [
            b for b, cb in self.compare_checkboxes.items() if cb.isChecked()
        ]
        if not active_bibles:
            cur = self.bible_combo.currentText()
            if cur:
                active_bibles = [cur]

        ref = f"{self.current_book_name} {self.current_chapter}"
        data = self.backend.get_compare(
            ref,
            bibles=active_bibles,
            book_number=self.current_book_number,
            chapter=self.current_chapter,
        )
        html = render_compare_html(
            data or {}, theme=self.current_theme, font_size=self.font_size - 1
        )
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
        target = book_name_or_prefix.strip().lower().rstrip(".")
        if target.isdigit():
            target_num = int(target)
            for i in range(self.book_combo.count()):
                data = self.book_combo.itemData(i)
                if data and data.get("book_number") == target_num:
                    self.book_combo.setCurrentIndex(i)
                    return

        for i in range(self.book_combo.count()):
            item_text = self.book_combo.itemText(i).lower()
            data = self.book_combo.itemData(i)
            short_name = (
                (data.get("short_name", "") if data else "").lower().rstrip(".")
            )
            if (
                target == item_text
                or target == short_name
                or item_text.startswith(target)
            ):
                self.book_combo.setCurrentIndex(i)
                break

    def resolve_numeric_reference(self, ref_str: str) -> str:
        """Converts a numeric reference like '10 1:1' into a readable book name reference like 'Genesis 1:1'."""
        clean = ref_str.strip().replace("/", " ").replace("–", "-")
        parts = clean.split(None, 1)
        if parts and parts[0].isdigit():
            book_num = int(parts[0])
            book_name = None
            for b in self.books_data:
                if b.get("book_number") == book_num:
                    book_name = b.get("long_name")
                    break
            if not book_name:
                standard_map = {
                    10: "Genesis",
                    20: "Exodus",
                    30: "Leviticus",
                    40: "Numbers",
                    50: "Deuteronomy",
                    60: "Joshua",
                    70: "Judges",
                    80: "Ruth",
                    90: "1 Samuel",
                    100: "2 Samuel",
                    110: "1 Kings",
                    120: "2 Kings",
                    130: "1 Chronicles",
                    140: "2 Chronicles",
                    150: "Ezra",
                    160: "Nehemiah",
                    170: "Esther",
                    190: "Job",
                    220: "Psalms",
                    230: "Proverbs",
                    240: "Ecclesiastes",
                    250: "Song of Solomon",
                    260: "Isaiah",
                    290: "Jeremiah",
                    300: "Lamentations",
                    310: "Ezekiel",
                    330: "Daniel",
                    340: "Hosea",
                    350: "Joel",
                    360: "Amos",
                    370: "Obadiah",
                    380: "Jonah",
                    390: "Micah",
                    400: "Nahum",
                    410: "Habakkuk",
                    420: "Zephaniah",
                    430: "Haggai",
                    440: "Zechariah",
                    450: "Malachi",
                    470: "Matthew",
                    480: "Mark",
                    490: "Luke",
                    500: "John",
                    510: "Acts",
                    520: "Romans",
                    530: "1 Corinthians",
                    540: "2 Corinthians",
                    550: "Galatians",
                    560: "Ephesians",
                    570: "Philippians",
                    580: "Colossians",
                    590: "1 Thessalonians",
                    600: "2 Thessalonians",
                    610: "1 Timothy",
                    620: "2 Timothy",
                    630: "Titus",
                    640: "Philemon",
                    650: "Hebrews",
                    660: "James",
                    670: "1 Peter",
                    680: "2 Peter",
                    690: "1 John",
                    700: "2 John",
                    710: "3 John",
                    720: "Jude",
                    730: "Revelation",
                }
                book_name = standard_map.get(book_num)

            if book_name:
                rest = parts[1] if len(parts) > 1 else "1"
                return f"{book_name} {rest}"
        return ref_str

    # --- Tooltips & Hyperlink Interception ---

    def on_link_highlighted(self, url):
        """Displays rich instant hover tooltips for Scripture references in commentaries and articles."""
        url_str = url.toString() if hasattr(url, "toString") else str(url or "")
        if not url_str:
            self._current_hover_url = ""
            QToolTip.hideText()
            return

        if url_str == self._current_hover_url:
            return
        self._current_hover_url = url_str

        # Scripture passages: goto:, xref:, b:, B:
        if any(url_str.lower().startswith(p) for p in ("goto:", "xref:", "b:")):
            ref = re.sub(r"^(?:goto|xref|[Bb]):", "", url_str).strip()
            ref = self.resolve_numeric_reference(ref)
            cur_bible = self.bible_combo.currentText()
            preview = self.backend.get_verse_preview(ref, bible=cur_bible)
            if preview:
                tooltip_html = f"""
                <div style="font-family: 'Segoe UI', sans-serif; font-size: 11pt; padding: 6px; max-width: 440px;">
                    <b style="color: #3182ce;">📖 {ref}</b> <span style="opacity: 0.7; font-size: 9pt;">({cur_bible})</span><br/><br/>
                    <span>{preview}</span>
                </div>
                """
                QToolTip.showText(QCursor.pos(), tooltip_html, None, QRect(), 20000)
            else:
                QToolTip.showText(QCursor.pos(), f"📖 {ref}", None, QRect(), 5000)

        elif (
            url_str.startswith("dict:")
            or url_str.startswith("s:")
            or url_str.startswith("S:")
        ):
            term = url_str.split(":", 1)[1].strip()
            if term.isdigit():
                prefix = "G" if self.current_book_number >= 400 else "H"
                term = f"{prefix}{term}"

            dict_mod = None
            if hasattr(self, "dict_module_combo"):
                cur_text = self.dict_module_combo.currentText()
                if cur_text and cur_text != "None installed":
                    dict_mod = cur_text

            entry = self.backend.get_dict_entry(term, module=dict_mod)
            if entry and entry.get("found"):
                topic = entry.get("topic", term)
                plain = (entry.get("plain_text", "") or "")[:200]
                mod = entry.get("module", "")
                tooltip_html = f"""
                <div style="font-family: 'Segoe UI', sans-serif; font-size: 11pt; padding: 6px; max-width: 440px;">
                    <b style="color: #805ad5;">📚 {topic}</b> <span style="opacity: 0.7; font-size: 9pt;">({mod})</span><br/><br/>
                    <span>{plain}...</span>
                </div>
                """
                QToolTip.showText(QCursor.pos(), tooltip_html, None, QRect(), 20000)
            else:
                QToolTip.showText(
                    QCursor.pos(),
                    f"📚 Lookup '{term}' in dictionary",
                    None,
                    QRect(),
                    5000,
                )

        elif url_str.startswith("fn:"):
            marker = url_str.replace("fn:", "").strip()
            QToolTip.showText(
                QCursor.pos(), f"Footnote [{marker}]", None, QRect(), 5000
            )

    def on_reader_link_clicked(self, qurl):
        """Handles user clicking interactive hyperlinks in browsers."""
        url = qurl.toString() if hasattr(qurl, "toString") else str(qurl or "")
        if url.startswith("fn:"):
            marker = url.replace("fn:", "")
            self.statusBar.showMessage(f"Footnote [{marker}] selected")
            self.comm_dock.raise_()

        elif url.startswith("dict:") or url.startswith("s:") or url.startswith("S:"):
            term = url.split(":", 1)[1].strip()
            if term.isdigit():
                prefix = "G" if self.current_book_number >= 400 else "H"
                term = f"{prefix}{term}"

            strong_dicts = self.modules_info.get("strong_dictionaries", [])
            if not strong_dicts:
                for d in self.modules_info.get("dictionaries", []):
                    if "strong" in d.lower():
                        strong_dicts.append(d)

            is_strong = term.startswith("G") or term.startswith("H")
            if is_strong and strong_dicts and hasattr(self, "dict_module_combo"):
                if self.dict_module_combo.currentText() not in strong_dicts:
                    self.dict_module_combo.setCurrentText(strong_dicts[0])

            self.dict_input.setText(term)
            self.lookup_dictionary_term(term)
            self.dict_dock.raise_()

        elif any(url.lower().startswith(p) for p in ("goto:", "xref:", "b:")):
            target = re.sub(r"^(?:goto|xref|[Bb]):", "", url).strip()
            target = self.resolve_numeric_reference(target)
            self.navigate_to_reference(target)

        elif url.startswith("verse:"):
            v_num = url.replace("verse:", "")
            self.statusBar.showMessage(f"Verse {v_num} selected")

    def navigate_to_reference(self, target_ref: str):
        """Parses reference and navigates to target book and chapter."""
        target_ref = self.resolve_numeric_reference(target_ref)
        clean = target_ref.replace("/", " ").replace("–", "-").strip()
        parts = clean.split()
        if not parts:
            return

        book_cand = ""
        ch_cand = ""
        v_cand = ""

        # Case 0: Numeric book ID directly (e.g. "10 1:1" or "470 1:1")
        if (
            len(parts) >= 2
            and parts[0].isdigit()
            and int(parts[0]) >= 10
            and (":" in parts[1] or parts[1].isdigit())
        ):
            book_cand = parts[0]
            rem = parts[1]
            if ":" in rem:
                ch_cand, v_cand = rem.split(":", 1)
            else:
                ch_cand = rem
        # Case 1: Numbered book, e.g. "2 Sam. 7:12" or "1 John 1:9"
        elif len(parts) >= 3 and parts[0].isdigit():
            book_cand = f"{parts[0]} {parts[1]}"
            rem = parts[2]
            if ":" in rem:
                ch_cand, v_cand = rem.split(":", 1)
            else:
                ch_cand = rem
        # Case 2: Standard book, e.g. "Gen. 21:3" or "Luke 3:23-38"
        elif len(parts) >= 2:
            book_cand = parts[0]
            rem = parts[1]
            if ":" in rem:
                ch_cand, v_cand = rem.split(":", 1)
            else:
                ch_cand = rem

        # Clean chapter and verse numbers
        ch_cand = ch_cand.split("-")[0].strip()
        v_cand = v_cand.split("-")[0].split(",")[0].strip()

        if book_cand and ch_cand.isdigit():
            self.set_active_book(book_cand)
            self.current_chapter = int(ch_cand)
            self.chapter_combo.setCurrentText(str(self.current_chapter))
            self.load_active_chapter()
            if v_cand.isdigit():
                self.reader_browser.scrollToAnchor(f"v{v_cand}")

    def toggle_double_click_lookup(self, checked: bool):
        self.lookup_on_double_click = checked
        self.settings.setValue("lookupOnDoubleClick", checked)
        state_str = "enabled" if checked else "disabled"
        self.statusBar.showMessage(f"Double-click dictionary lookup {state_str}.", 3000)

    def on_word_double_clicked(self, word: str):
        if not self.lookup_on_double_click:
            return
        clean_word = word.strip()
        if clean_word and len(clean_word) >= 2:
            self.dict_input.setText(clean_word)
            self.lookup_dictionary_term(clean_word)
            self.dict_dock.raise_()

    def lookup_dictionary_term(self, term: Optional[str] = None):
        if term is None:
            term = self.dict_input.text().strip()
        else:
            self.dict_input.setText(term)

        if not term:
            return

        dict_module = (
            self.dict_module_combo.currentText()
            if hasattr(self, "dict_module_combo")
            else None
        )
        if dict_module == "None installed":
            dict_module = None

        data = self.backend.get_dict_entry(term, module=dict_module)
        if data:
            resolved_module = data.get("module")
            if (
                resolved_module
                and hasattr(self, "dict_module_combo")
                and self.dict_module_combo.findText(resolved_module) >= 0
                and self.dict_module_combo.currentText() != resolved_module
            ):
                self.dict_module_combo.blockSignals(True)
                self.dict_module_combo.setCurrentText(resolved_module)
                self.dict_module_combo.blockSignals(False)

            html = render_dict_html(
                data, theme=self.current_theme, font_size=self.font_size
            )
            self.dict_browser.setHtml(html)
            self.dict_dock.raise_()
            self.statusBar.showMessage(
                f"Looked up '{term}' in {resolved_module or dict_module}"
            )
        else:
            self.dict_browser.setHtml(
                f"<p style='padding: 12px;'>No definition found for '{term}'.</p>"
            )

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
        self.update_dictionary_combo()
        self.update_devotion_combo()
        self.load_books()
        self.load_active_chapter()
        self.load_active_devotion()
        self.statusBar.showMessage("Modules successfully reloaded.")

    # --- Themes & Typography ---

    def on_theme_changed(self):
        text = self.theme_combo.currentText().lower()
        if "sepia" in text:
            self.apply_theme("sepia")
        elif "dark" in text:
            self.apply_theme("dark")
        else:
            self.apply_theme("light")

    def increase_font_size(self):
        if self.font_size < 28:
            self.font_size += 1
            self.load_active_chapter()
            self.load_active_devotion()

    def decrease_font_size(self):
        if self.font_size > 10:
            self.font_size -= 1
            self.load_active_chapter()
            self.load_active_devotion()

    def closeEvent(self, event):
        """Save window settings, dock layout, and cleanly terminate backend process."""
        try:
            self.save_current_passage_note()
            self.settings.setValue("geometry", self.saveGeometry())
            self.settings.setValue("windowState", self.saveState())
            self.settings.setValue("theme", self.current_theme)
            self.settings.setValue("fontSize", self.font_size)
            if hasattr(self, "bible_combo"):
                self.settings.setValue("activeBible", self.bible_combo.currentText())
            self.settings.setValue("activeBook", self.current_book_number)
            self.settings.setValue("activeChapter", self.current_chapter)
            if (
                hasattr(self, "dict_module_combo")
                and self.dict_module_combo.currentText()
            ):
                self.settings.setValue(
                    "activeDictionary", self.dict_module_combo.currentText()
                )
            if (
                hasattr(self, "devotion_module_combo")
                and self.devotion_module_combo.currentText()
            ):
                self.settings.setValue(
                    "activeDevotion", self.devotion_module_combo.currentText()
                )
            self.settings.setValue("lookupOnDoubleClick", self.lookup_on_double_click)
        except Exception as e:
            print(f"Error saving window settings: {e}")

        if hasattr(self, "backend") and self.backend:
            self.backend.close()
        event.accept()
