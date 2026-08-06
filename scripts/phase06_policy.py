#!/usr/bin/env python3
"""Protect accepted PHASE 06 contracts while validating later additive phases."""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ACCEPTED_PHASE06 = "036ac89c07ddee1e26402c1c523529adbba48860"
BANNED_SUFFIXES = (".zip", ".tar", ".tgz", ".gz", ".7z", ".rar", ".b64", ".base64", ".chunk", ".chunks")
BANNED_PARTS = ("payload", "transport-workflow", "write-workflow", "temporary-workflow")


def run(*args: str) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def fail(message: str) -> None:
    raise SystemExit(f"PHASE06 POLICY FAILED: {message}")


def main() -> int:
    changed = [
        line
        for line in run("git", "diff", "--name-only", f"{ACCEPTED_PHASE06}...HEAD").splitlines()
        if line
    ]
    frozen = [
        next((ROOT / "database/migrations").glob(f"{version:04d}_*.sql"))
        .relative_to(ROOT)
        .as_posix()
        for version in range(1, 6)
    ]
    altered_frozen = [
        line
        for line in run(
            "git",
            "diff",
            "--name-only",
            f"{ACCEPTED_PHASE06}...HEAD",
            "--",
            *frozen,
        ).splitlines()
        if line
    ]
    if altered_frozen:
        fail(f"accepted migrations 0001-0005 changed: {altered_frozen}")

    bad = [
        path
        for path in changed
        if path.lower().endswith(BANNED_SUFFIXES)
        or any(part in path.lower() for part in BANNED_PARTS)
        or "/helpers." in path.lower()
        or "/helper." in path.lower()
    ]
    if bad:
        fail(f"helper/payload/archive/chunk artifact forbidden: {bad}")

    for path in changed:
        candidate = ROOT / path
        if candidate.is_file() and candidate.stat().st_size > 2_000_000:
            fail(f"oversized source artifact: {path}")
        if candidate.name.startswith(".env"):
            fail(f"environment secret file forbidden: {path}")

    for workflow in (ROOT / ".github/workflows").glob("*.yml"):
        text = workflow.read_text(encoding="utf-8")
        if "permissions:" in text and not re.search(
            r"permissions:\s*\n\s*contents:\s*read", text
        ):
            fail(f"workflow permissions are not contents: read: {workflow.name}")

    source = "\n".join(
        path.read_text(encoding="utf-8", errors="ignore")
        for root in (ROOT / "src", ROOT / "src-tauri/src")
        for path in root.rglob("*")
        if path.is_file()
    )
    for token in ("reqwest::", "hyper::Client", "ureq::", "XMLHttpRequest", "WebSocket(", "axios."):
        if token in source:
            fail(f"runtime network client forbidden: {token}")

    print(
        "PHASE06 POLICY PASS: accepted migrations 0001-0005 unchanged; "
        f"{len(changed)} later-phase paths inspected; contents read workflows"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
