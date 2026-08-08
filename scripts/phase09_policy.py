#!/usr/bin/env python3
"""Reject unsafe or out-of-scope PHASE 09 repository state."""
from __future__ import annotations

import hashlib
import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FROZEN = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
    "0006_accounting_payments_hardening.sql": "08763076ce7cbd77e585bf06b10bc856e7b8f02193484b1db974db95143cebd0",
}
SKIP = {".git", "node_modules", "target", "dist", ".pytest_cache", "__pycache__"}
ARCHIVE_SUFFIXES = {".tar", ".gz", ".tgz", ".zip", ".7z", ".rar"}
RUNTIME_SUFFIXES = {".sqlite", ".sqlite3", ".db", ".wal", ".shm", ".journal", ".pdf"}
SECRET_NAMES = {".env", ".env.local", ".env.production", ".env.development"}
TEXT_SUFFIXES = {".md", ".sql", ".py", ".rs", ".ts", ".tsx", ".json", ".yml", ".yaml", ".txt", ".css", ".html", ".toml"}
OWNED_EXACT = {
    ".github/workflows/integration-ci.yml",
    ".github/workflows/phase09-ci.yml",
    "AGENTS.md",
    "database/migrations/0007_phase09_documents_reports_audit_backup.sql",
    "database/schema.sql",
    "database/tests/invariants.sql",
    "docs/PHASE-09-REPORT.md",
    "docs/architecture/phase-09-documents-reports-audit-backup.md",
    "docs/execution/POSMAN-PHASE-09-EXECUTION-PACK.md",
    "package.json",
    "scripts/phase09_policy.py",
    "scripts/verify_phase06.py",
    "scripts/verify_phase07.py",
    "scripts/verify_phase08.py",
    "scripts/verify_phase09.py",
    "scripts/verify_schema.py",
    "src-tauri/Cargo.lock",
    "src-tauri/Cargo.toml",
    "src-tauri/src/commands/mod.rs",
    "src-tauri/src/commands/phase09.rs",
    "src-tauri/src/infrastructure/maintenance.rs",
    "src-tauri/src/infrastructure/mod.rs",
    "src-tauri/src/infrastructure/native_dialog.rs",
    "src-tauri/src/infrastructure/paths.rs",
    "src-tauri/src/lib.rs",
    "src-tauri/src/phase05/mod.rs",
    "src-tauri/src/phase05/state.rs",
    "src/app/AppRoot.tsx",
    "src/platform/tauri/phase09.ts",
    "tests/e2e/run_phase09.py",
    "tests/integration/phase09-gateway.test.ts",
    "tests/ui/phase09-ui-contract.test.ts",
}
OWNED_PREFIXES = ("src-tauri/src/phase09/", "src/features/phase09/")


def fail(message: str) -> None:
    raise SystemExit(f"PHASE09 POLICY FAILED: {message}")


def files() -> list[Path]:
    result = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        parts = path.relative_to(ROOT).parts
        if any(part in SKIP for part in parts):
            continue
        result.append(path)
    return result


def changed_paths(ownership_range: str) -> list[str]:
    return subprocess.run(
        ["git", "diff", "--name-only", ownership_range],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.splitlines()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--range", dest="ownership_range")
    arguments = parser.parse_args()

    if arguments.ownership_range:
        changed = changed_paths(arguments.ownership_range)
        unexpected = [
            path
            for path in changed
            if path not in OWNED_EXACT and not path.startswith(OWNED_PREFIXES)
        ]
        if unexpected:
            fail("out-of-scope changed paths:\n" + "\n".join(unexpected))
        print(f"PHASE09 ownership PASS: {len(changed)} changed paths in {arguments.ownership_range}")

    for name, expected in FROZEN.items():
        path = ROOT / "database" / "migrations" / name
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            fail(f"frozen migration changed: {name}: {actual}")

    all_files = files()
    for path in all_files:
        relative = path.relative_to(ROOT).as_posix()
        lower = relative.lower()
        if relative.startswith(".phase09-bootstrap/") or relative == ".phase09-bootstrap":
            fail(f"temporary recovery directory is tracked: {relative}")
        if path.name in SECRET_NAMES or path.name.startswith(".env."):
            fail(f"environment file is tracked: {relative}")
        if path.suffix.lower() in ARCHIVE_SUFFIXES:
            fail(f"source/archive artifact is tracked: {relative}")
        if path.suffix.lower() in RUNTIME_SUFFIXES:
            fail(f"private runtime/evidence artifact is tracked: {relative}")
        if any(token in lower for token in ("checkpoint-part", "checkpoint_piece", "source-archive", "private-export")):
            fail(f"checkpoint/private export artifact is tracked: {relative}")
        if path.stat().st_size > 2_000_000:
            fail(f"oversized tracked artifact: {relative}")
        if path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        private_key_marker = "-----BEGIN " + "PRIVATE KEY-----"
        token_pattern = r"\b(?:github" + r"_pat_|gh" + r"p_|sk" + r"-proj-)[A-Za-z0-9_-]+"
        if private_key_marker in text or re.search(token_pattern, text):
            fail(f"secret-like value found: {relative}")
        if relative.startswith(".github/workflows/"):
            normalized = re.sub(r"\s+", " ", text.lower())
            if re.search(r"contents\s*:\s*write", text, re.IGNORECASE):
                fail(f"workflow has contents: write: {relative}")
            if re.search(r"\bgit\s+(?:push|commit)\b", normalized) or "create-or-update-file-contents" in normalized:
                fail(f"workflow commits or pushes source: {relative}")
        if relative.startswith(("src/features/phase09/", "src/platform/tauri/phase09/", "src/platform/tauri/phase09.ts")):
            for forbidden in ("fetch(", "XMLHttpRequest", "WebSocket(", "http://", "https://"):
                if forbidden in text:
                    fail(f"frontend runtime network primitive in {relative}: {forbidden}")
            if re.search(r"(?<!<)\b(?:SELECT|INSERT|UPDATE|DELETE)\s+", text, re.IGNORECASE):
                fail(f"frontend SQL found: {relative}")
            if "@tauri-apps/plugin-fs" in text or "@tauri-apps/api/fs" in text:
                fail(f"unrestricted frontend filesystem access: {relative}")
        if relative.startswith("src-tauri/src/phase09/") and re.search(r"(?:todo!|unimplemented!)\s*\(", text):
            fail(f"unfinished Rust implementation: {relative}")
        if relative.startswith(("src-tauri/src/phase09/", "src/features/phase09/")) and re.search(r"<script|javascript:", text, re.IGNORECASE):
            if "forbidden" not in text.lower() and "reject" not in text.lower():
                fail(f"raw JavaScript template support: {relative}")

    status = subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=all"],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.splitlines()
    local_changes = len(status)
    print(
        "PHASE09 POLICY PASS: "
        f"{local_changes} local changed paths; migrations 0001-0006 frozen; "
        "no write-capable workflow, source archive, private artifact, secret, or runtime network authority"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
