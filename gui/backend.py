"""MyBible Backend Client
Communicates with the persistent Rust 'mybible server' process via QProcess
using line-delimited JSON RPC.
"""

import json
import os
import shutil
import sys
from pathlib import Path
from typing import Callable, Optional

from PySide6.QtCore import QByteArray, QObject, QProcess, Signal


def find_mybible_binary() -> str:
    """Locates the compiled mybible executable."""
    base_dir = Path(__file__).resolve().parent.parent

    # 1. Project root
    root_exe = base_dir / ("mybible.exe" if sys.platform == "win32" else "mybible")
    if root_exe.exists():
        return str(root_exe)

    # 2. target/release
    release_exe = (
        base_dir
        / "target"
        / "release"
        / ("mybible.exe" if sys.platform == "win32" else "mybible")
    )
    if release_exe.exists():
        return str(release_exe)

    # 3. target/debug
    debug_exe = (
        base_dir
        / "target"
        / "debug"
        / ("mybible.exe" if sys.platform == "win32" else "mybible")
    )
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
    """Client communicating with the long-running Rust server process."""

    serverReady = Signal()
    serverError = Signal(str)

    def __init__(self, parent: Optional[QObject] = None):
        super().__init__(parent)
        self.binary_path = find_mybible_binary()
        self.process: Optional[QProcess] = None
        self._request_counter = 0
        self._buffer = QByteArray()
        self.start_server()

    def start_server(self):
        """Starts the persistent background server process."""
        if self.process and self.process.state() != QProcess.NotRunning:
            return

        self.process = QProcess(self)
        self.process.setProgram(self.binary_path)
        self.process.setArguments(["server"])
        self.process.setProcessChannelMode(QProcess.SeparateChannels)
        self.process.start()

        if not self.process.waitForStarted(5000):
            err = self.process.errorString()
            print(f"Failed to start mybible server: {err}", file=sys.stderr)
            self.serverError.emit(err)

    def is_alive(self) -> bool:
        return self.process is not None and self.process.state() == QProcess.Running

    def send_request(self, method: str, params: Optional[dict] = None, timeout_ms: int = 15000) -> Optional[dict]:
        """Sends a JSON RPC request to the interactive server and waits for the matching response."""
        if not self.is_alive():
            self.start_server()
            if not self.is_alive():
                return None

        self._request_counter += 1
        req_id = self._request_counter
        payload = {
            "id": req_id,
            "method": method,
            "params": params or {},
        }

        req_bytes = (json.dumps(payload) + "\n").encode("utf-8")
        self.process.write(req_bytes)
        self.process.waitForBytesWritten(3000)

        # Read line from server stdout
        start_time = 0
        while self.is_alive():
            if self.process.canReadLine():
                raw_line = bytes(self.process.readLine()).decode("utf-8", errors="replace").strip()
                if not raw_line:
                    continue
                try:
                    resp = json.loads(raw_line)
                    if resp.get("id") == req_id:
                        if resp.get("success"):
                            return resp.get("data")
                        else:
                            print(f"Server error for {method}: {resp.get('error')}", file=sys.stderr)
                            return None
                except json.JSONDecodeError as e:
                    print(f"JSON decode error: {e} | line: {raw_line[:120]}", file=sys.stderr)
            else:
                if not self.process.waitForReadyRead(timeout_ms):
                    print(f"Timeout waiting for server response to {method} (id={req_id})", file=sys.stderr)
                    return None

        return None

    def close(self):
        """Cleanly terminates the server process."""
        if self.process and self.process.state() != QProcess.NotRunning:
            self.process.terminate()
            if not self.process.waitForFinished(1000):
                self.process.kill()
                self.process.waitForFinished(500)

    # --- High-level API Methods ---

    def get_modules(self) -> dict:
        """Retrieves list of all locally installed modules grouped by category."""
        data = self.send_request("status")
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
        params = {}
        if bible:
            params["bible"] = bible
        data = self.send_request("books", params)
        if data and "books" in data:
            return data["books"]
        return []

    def get_passage(
        self,
        reference: str,
        bible: Optional[str] = None,
        subheadings: bool = True,
        footnotes: bool = True,
        xrefs: bool = True,
        subheadings_module: Optional[str] = None,
    ) -> Optional[dict]:
        """Reads a Bible passage."""
        params = {
            "ref": reference,
            "subheadings": subheadings,
            "footnotes": footnotes,
            "xrefs": xrefs,
        }
        if bible:
            params["bible"] = bible
        if subheadings_module:
            params["subheadings_module"] = subheadings_module

        return self.send_request("read", params)

    def get_compare(self, reference: str, bibles: list[str]) -> Optional[dict]:
        """Retrieves parallel verses aligned for multiple Bible translations."""
        params = {
            "ref": reference,
            "bibles": bibles,
        }
        return self.send_request("compare", params)

    def get_verse_preview(self, reference: str, bible: Optional[str] = None) -> Optional[str]:
        """Retrieves plain-text preview of a verse (for tooltips and references)."""
        params = {"ref": reference}
        if bible:
            params["bible"] = bible
        data = self.send_request("preview", params)
        if data:
            return data.get("text")
        return None

    def get_commentary(
        self, reference: str, module: Optional[str] = None
    ) -> Optional[dict]:
        """Retrieves commentary notes for a passage."""
        params = {"ref": reference}
        if module:
            params["module"] = module
        return self.send_request("commentary", params)

    def get_dict_entry(
        self, term: str, module: Optional[str] = None
    ) -> Optional[dict]:
        """Looks up a word or Strong's Concordance number."""
        params = {"term": term}
        if module:
            params["module"] = module
        return self.send_request("dict", params)

    def get_cross_references(self, reference: str) -> Optional[dict]:
        """Retrieves cross-references for a verse."""
        return self.send_request("xref", {"ref": reference})

    def search_bible(
        self,
        query: str,
        bible: Optional[str] = None,
        bibles: Optional[list[str]] = None,
        all_bibles: bool = False,
        testament: Optional[str] = None,
        mode: str = "all",
        limit: int = 100,
        case_sensitive: bool = False,
        match_accents: bool = False,
    ) -> Optional[dict]:
        """Executes full-text search across installed Bible modules."""
        params = {
            "query": query,
            "mode": mode,
            "limit": limit,
            "case_sensitive": case_sensitive,
            "match_accents": match_accents,
            "all_bibles": all_bibles,
        }
        if bibles:
            params["bibles"] = bibles
        elif bible:
            params["bible"] = bible
        if testament:
            params["testament"] = testament

        return self.send_request("search", params)

    def get_last_search(self) -> Optional[dict]:
        """Retrieves persistent results from the last executed search."""
        return self.send_request("last_search")

    def get_catalog(
        self,
        query: str = "",
        language: Optional[str] = None,
        mod_type: Optional[str] = None,
    ) -> Optional[dict]:
        """Retrieves the official repository catalog with language breakdown."""
        params = {
            "query": query,
            "language": language or "all",
            "mod_type": mod_type or "all",
        }
        return self.send_request("catalog", params)

    def download_module(self, abbreviation: str) -> Optional[dict]:
        """Downloads and loads a module from the repository catalog."""
        return self.send_request("download", {"abbr": abbreviation}, timeout_ms=60000)

    def reload_modules(self) -> Optional[dict]:
        """Reloads all installed modules."""
        return self.send_request("reload")
