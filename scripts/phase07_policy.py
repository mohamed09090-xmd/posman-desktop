#!/usr/bin/env python3
"""Protect accepted PHASE 07 contracts while validating later additive phases."""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ACCEPTED_PHASE07 = "ae133cea9c3b6760a5fd22b38d3169aa2f976dc6"
FROZEN_MIGRATIONS = tuple(
    next((ROOT / "database/migrations").glob(f"{version:04d}_*.sql"))
    for version in range(1, 6)
)
FORBIDDEN_SUFFIXES = (".zip", ".tar", ".tgz", ".gz", ".7z", ".rar", ".b64", ".base64", ".chunk")


def fail(message: str) -> None:
    raise SystemExit(f"PHASE07 POLICY FAILED: {message}")


def main() -> int:
    changed = subprocess.check_output(
        ["git", "diff", "--name-only", f"{ACCEPTED_PHASE07}...HEAD"],
        cwd=ROOT,
        text=True,
    ).splitlines()

    frozen_paths = [path.relative_to(ROOT).as_posix() for path in FROZEN_MIGRATIONS]
    altered_frozen = subprocess.check_output(
        ["git", "diff", "--name-only", f"{ACCEPTED_PHASE07}...HEAD", "--", *frozen_paths],
        cwd=ROOT,
        text=True,
    ).splitlines()
    if altered_frozen:
        fail(f"accepted migrations 0001-0005 changed: {altered_frozen}")

    migrations = sorted((ROOT / "database/migrations").glob("*.sql"))
    if len(migrations) < 5 or any(
        not migrations[index - 1].name.startswith(f"{index:04d}_")
        for index in range(1, 6)
    ):
        fail("accepted migrations 0001-0005 must remain present and ordered")

    bad = [
        path
        for path in changed
        if path.lower().endswith(FORBIDDEN_SUFFIXES)
        or "payload" in path.lower()
        or "transport" in path.lower()
    ]
    if bad:
        fail(f"temporary transport/archive artifact forbidden: {bad}")

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
        "PHASE07 POLICY PASS: accepted migrations 0001-0005 unchanged; "
        f"{len(changed)} later-phase paths inspected; contents read workflows"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
