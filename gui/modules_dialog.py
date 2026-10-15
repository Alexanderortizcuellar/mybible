"""
Modules Manager Dialog for MyBible GUI
Enables viewing installed modules and searching/downloading modules from the 4,000+ repository.
"""

from typing import Optional
from PySide6.QtCore import Signal
from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QTabWidget,
    QWidget,
    QTableWidget,
    QTableWidgetItem,
    QHeaderView,
    QLineEdit,
    QPushButton,
    QLabel,
    QTextEdit,
    QProgressBar,
    QMessageBox,
)

from gui.backend import RustBackendClient


class ModulesDialog(QDialog):
    """Module management dialog for installed and online repository modules."""

    modulesUpdated = Signal()

    def __init__(self, backend: RustBackendClient, parent=None):
        super().__init__(parent)
        self.backend = backend
        self.download_process = None

        self.setWindowTitle("Module Manager — MyBible")
        self.resize(850, 600)
        self.init_ui()
        self.load_installed()

    def init_ui(self):
        layout = QVBoxLayout(self)

        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        # Tab 1: Installed Modules
        self.tab_installed = QWidget()
        inst_layout = QVBoxLayout(self.tab_installed)

        self.dir_label = QLabel("Data Directory: ...")
        inst_layout.addWidget(self.dir_label)

        self.table_installed = QTableWidget(0, 3)
        self.table_installed.setHorizontalHeaderLabels(["Category", "Abbreviation", "Status"])
        self.table_installed.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)
        inst_layout.addWidget(self.table_installed)

        refresh_btn = QPushButton("Refresh Installed Modules")
        refresh_btn.clicked.connect(self.load_installed)
        inst_layout.addWidget(refresh_btn)

        self.tabs.addTab(self.tab_installed, "Installed Modules")

        # Tab 2: Online Repository Catalog
        self.tab_online = QWidget()
        online_layout = QVBoxLayout(self.tab_online)

        search_layout = QHBoxLayout()
        self.repo_input = QLineEdit()
        self.repo_input.setPlaceholderText("Search online modules (e.g. Spanish, King James, Greek)...")
        self.repo_input.returnPressed.connect(self.search_online)
        search_layout.addWidget(self.repo_input)

        self.search_repo_btn = QPushButton("Search Catalog")
        self.search_repo_btn.clicked.connect(self.search_online)
        search_layout.addWidget(self.search_repo_btn)
        online_layout.addLayout(search_layout)

        self.table_online = QTableWidget(0, 5)
        self.table_online.setHorizontalHeaderLabels(["Abbrev", "Type", "Language", "Size", "Description"])
        self.table_online.horizontalHeader().setSectionResizeMode(4, QHeaderView.Stretch)
        self.table_online.setSelectionBehavior(QTableWidget.SelectRows)
        online_layout.addWidget(self.table_online)

        action_layout = QHBoxLayout()
        self.download_btn = QPushButton("Download Selected Module")
        self.download_btn.clicked.connect(self.download_selected)
        action_layout.addWidget(self.download_btn)
        online_layout.addLayout(action_layout)

        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        self.progress_bar.setRange(0, 0)
        online_layout.addWidget(self.progress_bar)

        self.log_output = QTextEdit()
        self.log_output.setReadOnly(True)
        self.log_output.setMaximumHeight(120)
        online_layout.addWidget(self.log_output)

        self.tabs.addTab(self.tab_online, "Download Modules")

    def load_installed(self):
        """Loads and lists currently installed modules."""
        modules_data = self.backend.get_modules()
        self.dir_label.setText(f"📁 Active Data Directory: {modules_data.get('data_dir', '')}")

        self.table_installed.setRowCount(0)
        categories = [
            ("Bible", modules_data.get("bibles", [])),
            ("Commentary", modules_data.get("commentaries", [])),
            ("Dictionary", modules_data.get("dictionaries", [])),
            ("Cross-Reference", modules_data.get("cross_references", [])),
            ("Subheading", modules_data.get("subheadings", [])),
            ("Reading Plan", modules_data.get("reading_plans", [])),
            ("Devotional", modules_data.get("devotions", [])),
        ]

        row = 0
        for cat_name, items in categories:
            for item in items:
                self.table_installed.insertRow(row)
                self.table_installed.setItem(row, 0, QTableWidgetItem(cat_name))
                self.table_installed.setItem(row, 1, QTableWidgetItem(item))
                self.table_installed.setItem(row, 2, QTableWidgetItem("Installed"))
                row += 1

    def search_online(self):
        query = self.repo_input.text().strip()
        if not query:
            return

        self.search_repo_btn.setEnabled(False)
        self.progress_bar.setVisible(True)
        self.log_output.append(f"Searching online catalog for '{query}'...")

        results = self.backend.search_repository(query)

        self.search_repo_btn.setEnabled(True)
        self.progress_bar.setVisible(False)

        self.table_online.setRowCount(0)
        for row, item in enumerate(results):
            self.table_online.insertRow(row)
            self.table_online.setItem(row, 0, QTableWidgetItem(item.get("abbreviation", "")))
            self.table_online.setItem(row, 1, QTableWidgetItem(item.get("module_type", "")))
            self.table_online.setItem(row, 2, QTableWidgetItem(item.get("language", "-")))
            self.table_online.setItem(row, 3, QTableWidgetItem(item.get("size_str", "-")))
            self.table_online.setItem(row, 4, QTableWidgetItem(item.get("description", "")))

        self.log_output.append(f"Found {len(results)} matching modules.")

    def download_selected(self):
        selected_rows = self.table_online.selectedIndexes()
        if not selected_rows:
            QMessageBox.information(self, "Selection", "Please select a module to download.")
            return

        row = selected_rows[0].row()
        abbrev = self.table_online.item(row, 0).text()

        self.download_btn.setEnabled(False)
        self.progress_bar.setVisible(True)
        self.log_output.append(f"\nStarting download for '{abbrev}'...")

        def on_stream(text: str):
            self.log_output.append(text.strip())

        def on_done(exit_code: int):
            self.progress_bar.setVisible(False)
            self.download_btn.setEnabled(True)
            if exit_code == 0:
                self.log_output.append(f"✓ Module '{abbrev}' successfully installed!\n")
                self.load_installed()
                self.modulesUpdated.emit()
            else:
                self.log_output.append(f"❌ Failed to install module '{abbrev}' (exit code {exit_code}).\n")

        self.download_process = self.backend.start_download(abbrev, on_stream, on_done)
