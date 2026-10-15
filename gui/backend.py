"""
MyBible Backend Client
Communicates with the compiled Rust 'mybible' binary via QProcess.
"""

import json
import os
import shutil
import sys
from pathlib import Path
from typing import Callable, Optional

from PySide6.QtCore import QObject, QProcess, Signal


def find_mybible_binary() -> str:
    """Locates the compiled mybible executable."""
    base_dir = Path(__file__).resolve().parent.parent

    # 1. Project root
    root_exe = base_dir / ("mybible.exe" if sys.platform == "win32" else "mybible")
    if root_exe.exists():
        return str(root_exe)

    # 2. target/release
    release_exe = base_dir / "target" / "release" / ("mybible.exe" if sys.platform == "win32" else "mybible")
    if release_exe.exists():
        return str(release_exe)

    # 3. target/debug
    debug_exe = base_dir / "target" / "debug" / ("mybible.exe" if sys.platform == "win32" else "mybible")
    if debug_exe.exists():
        return str(debug_exe)

    # 4. PATH
    which_path = shutil.which("mybible")
    if which_path:
        return which_path

    raise FileNotFoundError(
        "Could not find 'mybible' executable. Run 'cargo build --release' to compile it."
    )


class RustBackendClient(QObject):
    """Client wrapping the Rust backend CLI with QProcess."""

    def __init__(self, parent: Optional[QObject] = None):
        super().__init__(parent)
        self.binary_path = find_mybible_binary()

    def run_sync(self, args: list[str], timeout_ms: int = 15000) -> tuple[int, str, str]:
        """Runs a command synchronously using QProcess and returns (exit_code, stdout, stderr)."""
        process = QProcess()
        process.setProgram(self.binary_path)
        process.setArguments(args)
        process.start()

        if not process.waitForFinished(timeout_ms):
            process.kill()
            return -1, "", "Process timed out"

        exit_code = process.exitCode()
        stdout = bytes(process.readAllStandardOutput()).decode("utf-8", errors="replace")
        stderr = bytes(process.readAllStandardError()).decode("utf-8", errors="replace")
        return exit_code, stdout, stderr

    def run_json(self, args: list[str]) -> Optional[dict]:
        """Runs command with --json and parses JSON output."""
        full_args = ["--json"] + args
        exit_code, stdout, stderr = self.run_sync(full_args)
        if exit_code != 0:
            print(f"Backend error: {stderr}", file=sys.stderr)
            return None
        try:
            return json.loads(stdout.strip())
        except json.JSONDecodeError as e:
            print(f"Failed to parse JSON: {e}\nRaw output: {stdout[:300]}", file=sys.stderr)
            return None

    # --- High-level API Methods ---

    def get_modules(self) -> dict:
        """Retrieves list of all locally installed modules grouped by category."""
        data = self.run_json(["modules", "list"])
        return data or {
            "bibles": [],
            "commentaries": [],
            "dictionaries": [],
            "cross_references": [],
            "subheadings": [],
            "reading_plans": [],
            "devotions": [],
            "total_modules": 0,
        }

    def get_books(self, bible: Optional[str] = None) -> list[dict]:
        """Retrieves books list for a translation with chapter counts."""
        args = ["books"]
        if bible:
            args.extend(["-b", bible])
        data = self.run_json(args)
        return data if isinstance(data, list) else []

    def read_passage(
        self,
        reference: str,
        bible: Optional[str] = None,
        subheadings: Optional[str] = None,
        no_subheadings: bool = False,
        footnotes: bool = True,
        xrefs: bool = True,
    ) -> Optional[dict]:
        """Reads a Bible passage or chapter."""
        args = ["read", reference]
        if bible:
            args.extend(["-b", bible])
        if subheadings:
            args.extend(["-s", subheadings])
        if no_subheadings:
            args.append("--no-subheadings")
        if footnotes:
            args.append("--footnotes")
        if xrefs:
            args.append("--xrefs")
        return self.run_json(args)

    def search_bible(
        self,
        query: str,
        bible: Optional[str] = None,
        all_bibles: bool = False,
        book: Optional[str] = None,
        testament: Optional[str] = None,
        mode: str = "all",
        limit: int = 50,
    ) -> Optional[dict]:
        """Executes search across Bible module(s)."""
        args = ["search", query, "-m", mode, "-l", str(limit)]
        if all_bibles:
            args.append("--all")
        elif bible:
            args.extend(["-b", bible])
        if book:
            args.extend(["-B", book])
        if testament:
            args.extend(["-t", testament])
        return self.run_json(args)

    def get_commentary(self, reference: str, module: Optional[str] = None) -> Optional[dict]:
        """Retrieves commentary notes for a reference."""
        args = ["commentary", reference]
        if module:
            args.extend(["-m", module])
        return self.run_json(args)

    def get_dict_entry(self, term: str, module: Optional[str] = None) -> Optional[dict]:
        """Retrieves dictionary entry for a Strong's code or word topic."""
        args = ["dict", term]
        if module:
            args.extend(["-m", module])
        return self.run_json(args)

    def get_cross_references(self, reference: str) -> Optional[dict]:
        """Retrieves cross-references for a verse."""
        return self.run_json(["xref", reference])

    def search_repository(self, query: str, language: Optional[str] = None) -> list[dict]:
        """Searches the online module catalog."""
        args = ["modules", "search", query]
        if language:
            args.extend(["-l", language])
        data = self.run_json(args)
        return data if isinstance(data, list) else []

    def start_download(
        self,
        abbrev: str,
        on_output: Callable[[str], None],
        on_finished: Callable[[int], None],
    ) -> QProcess:
        """Starts asynchronous module download process with live output streaming."""
        process = QProcess(self)
        process.setProgram(self.binary_path)
        process.setArguments(["modules", "download", abbrev])

        def handle_stdout():
            text = bytes(process.readAllStandardOutput()).decode("utf-8", errors="replace")
            on_output(text)

        def handle_stderr():
            text = bytes(process.readAllStandardError()).decode("utf-8", errors="replace")
            on_output(text)

        process.readyReadStandardOutput.connect(handle_stdout)
        process.readyReadStandardError.connect(handle_stderr)
        process.finished.connect(on_finished)
        process.start()
        return process
