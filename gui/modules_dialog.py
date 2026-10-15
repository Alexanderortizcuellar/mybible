"""Modules Manager Dialog for MyBible Desktop.
Allows browsing the 4,000+ module repository with language selection,
module type filters, instant search, and one-click downloading.
"""

from typing import Optional

from PySide6.QtCore import QLocale, Qt, Signal, QThread
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QComboBox,
    QDialog,
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QMessageBox,
    QProgressBar,
    QPushButton,
    QTabWidget,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget,
)

from gui.backend import RustBackendClient


class DownloadWorker(QThread):
    """Worker thread for downloading a module without blocking the GUI event loop."""

    downloadFinished = Signal(bool, str, str)  # success, abbr, message

    def __init__(self, backend: RustBackendClient, abbreviation: str, parent=None):
        super().__init__(parent)
        self.backend = backend
        self.abbreviation = abbreviation

    def run(self):
        try:
            res = self.backend.download_module(self.abbreviation)
            if res and res.get("success"):
                self.downloadFinished.emit(True, self.abbreviation, f"Module '{self.abbreviation}' downloaded and loaded.")
            else:
                msg = res.get("error", "Unknown error") if res else "Download request failed."
                self.downloadFinished.emit(False, self.abbreviation, msg)
        except Exception as e:
            self.downloadFinished.emit(False, self.abbreviation, str(e))


