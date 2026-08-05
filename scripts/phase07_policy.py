#!/usr/bin/env python3
"""Enforce PHASE 07 ownership, frozen migrations, and public-repository safety."""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "036ac89c07ddee1e26402c1c523529adbba48860"
ALLOWED = (
    ".github/workflows/", "docs/architecture/phase-07", "docs/PHASE-07-REPORT.md",
    "scripts/verify_phase06.py", "scripts/phase06_policy.py", "scripts/verify_phase07.py",
    "scripts/phase07_policy.py", "package.json",
    "src-tauri/src/commands/mod.rs", "src-tauri/src/commands/phase07.rs",
    "src-tauri/src/lib.rs", "src-tauri/src/phase07/", "src/app/AppRoot.tsx",
    "src/features/phase07/", "src/platform/tauri/phase07.ts", "tests/e2e/run_phase07.py",
    "tests/integration/phase07-gateway.test.ts", "tests/ui/phase07-ui-contract.test.ts",
)
FROZEN = tuple(f"database/migrations/{version:04d}_" for version in range(1, 6))
FORBIDDEN_SUFFIXES = (".zip", ".tar", ".tgz", ".gz", ".7z", ".rar", ".b64", ".base64", ".chunk")


def fail(message: str) -> None:
    raise SystemExit(f"PHASE07 POLICY FAILED: {message}")


def main() -> int:
    changed = subprocess.check_output(
        ["git", "diff", "--name-only", f"{BASELINE}...HEAD"], cwd=ROOT, text=True
    ).splitlines()
    outside = [path for path in changed if not any(path == item or path.startswith(item) for item in ALLOWED)]
    if outside:
        fail(f"ownership violation: {outside}")
    if any(path.startswith("database/") for path in changed):
        fail("PHASE 07 may not change the accepted database tree")
    bad = [path for path in changed if path.lower().endswith(FORBIDDEN_SUFFIXES) or "payload" in path.lower() or "transport" in path.lower()]
    if bad:
        fail(f"temporary transport/archive artifact forbidden: {bad}")
    migrations = sorted((ROOT / "database/migrations").glob("*.sql"))
    if len(migrations) != 5 or any(not path.name.startswith(f"{index:04d}_") for index, path in enumerate(migrations, 1)):
        fail("accepted migrations must remain exactly 0001-0005")
    for workflow in (ROOT / ".github/workflows").glob("*.yml"):
        text = workflow.read_text(encoding="utf-8")
        if "permissions:" in text and not re.search(r"permissions:\s*\n\s*contents:\s*read", text):
            fail(f"workflow permissions are not contents: read: {workflow.name}")
    source = "\n".join(
        path.read_text(encoding="utf-8", errors="ignore")
        for root in (ROOT / "src", ROOT / "src-tauri/src")
        for path in root.rglob("*") if path.is_file()
    )
    for token in ("reqwest::", "hyper::Client", "ureq::", "XMLHttpRequest", "WebSocket(", "axios."):
        if token in source:
            fail(f"runtime network client forbidden: {token}")
    print(f"PHASE07 POLICY PASS: {len(changed)} owned paths; frozen migrations 0001-0005; contents read workflows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
