"""Professional Study Tool Styling System for MyBible Desktop.
Provides high-contrast, cohesive styling across light, sepia, and dark study themes.
"""

import os
from PySide6.QtGui import QColor, QPalette

ICONS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "icons")).replace(
    "\\", "/"
)


def get_theme_palette(theme_name: str = "light") -> QPalette:
    """Returns a full QPalette ensuring all native Qt controls have crisp, readable contrast."""
    pal = QPalette()
    t = theme_name.lower()

    if "dark" in t:
        bg_dark = QColor("#141824")
        card_bg = QColor("#1e2538")
        input_bg = QColor("#283149")
        text_white = QColor("#f7fafc")
        accent = QColor("#3182ce")

        pal.setColor(QPalette.Window, bg_dark)
        pal.setColor(QPalette.WindowText, text_white)
        pal.setColor(QPalette.Base, input_bg)
        pal.setColor(QPalette.AlternateBase, card_bg)
        pal.setColor(QPalette.ToolTipBase, card_bg)
        pal.setColor(QPalette.ToolTipText, text_white)
        pal.setColor(QPalette.Text, text_white)
        pal.setColor(QPalette.Button, card_bg)
        pal.setColor(QPalette.ButtonText, text_white)
        pal.setColor(QPalette.BrightText, QColor("#fc8181"))
        pal.setColor(QPalette.Highlight, accent)
        pal.setColor(QPalette.HighlightedText, text_white)

    elif "sepia" in t:
        bg_sepia = QColor("#f4ecd8")
        card_bg = QColor("#fbf7ed")
        text_dark = QColor("#3d2f1d")
        accent = QColor("#8c5b2a")

        pal.setColor(QPalette.Window, bg_sepia)
        pal.setColor(QPalette.WindowText, text_dark)
        pal.setColor(QPalette.Base, card_bg)
        pal.setColor(QPalette.AlternateBase, bg_sepia)
        pal.setColor(QPalette.ToolTipBase, card_bg)
        pal.setColor(QPalette.ToolTipText, text_dark)
        pal.setColor(QPalette.Text, text_dark)
        pal.setColor(QPalette.Button, card_bg)
        pal.setColor(QPalette.ButtonText, text_dark)
        pal.setColor(QPalette.Highlight, accent)
        pal.setColor(QPalette.HighlightedText, QColor("#ffffff"))

    else:  # Light mode (default)
        bg_light = QColor("#f0f4f8")
        card_bg = QColor("#ffffff")
        text_dark = QColor("#1a202c")
        accent = QColor("#2b6cb0")

        pal.setColor(QPalette.Window, bg_light)
        pal.setColor(QPalette.WindowText, text_dark)
        pal.setColor(QPalette.Base, card_bg)
        pal.setColor(QPalette.AlternateBase, bg_light)
        pal.setColor(QPalette.ToolTipBase, QColor("#2d3748"))
        pal.setColor(QPalette.ToolTipText, QColor("#ffffff"))
        pal.setColor(QPalette.Text, text_dark)
        pal.setColor(QPalette.Button, QColor("#edf2f7"))
        pal.setColor(QPalette.ButtonText, text_dark)
        pal.setColor(QPalette.Highlight, accent)
        pal.setColor(QPalette.HighlightedText, QColor("#ffffff"))

    return pal


