"""Main entry point for MyBible Desktop PySide6 Application."""

import sys
from pathlib import Path

# Ensure parent directory is on sys.path so `gui.*` package imports resolve cleanly
ROOT_DIR = Path(__file__).resolve().parent.parent
if str(ROOT_DIR) not in sys.path:
    sys.path.insert(0, str(ROOT_DIR))

from PySide6.QtCore import QTimer  # noqa: E402
from PySide6.QtWidgets import QApplication  # noqa: E402

from gui.main_window import MainWindow  # noqa: E402


def main():
    # Create QApplication
    app = QApplication(sys.argv)
    app.setApplicationName("MyBible Desktop")
    app.setOrganizationName("MyBible")

    # Enable periodic timer to allow Python's signal handler (e.g. Ctrl+C) to run
    timer = QTimer()
    timer.timeout.connect(lambda: None)
    timer.start(500)

    # Launch main window
    window = MainWindow()
    window.show()

    # Run Qt event loop
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