class ModulesDialog(QDialog):
    """Dialog for managing local and downloadable MyBible modules."""

    modulesUpdated = Signal()

    def __init__(self, backend: RustBackendClient, parent=None):
        super().__init__(parent)
        self.backend = backend
        self.catalog_data: Optional[dict] = None
        self.active_workers: list[DownloadWorker] = []

        self.setWindowTitle("Module Manager — MyBible")
        self.resize(950, 680)
        self.init_ui()
        self.load_installed()
        self.load_catalog_filter()

    def init_ui(self):
        layout = QVBoxLayout(self)

        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        # Tab 1: Explore & Download Online Catalog
        self.tab_catalog = QWidget()
        cat_layout = QVBoxLayout(self.tab_catalog)

        # Top Controls: Language, Type, Search Filter
        filter_row = QHBoxLayout()

        # Language dropdown
        filter_row.addWidget(QLabel("<b>Language:</b>"))
        self.lang_combo = QComboBox()
        self.lang_combo.setMinimumWidth(160)
        self.lang_combo.currentIndexChanged.connect(self.on_filter_changed)
        filter_row.addWidget(self.lang_combo)

        # Module type dropdown
        filter_row.addWidget(QLabel("<b>Type:</b>"))
        self.type_combo = QComboBox()
        self.type_combo.addItem("All Types", userData="all")
        self.type_combo.addItem("Bibles", userData="bible")
        self.type_combo.addItem("Commentaries", userData="commentaries")
        self.type_combo.addItem("Dictionaries", userData="dictionary")
        self.type_combo.addItem("Cross References", userData="crossreferences")
        self.type_combo.addItem("Subheadings", userData="subheadings")
        self.type_combo.addItem("Reading Plans", userData="readingplan")
        self.type_combo.addItem("Devotions", userData="devotions")
        self.type_combo.currentIndexChanged.connect(self.on_filter_changed)
        filter_row.addWidget(self.type_combo)

        # Search box
        filter_row.addWidget(QLabel("<b>Filter:</b>"))
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Filter by name or abbreviation...")
        self.search_input.textChanged.connect(self.on_filter_changed)
        filter_row.addWidget(self.search_input)

        refresh_cat_btn = QPushButton("Refresh Catalog")
        refresh_cat_btn.clicked.connect(self.load_catalog_filter)
        filter_row.addWidget(refresh_cat_btn)

        cat_layout.addLayout(filter_row)

        # Catalog Modules Table
        self.table_catalog = QTableWidget(0, 6)
        self.table_catalog.setHorizontalHeaderLabels([
            "Abbr", "Type", "Language", "Size", "Description", "Action"
        ])
        header = self.table_catalog.horizontalHeader()
        header.setSectionResizeMode(0, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(1, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(2, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(3, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(4, QHeaderView.Stretch)
        header.setSectionResizeMode(5, QHeaderView.ResizeToContents)
        self.table_catalog.setAlternatingRowColors(True)
        cat_layout.addWidget(self.table_catalog)

        # Catalog Status Bar & Progress
        self.cat_status = QLabel("Loading catalog...")
        cat_layout.addWidget(self.cat_status)

        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 0)
        self.progress_bar.setVisible(False)
        cat_layout.addWidget(self.progress_bar)

        self.tabs.addTab(self.tab_catalog, "Explore Online Repository (4,000+)")

        # Tab 2: Locally Installed Modules
        self.tab_installed = QWidget()
        inst_layout = QVBoxLayout(self.tab_installed)

        self.dir_label = QLabel("Active Data Directory: ...")
        self.dir_label.setStyleSheet("font-weight: bold; color: #2b6cb0; margin-bottom: 6px;")
        inst_layout.addWidget(self.dir_label)

        self.table_installed = QTableWidget(0, 3)
        self.table_installed.setHorizontalHeaderLabels(["Category", "Abbreviation", "Status"])
        self.table_installed.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)
        self.table_installed.setAlternatingRowColors(True)
        inst_layout.addWidget(self.table_installed)

        refresh_inst_btn = QPushButton("Reload Local Modules")
        refresh_inst_btn.clicked.connect(self.load_installed)
        inst_layout.addWidget(refresh_inst_btn)

        self.tabs.addTab(self.tab_installed, "Installed Local Modules")

    # --- Language Detection & Catalog Loading ---

    def detect_default_language(self, available_languages: list[dict]) -> str:
        """Detects the default language based on the system locale."""
        sys_locale = QLocale.system().name().lower()  # e.g. "es_co", "en_us", "ru_ru"

        # Check prefix match (e.g. "es", "en", "ru")
        prefix = sys_locale.split("_")[0] if "_" in sys_locale else sys_locale[:2]

        for lang in available_languages:
            code = lang.get("code", "").lower()
            if code == prefix or code.startswith(prefix):
                return code

        # Default fallback to Spanish if present, otherwise English, otherwise "all"
        for candidate in ["es", "en"]:
            for lang in available_languages:
                if lang.get("code", "").lower() == candidate:
                    return candidate

        return "all"

    def load_catalog_filter(self):
        """Fetches catalog from backend with current filter parameters."""
        self.cat_status.setText("Fetching official module registry...")
        self.progress_bar.setVisible(True)

        lang = self.lang_combo.currentData() if self.lang_combo.count() > 0 else None
        mtype = self.type_combo.currentData() if self.type_combo.count() > 0 else "all"
        query = self.search_input.text().strip()

        data = self.backend.get_catalog(query=query, language=lang, mod_type=mtype)
        self.progress_bar.setVisible(False)

        if not data:
            self.cat_status.setText("Failed to load catalog from server.")
            return

        self.catalog_data = data

        # Populate language dropdown if not populated yet
        if self.lang_combo.count() == 0:
            self.lang_combo.blockSignals(True)
            self.lang_combo.addItem("All Languages", userData="all")

            languages = data.get("languages", [])
            detected = self.detect_default_language(languages)

            selected_idx = 0
            for idx, l in enumerate(languages):
                code = l.get("code", "")
                name = l.get("name", code)
                count = l.get("count", 0)
                label = f"{name} ({count})"
                self.lang_combo.addItem(label, userData=code)
                if code.lower() == detected.lower():
                    selected_idx = idx + 1  # +1 for "All Languages"

            self.lang_combo.setCurrentIndex(selected_idx)
            self.lang_combo.blockSignals(False)

            # Re-fetch with detected language
            if selected_idx != 0:
                self.on_filter_changed()
                return

        self.populate_catalog_table(data.get("modules", []))

    def on_filter_changed(self):
        """Called whenever language, type, or query filter changes."""
        lang = self.lang_combo.currentData() or "all"
        mtype = self.type_combo.currentData() or "all"
        query = self.search_input.text().strip()

        self.cat_status.setText("Filtering modules...")
        data = self.backend.get_catalog(query=query, language=lang, mod_type=mtype)
        if data:
            self.populate_catalog_table(data.get("modules", []))

    def populate_catalog_table(self, modules: list[dict]):
        """Populates the catalog table with downloadable module cards."""
        self.table_catalog.setRowCount(0)
        self.table_catalog.setRowCount(len(modules))

        for row, m in enumerate(modules):
            abbr = m.get("abbreviation", "")
            mtype = m.get("module_type", "")
            lang = m.get("language", "") or ""
            size = m.get("size_str", "") or ""
            desc = m.get("description", "")
            is_installed = m.get("is_installed", False)

            item_abbr = QTableWidgetItem(abbr)
            item_abbr.setFont(QFont("Segoe UI", 9, QFont.Bold))
            self.table_catalog.setItem(row, 0, item_abbr)

            self.table_catalog.setItem(row, 1, QTableWidgetItem(mtype))
            self.table_catalog.setItem(row, 2, QTableWidgetItem(lang.upper()))
            self.table_catalog.setItem(row, 3, QTableWidgetItem(size))
            self.table_catalog.setItem(row, 4, QTableWidgetItem(desc))

            if is_installed:
                installed_label = QLabel("✓ Installed")
                installed_label.setAlignment(Qt.AlignCenter)
                installed_label.setStyleSheet("color: #2e7d32; font-weight: bold; padding: 2px;")
                self.table_catalog.setCellWidget(row, 5, installed_label)
            else:
                dl_btn = QPushButton("Download")
                dl_btn.setStyleSheet("""
                    QPushButton {
                        background-color: #2b6cb0;
                        color: white;
                        border-radius: 4px;
                        padding: 3px 10px;
                        font-weight: bold;
                    }
                    QPushButton:hover {
                        background-color: #3182ce;
                    }
                """)
                dl_btn.clicked.connect(lambda _, a=abbr, r=row: self.start_download(a, r))
                self.table_catalog.setCellWidget(row, 5, dl_btn)

        count = len(modules)
        self.cat_status.setText(f"Displaying {count} modules from repository.")

    def start_download(self, abbreviation: str, row: int):
        """Starts background download for the selected module."""
        # Replace button with downloading label
        spin_label = QLabel("Downloading...")
        spin_label.setAlignment(Qt.AlignCenter)
        spin_label.setStyleSheet("color: #d69e2e; font-weight: bold;")
        self.table_catalog.setCellWidget(row, 5, spin_label)
        self.cat_status.setText(f"Downloading module '{abbreviation}'...")
        self.progress_bar.setVisible(True)

        worker = DownloadWorker(self.backend, abbreviation, self)
        worker.downloadFinished.connect(lambda ok, a, msg: self.on_download_finished(ok, a, msg, row))
        self.active_workers.append(worker)
        worker.start()

    def on_download_finished(self, success: bool, abbreviation: str, message: str, row: int):
        self.progress_bar.setVisible(False)
        if success:
            installed_label = QLabel("✓ Installed")
            installed_label.setAlignment(Qt.AlignCenter)
            installed_label.setStyleSheet("color: #2e7d32; font-weight: bold; padding: 2px;")
            self.table_catalog.setCellWidget(row, 5, installed_label)
            self.cat_status.setText(f"Successfully downloaded '{abbreviation}'.")
            self.load_installed()
            self.modulesUpdated.emit()
            QMessageBox.information(self, "Download Complete", f"Module '{abbreviation}' was successfully installed and loaded into MyBible!")
        else:
            retry_btn = QPushButton("Retry")
            retry_btn.clicked.connect(lambda _, a=abbreviation, r=row: self.start_download(a, r))
            self.table_catalog.setCellWidget(row, 5, retry_btn)
            self.cat_status.setText(f"Download failed: {message}")
            QMessageBox.critical(self, "Download Failed", f"Could not download '{abbreviation}':\n{message}")

    # --- Installed Modules Tab ---

    def load_installed(self):
        """Loads and lists currently installed local modules."""
        info = self.backend.get_modules()
        self.dir_label.setText(f"Active Data Directory: {info.get('data_dir', 'modules')}")

        categories = [
            ("Bible", info.get("bibles", [])),
            ("Commentary", info.get("commentaries", [])),
            ("Dictionary", info.get("dictionaries", [])),
            ("Cross-Reference", info.get("cross_references", [])),
            ("Subheading", info.get("subheadings", [])),
            ("Reading Plan", info.get("reading_plans", [])),
            ("Devotional", info.get("devotions", [])),
        ]

        total_rows = sum(len(items) for _, items in categories)
        self.table_installed.setRowCount(total_rows)

        row = 0
        for cat_name, items in categories:
            for item in items:
                self.table_installed.setItem(row, 0, QTableWidgetItem(cat_name))
                self.table_installed.setItem(row, 1, QTableWidgetItem(item))
                status_item = QTableWidgetItem("Loaded")
                status_item.setForeground(QColor("#2e7d32"))
                self.table_installed.setItem(row, 2, status_item)
                row += 1