def get_theme_stylesheet(theme_name: str = "light") -> str:
    """Returns a comprehensive, professional stylesheet with clear visual hierarchy."""
    t = theme_name.lower()

    if "dark" in t:
        return (
            """
        /* === PROFESSIONAL DARK THEME === */
        QMainWindow {
            background-color: #141824;
            color: #f7fafc;
        }
        QWidget {
            color: #f7fafc;
            font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, 'Roboto', sans-serif;
            font-size: 13px;
        }

        /* Menu Bar */
        QMenuBar {
            background-color: #0f121d;
            color: #e2e8f0;
            border-bottom: 1px solid #232b3e;
            padding: 3px 6px;
            font-size: 13px;
        }
        QMenuBar::item {
            background: transparent;
            padding: 5px 10px;
            border-radius: 4px;
            color: #e2e8f0;
        }
        QMenuBar::item:selected {
            background-color: #232b3e;
            color: #ffffff;
        }
        QMenu {
            background-color: #1e2538;
            color: #f7fafc;
            border: 1px solid #36425d;
            border-radius: 6px;
            padding: 5px;
        }
        QMenu::item {
            padding: 6px 26px 6px 14px;
            border-radius: 4px;
            color: #f7fafc;
        }
        QMenu::item:selected {
            background-color: #3182ce;
            color: #ffffff;
        }
        QMenu::separator {
            height: 1px;
            background-color: #2d3748;
            margin: 4px 6px;
        }

        /* Toolbar */
        QToolBar {
            background-color: #192032;
            border-bottom: 1px solid #28334a;
            spacing: 8px;
            padding: 6px 12px;
        }
        QToolBar QLabel {
            color: #cbd5e0;
            font-weight: 600;
            font-size: 12px;
            margin-right: 2px;
        }
        QToolBar::separator {
            width: 1px;
            background-color: #2d3a52;
            margin: 4px 6px;
        }

        /* ComboBox Dropdowns */
        QComboBox {
            background-color: #232b3e;
            color: #ffffff !important;
            border: 1px solid #36425d;
            border-radius: 5px;
            padding: 2px 24px 2px 8px;
            min-height: 26px;
            font-size: 13px;
            font-weight: 500;
        }
        QComboBox:hover {
            border-color: #4299e1;
        }
        QComboBox:focus {
            border-color: #3182ce;
        }
        QComboBox::drop-down {
            subcontrol-origin: padding;
            subcontrol-position: top right;
            width: 18px;
            border-left: 1px solid #36425d;
            border-top-right-radius: 4px;
            border-bottom-right-radius: 4px;
            background: #1e2538;
        }
        QComboBox::down-arrow {
            image: url(@ARROW_DOWN@);
            width: 10px;
            height: 10px;
        }
        QComboBox QAbstractItemView {
            background-color: #1e2538;
            color: #ffffff !important;
            border: 1px solid #36425d;
            border-radius: 4px;
            selection-background-color: #3182ce;
            selection-color: #ffffff !important;
            padding: 4px;
            outline: none;
        }
        QComboBox QAbstractItemView::item {
            min-height: 24px;
            padding: 4px 8px;
            color: #ffffff;
        }
        QComboBox QAbstractItemView::item:selected {
            background-color: #3182ce;
            color: #ffffff;
        }

        /* Inputs */
        QLineEdit {
            background-color: #232b3e;
            color: #ffffff !important;
            border: 1px solid #36425d;
            border-radius: 5px;
            padding: 4px 8px;
            min-height: 24px;
            selection-background-color: #3182ce;
        }
        QLineEdit:focus {
            border-color: #3182ce;
        }

        /* Buttons */
        QPushButton {
            background-color: #232b3e;
            color: #f7fafc;
            border: 1px solid #36425d;
            border-radius: 5px;
            padding: 5px 12px;
            font-weight: 600;
            min-height: 22px;
        }
        QPushButton:hover {
            background-color: #2d374f;
            border-color: #4299e1;
            color: #ffffff;
        }
        QPushButton:pressed {
            background-color: #171d2b;
        }

        /* Primary Action Buttons */
        QPushButton#primaryBtn {
            background-color: #2b6cb0;
            color: #ffffff;
            border: 1px solid #3182ce;
        }
        QPushButton#primaryBtn:hover {
            background-color: #3182ce;
            border-color: #4299e1;
        }

        /* Nav Buttons (◀ ▶) */
        QPushButton#navBtn {
            font-size: 13px;
            font-weight: bold;
            padding: 1px 2px;
            min-width: 26px;
        }

        /* Font Buttons (A- A+) */
        QPushButton#fontBtn {
            font-size: 11px;
            font-weight: bold;
            padding: 1px 1px;
            min-width: 26px;
        }

        /* Tree and Tables */
        QTreeWidget, QTableWidget {
            background-color: #171d2b;
            color: #f7fafc;
            border: 1px solid #28334a;
            border-radius: 4px;
            outline: none;
        }
        QTreeWidget::item {
            padding: 4px 6px;
            border-radius: 3px;
        }
        QTreeWidget::item:hover {
            background-color: #232b3e;
        }
        QTreeWidget::item:selected {
            background-color: #2b6cb0;
            color: #ffffff;
        }
        QHeaderView::section {
            background-color: #111520;
            color: #a0aec0;
            padding: 6px 8px;
            border: none;
            border-bottom: 1px solid #28334a;
            font-weight: 600;
        }

        /* Docks */
        QDockWidget {
            color: #f7fafc;
            font-weight: bold;
        }
        QDockWidget::title {
            background-color: #192032;
            color: #e2e8f0;
            padding: 7px 12px;
            padding-right: 58px;
            border-bottom: 2px solid #2b6cb0;
            font-size: 12px;
            font-weight: bold;
        }
        QDockWidget::close-button {
            subcontrol-position: top right;
            subcontrol-origin: margin;
            position: absolute;
            top: 7px;
            right: 8px;
            width: 18px;
            height: 18px;
            border: none;
            border-radius: 3px;
            image: url(@DOCK_CLOSE@);
            background: transparent;
        }
        QDockWidget::close-button:hover {
            background-color: rgba(239, 68, 68, 0.25);
            image: url(@DOCK_CLOSE_HOVER@);
        }
        QDockWidget::float-button {
            subcontrol-position: top right;
            subcontrol-origin: margin;
            position: absolute;
            top: 7px;
            right: 32px;
            width: 18px;
            height: 18px;
            border: none;
            border-radius: 3px;
            image: url(@DOCK_FLOAT@);
            background: transparent;
        }
        QDockWidget::float-button:hover {
            background-color: #2d3a52;
        }

        /* Tab Widget */
        QTabWidget::pane {
            border: 1px solid #28334a;
            background-color: #171d2b;
            border-radius: 4px;
        }
        QTabBar::tab {
            background-color: #111520;
            color: #a0aec0;
            padding: 6px 14px;
            border: 1px solid #28334a;
            border-bottom: none;
            border-top-left-radius: 5px;
            border-top-right-radius: 5px;
            margin-right: 2px;
            font-weight: 600;
        }
        QTabBar::tab:selected {
            background-color: #171d2b;
            color: #63b3ed;
            border-bottom: 2px solid #3182ce;
        }
        QTabBar::tab:hover:!selected {
            background-color: #1e2538;
            color: #e2e8f0;
        }

        /* Status Bar */
        QStatusBar {
            background-color: #0f121d;
            color: #a0aec0;
            border-top: 1px solid #232b3e;
            font-size: 11px;
            padding: 3px 8px;
        }

        /* Tooltip */
        QToolTip {
            background-color: #1e2538;
            color: #f7fafc;
            border: 1px solid #36425d;
            border-radius: 4px;
            padding: 6px 10px;
            font-size: 12px;
        }

        /* CheckBox */
        QCheckBox {
            color: #e2e8f0;
            spacing: 6px;
            font-weight: 500;
        }
        QCheckBox::indicator {
            width: 16px;
            height: 16px;
            border-radius: 3px;
            border: 1px solid #36425d;
            background: #232b3e;
        }
        QCheckBox::indicator:checked {
            background-color: #3182ce;
            border-color: #3182ce;
        }
        """.replace("@ARROW_DOWN@", f"{ICONS_DIR}/arrow_down_white.png")
            .replace("@DOCK_CLOSE@", f"{ICONS_DIR}/dock_close_white.png")
            .replace("@DOCK_CLOSE_HOVER@", f"{ICONS_DIR}/dock_close_red.png")
            .replace("@DOCK_FLOAT@", f"{ICONS_DIR}/dock_float_white.png")
        )

    elif "sepia" in t:
        return (
            """
        /* === PROFESSIONAL SEPIA THEME === */
        QMainWindow {
            background-color: #f4ecd8;
            color: #3d2f1d;
        }
        QWidget {
            color: #3d2f1d;
            font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, 'Roboto', sans-serif;
            font-size: 13px;
        }

        QMenuBar {
            background-color: #ede2c8;
            color: #3d2f1d;
            border-bottom: 1px solid #dcd0b6;
            padding: 3px 6px;
        }
        QMenuBar::item:selected {
            background-color: #e2d4b7;
        }
        QMenu {
            background-color: #fbf7ed;
            color: #3d2f1d;
            border: 1px solid #dcd0b6;
            padding: 4px;
        }
        QMenu::item:selected {
            background-color: #8c5b2a;
            color: #ffffff;
        }

        QToolBar {
            background-color: #ece1c6;
            border-bottom: 1px solid #dcd0b6;
            spacing: 8px;
            padding: 6px 12px;
        }
        QToolBar QLabel {
            color: #4a3823;
            font-weight: 600;
            font-size: 12px;
        }

        QComboBox {
            background-color: #fbf7ed;
            color: #3d2f1d !important;
            border: 1px solid #c9bca0;
            border-radius: 5px;
            padding: 2px 24px 2px 8px;
            min-height: 26px;
            font-size: 13px;
            font-weight: 500;
        }
        QComboBox:hover {
            border-color: #8c5b2a;
        }
        QComboBox::drop-down {
            subcontrol-origin: padding;
            subcontrol-position: top right;
            width: 18px;
            border-left: 1px solid #c9bca0;
            background: #ede3cb;
        }
        QComboBox::down-arrow {
            image: url(@ARROW_DOWN@);
            width: 10px;
            height: 10px;
        }
        QComboBox QAbstractItemView {
            background-color: #fbf7ed;
            color: #3d2f1d !important;
            border: 1px solid #c9bca0;
            selection-background-color: #8c5b2a;
            selection-color: #ffffff !important;
            padding: 4px;
        }

        QLineEdit {
            background-color: #fbf7ed;
            color: #3d2f1d !important;
            border: 1px solid #c9bca0;
            border-radius: 5px;
            padding: 4px 8px;
            min-height: 24px;
        }

        QPushButton {
            background-color: #fbf7ed;
            color: #3d2f1d;
            border: 1px solid #c9bca0;
            border-radius: 5px;
            padding: 5px 12px;
            font-weight: 600;
            min-height: 22px;
        }
        QPushButton:hover {
            background-color: #f0e6ce;
            border-color: #8c5b2a;
        }
        QPushButton#primaryBtn {
            background-color: #8c5b2a;
            color: #ffffff;
            border: 1px solid #8c5b2a;
        }
        QPushButton#primaryBtn:hover {
            background-color: #a06c3a;
        }

        /* Nav Buttons (◀ ▶) */
        QPushButton#navBtn {
            font-size: 13px;
            font-weight: bold;
            padding: 1px 2px;
            min-width: 26px;
        }

        /* Font Buttons (A- A+) */
        QPushButton#fontBtn {
            font-size: 11px;
            font-weight: bold;
            padding: 1px 1px;
            min-width: 26px;
        }

        QTreeWidget, QTableWidget {
            background-color: #fbf7ed;
            color: #3d2f1d;
            border: 1px solid #dcd0b6;
        }
        QTreeWidget::item:selected {
            background-color: #8c5b2a;
            color: #ffffff;
        }
        QHeaderView::section {
            background-color: #ede2c8;
            color: #4a3823;
            border-bottom: 1px solid #dcd0b6;
            font-weight: 600;
        }

        /* Docks */
        QDockWidget {
            color: #4a3823;
            font-weight: bold;
        }
        QDockWidget::title {
            background-color: #ece1c6;
            color: #4a3823;
            padding: 7px 12px;
            padding-right: 58px;
            border-bottom: 2px solid #8c5b2a;
            font-size: 12px;
            font-weight: bold;
        }
        QDockWidget::close-button {
            subcontrol-position: top right;
            subcontrol-origin: margin;
            position: absolute;
            top: 7px;
            right: 8px;
            width: 18px;
            height: 18px;
            border: none;
            border-radius: 3px;
            image: url(@DOCK_CLOSE@);
            background: transparent;
        }
        QDockWidget::close-button:hover {
            background-color: #fde8e8;
            image: url(@DOCK_CLOSE_HOVER@);
        }
        QDockWidget::float-button {
            subcontrol-position: top right;
            subcontrol-origin: margin;
            position: absolute;
            top: 7px;
            right: 32px;
            width: 18px;
            height: 18px;
            border: none;
            border-radius: 3px;
            image: url(@DOCK_FLOAT@);
            background: transparent;
        }
        QDockWidget::float-button:hover {
            background-color: #ede3cb;
        }

        QTabBar::tab:selected {
            background-color: #fbf7ed;
            color: #8c5b2a;
            border-bottom: 2px solid #8c5b2a;
        }

        QStatusBar {
            background-color: #ede2c8;
            color: #4a3823;
            border-top: 1px solid #dcd0b6;
        }
        """.replace("@ARROW_DOWN@", f"{ICONS_DIR}/arrow_down_sepia.png")
            .replace("@DOCK_CLOSE@", f"{ICONS_DIR}/dock_close_sepia.png")
            .replace("@DOCK_CLOSE_HOVER@", f"{ICONS_DIR}/dock_close_red.png")
            .replace("@DOCK_FLOAT@", f"{ICONS_DIR}/dock_float_sepia.png")
        )

    else:
        # === PROFESSIONAL LIGHT THEME ===
        return (
            """
        /* === PROFESSIONAL LIGHT THEME === */
        QMainWindow {
            background-color: #f0f4f8;
            color: #1a202c;
        }
        QWidget {
            color: #1a202c;
            font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, 'Roboto', sans-serif;
            font-size: 13px;
        }

        /* Menu Bar */
        QMenuBar {
            background-color: #ffffff;
            color: #1a202c;
            border-bottom: 1px solid #d2dbe5;
            padding: 3px 6px;
            font-size: 13px;
        }
        QMenuBar::item {
            background: transparent;
            padding: 5px 10px;
            border-radius: 4px;
            color: #2d3748;
            font-weight: 500;
        }
        QMenuBar::item:selected {
            background-color: #edf2f7;
            color: #1a202c;
        }
        QMenu {
            background-color: #ffffff;
            color: #1a202c;
            border: 1px solid #cbd5e0;
            border-radius: 6px;
            padding: 4px;
        }
        QMenu::item {
            padding: 6px 26px 6px 14px;
            border-radius: 4px;
            color: #2d3748;
        }
        QMenu::item:selected {
            background-color: #2b6cb0;
            color: #ffffff;
        }
        QMenu::separator {
            height: 1px;
            background-color: #e2e8f0;
            margin: 4px 6px;
        }

        /* Toolbar */
        QToolBar {
            background-color: #ffffff;
            border-bottom: 1px solid #d2dbe5;
            spacing: 8px;
            padding: 6px 12px;
        }
        QToolBar QLabel {
            color: #2d3748;
            font-weight: 600;
            font-size: 12px;
            margin-right: 2px;
        }
        QToolBar::separator {
            width: 1px;
            background-color: #cbd5e0;
            margin: 4px 6px;
        }

        /* ComboBox Dropdowns */
        QComboBox {
            background-color: #ffffff;
            color: #1a202c !important;
            border: 1px solid #b8c4d1;
            border-radius: 5px;
            padding: 2px 24px 2px 8px;
            min-height: 26px;
            font-size: 13px;
            font-weight: 500;
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
            width: 18px;
            border-left: 1px solid #cbd5e0;
            border-top-right-radius: 4px;
            border-bottom-right-radius: 4px;
            background: #f7fafc;
        }
        QComboBox::down-arrow {
            image: url(@ARROW_DOWN@);
            width: 10px;
            height: 10px;
        }
        QComboBox QAbstractItemView {
            background-color: #ffffff;
            color: #1a202c !important;
            border: 1px solid #cbd5e0;
            border-radius: 4px;
            selection-background-color: #2b6cb0;
            selection-color: #ffffff !important;
            padding: 4px;
            outline: none;
        }
        QComboBox QAbstractItemView::item {
            min-height: 24px;
            padding: 4px 8px;
            color: #1a202c;
        }
        QComboBox QAbstractItemView::item:selected {
            background-color: #2b6cb0;
            color: #ffffff;
        }

        /* Inputs */
        QLineEdit {
            background-color: #ffffff;
            color: #1a202c !important;
            border: 1px solid #b8c4d1;
            border-radius: 5px;
            padding: 4px 8px;
            min-height: 24px;
            selection-background-color: #2b6cb0;
            selection-color: #ffffff;
        }
        QLineEdit:focus {
            border-color: #2b6cb0;
        }

        /* Buttons */
        QPushButton {
            background-color: #ffffff;
            color: #2d3748;
            border: 1px solid #b8c4d1;
            border-radius: 5px;
            padding: 5px 14px;
            font-weight: 600;
            min-height: 22px;
        }
        QPushButton:hover {
            background-color: #edf2f7;
            border-color: #718096;
            color: #1a202c;
        }
        QPushButton:pressed {
            background-color: #e2e8f0;
        }

        /* Primary Action Buttons */
        QPushButton#primaryBtn {
            background-color: #2b6cb0;
            color: #ffffff;
            border: 1px solid #2b6cb0;
        }
        QPushButton#primaryBtn:hover {
            background-color: #3182ce;
        }

        /* Nav Buttons (◀ ▶) */
        QPushButton#navBtn {
            font-size: 13px;
            font-weight: bold;
            padding: 1px 2px;
            min-width: 26px;
        }

        /* Font Buttons (A- A+) */
        QPushButton#fontBtn {
            font-size: 11px;
            font-weight: bold;
            padding: 1px 1px;
            min-width: 26px;
        }

        /* Tree and Tables */
        QTreeWidget, QTableWidget {
            background-color: #ffffff;
            color: #1a202c;
            border: 1px solid #d2dbe5;
            border-radius: 4px;
            outline: none;
        }
        QTreeWidget::item {
            padding: 4px 6px;
            border-radius: 3px;
        }
        QTreeWidget::item:hover {
            background-color: #edf2f7;
        }
        QTreeWidget::item:selected {
            background-color: #2b6cb0;
            color: #ffffff;
        }
        QHeaderView::section {
            background-color: #f7fafc;
            color: #4a5568;
            padding: 7px;
            border: none;
            border-bottom: 1px solid #d2dbe5;
            font-weight: 600;
        }

        /* Docks */
        QDockWidget {
            color: #1a202c;
            font-weight: bold;
        }
        QDockWidget::title {
            background-color: #f7fafc;
            color: #2d3748;
            padding: 7px 12px;
            padding-right: 58px;
            border-bottom: 2px solid #2b6cb0;
            font-size: 12px;
            font-weight: bold;
        }
        QDockWidget::close-button {
            subcontrol-position: top right;
            subcontrol-origin: margin;
            position: absolute;
            top: 7px;
            right: 8px;
            width: 18px;
            height: 18px;
            border: none;
            border-radius: 3px;
            image: url(@DOCK_CLOSE@);
            background: transparent;
        }
        QDockWidget::close-button:hover {
            background-color: #fee2e2;
            image: url(@DOCK_CLOSE_HOVER@);
        }
        QDockWidget::float-button {
            subcontrol-position: top right;
            subcontrol-origin: margin;
            position: absolute;
            top: 7px;
            right: 32px;
            width: 18px;
            height: 18px;
            border: none;
            border-radius: 3px;
            image: url(@DOCK_FLOAT@);
            background: transparent;
        }
        QDockWidget::float-button:hover {
            background-color: #edf2f7;
        }

        /* Tab Widget */
        QTabWidget::pane {
            border: 1px solid #d2dbe5;
            background-color: #ffffff;
            border-radius: 4px;
        }
        QTabBar::tab {
            background-color: #f0f4f8;
            color: #718096;
            padding: 7px 16px;
            border: 1px solid #d2dbe5;
            border-bottom: none;
            border-top-left-radius: 5px;
            border-top-right-radius: 5px;
            margin-right: 2px;
            font-weight: 600;
        }
        QTabBar::tab:selected {
            background-color: #ffffff;
            color: #2b6cb0;
            border-bottom: 2px solid #2b6cb0;
        }
        QTabBar::tab:hover:!selected {
            background-color: #e2e8f0;
            color: #2d3748;
        }

        /* Status Bar */
        QStatusBar {
            background-color: #ffffff;
            color: #718096;
            border-top: 1px solid #d2dbe5;
            font-size: 11px;
            padding: 3px 8px;
        }

        /* Tooltip */
        QToolTip {
            background-color: #2d3748;
            color: #ffffff;
            border: 1px solid #4a5568;
            border-radius: 4px;
            padding: 6px 10px;
            font-size: 12px;
        }

        /* CheckBox */
        QCheckBox {
            color: #2d3748;
            spacing: 6px;
            font-weight: 500;
        }
        QCheckBox::indicator {
            width: 16px;
            height: 16px;
            border-radius: 3px;
            border: 1px solid #b8c4d1;
            background: #ffffff;
        }
        QCheckBox::indicator:checked {
            background-color: #2b6cb0;
            border-color: #2b6cb0;
        }
        """.replace("@ARROW_DOWN@", f"{ICONS_DIR}/arrow_down_dark.png")
            .replace("@DOCK_CLOSE@", f"{ICONS_DIR}/dock_close_dark.png")
            .replace("@DOCK_CLOSE_HOVER@", f"{ICONS_DIR}/dock_close_red.png")
            .replace("@DOCK_FLOAT@", f"{ICONS_DIR}/dock_float_dark.png")
        )
