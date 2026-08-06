#!/usr/bin/env python3
"""PHASE 08 ownership, offline, workflow-permission, and artifact-transport policy."""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "ae133cea9c3b6760a5fd22b38d3169aa2f976dc6"
ALLOWED = (
    ".github/workflows/", "database/", "docs/", "scripts/", "src-tauri/", "src/", "tests/",
    "AGENTS.md", "README.md", "package.json", "package-lock.json",
)
BANNED_SUFFIXES = (".zip", ".tar", ".tgz", ".gz", ".7z", ".rar", ".b64", ".base64", ".chunk", ".chunks")
BANNED_PARTS = ("payload", "transport-workflow", "write-workflow", "temporary-workflow")
NETWORK_TOKENS = ("reqwest::", "hyper::Client", "ureq::", "XMLHttpRequest", "WebSocket(", "axios.")


def run(*args: str) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def fail(message: str) -> None:
    raise SystemExit(f"PHASE08 POLICY FAILED: {message}")


def main() -> int:
    changed = [line for line in run("git", "diff", "--name-only", f"{BASELINE}...HEAD").splitlines() if line]
    outside = [path for path in changed if not any(path == prefix or path.startswith(prefix) for prefix in ALLOWED)]
    if outside:
        fail(f"ownership violation: {outside}")
    bad = [
        path for path in changed
        if path.lower().endswith(BANNED_SUFFIXES)
        or any(part in path.lower() for part in BANNED_PARTS)
        or "/helpers." in path.lower()
        or "/helper." in path.lower()
    ]
    if bad:
        fail(f"helper/payload/archive/chunk transport forbidden: {bad}")
    for path in changed:
        candidate = ROOT / path
        if candidate.is_file() and candidate.stat().st_size > 2_000_000:
            fail(f"oversized source artifact: {path}")
        if candidate.name.startswith(".env"):
            fail(f"environment secret file forbidden: {path}")
    for workflow in (ROOT / ".github/workflows").glob("*.yml"):
        text = workflow.read_text(encoding="utf-8")
        if "permissions:" in text and not re.search(r"permissions:\s*\n\s*contents:\s*read", text):
            fail(f"workflow permissions are not contents: read: {workflow.name}")
    source = "\n".join(
        path.read_text(encoding="utf-8", errors="ignore")
        for root in (ROOT / "src", ROOT / "src-tauri/src")
        for path in root.rglob("*") if path.is_file()
    )
    for token in NETWORK_TOKENS:
        if token in source:
            fail(f"runtime network client forbidden: {token}")
    for version in range(1, 6):
        paths = list((ROOT / "database/migrations").glob(f"{version:04d}_*.sql"))
        if len(paths) != 1:
            fail(f"accepted migration {version:04d} missing or duplicated")
    migration6 = list((ROOT / "database/migrations").glob("0006_*.sql"))
    if len(migration6) != 1:
        fail("exactly one authorized migration 0006 is required")
    print(f"PHASE08 POLICY PASS: {len(changed)} owned paths; frozen migrations 0001-0005; offline runtime; contents read workflows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
