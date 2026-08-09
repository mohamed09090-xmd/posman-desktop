#!/usr/bin/env python3
"""Reject unsafe, secret-bearing, or out-of-scope PHASE 10 repository state."""
from __future__ import annotations

import argparse
import hashlib
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
    "0007_phase09_documents_reports_audit_backup.sql": "f22f220ccf6ae2f85f0be85ae018b9f5760e725e9e88c6b2b87c37598424eb90",
}
OWNED_EXACT = {
    ".github/workflows/integration-ci.yml",
    ".github/workflows/phase09-ci.yml",
    ".github/workflows/phase10-ci.yml",
    ".github/workflows/ui-ci.yml",
    "CHANGELOG.md",
    "README.md",
    "SECURITY.md",
    "assets/branding/README.md",
    "assets/branding/posman-app-icon.png",
    "docs/PHASE-10-REPORT.md",
    "docs/architecture/phase-10-distribution-hardening.md",
    "docs/execution/POSMAN-PHASE-10-EXECUTION-PACK.md",
    "docs/release/CODE-SIGNING.md",
    "docs/release/POSMAN-1.0.0-RELEASE-NOTES.md",
    "docs/release/WINDOWS-INSTALLATION.md",
    "package-lock.json",
    "package.json",
    "scripts/generate_brand_assets.py",
    "scripts/phase08_policy.py",
    "scripts/phase10_performance.py",
    "scripts/phase10_policy.py",
    "scripts/release/collect_release.ps1",
    "scripts/release/test_installer_lifecycle.ps1",
    "scripts/verify_phase10.py",
    "src-tauri/Cargo.lock",
    "src-tauri/Cargo.toml",
    "src-tauri/icons/128x128.png",
    "src-tauri/icons/128x128@2x.png",
    "src-tauri/icons/32x32.png",
    "src-tauri/icons/icon.ico",
    "src-tauri/installer.nsi",
    "src-tauri/tauri.conf.json",
    "src-tauri/tauri.phase10-upgrade-fixture.conf.json",
}
SKIP = {".git", "node_modules", "target", "dist", "__pycache__", ".pytest_cache"}
SECRET_NAMES = {".env", ".env.local", ".env.production", ".env.development"}
SECRET_SUFFIXES = {".pfx", ".p12", ".pem", ".key", ".keystore", ".jks"}
RUNTIME_SUFFIXES = {".sqlite", ".sqlite3", ".db", ".wal", ".shm", ".journal"}
ARCHIVE_SUFFIXES = {".zip", ".7z", ".rar", ".tgz", ".gz", ".tar"}
TEXT_SUFFIXES = {".md", ".py", ".ps1", ".rs", ".ts", ".tsx", ".json", ".yml", ".yaml", ".txt", ".css", ".html", ".toml"}


def fail(message: str) -> None:
    raise SystemExit(f"PHASE10 POLICY FAILED: {message}")


def changed_paths(revision_range: str) -> list[str]:
    return subprocess.run(
        ["git", "diff", "--name-only", revision_range],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()


def tracked_files() -> list[Path]:
    return [
        path
        for path in ROOT.rglob("*")
        if path.is_file() and not any(part in SKIP for part in path.relative_to(ROOT).parts)
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--range", dest="revision_range")
    args = parser.parse_args()
    if args.revision_range:
        changed = changed_paths(args.revision_range)
        unexpected = sorted(path for path in changed if path not in OWNED_EXACT)
        if unexpected:
            fail("out-of-scope changed paths:\n" + "\n".join(unexpected))
        print(f"PHASE10 ownership PASS: {len(changed)} paths in {args.revision_range}")

    for name, expected in FROZEN.items():
        path = ROOT / "database/migrations" / name
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            fail(f"accepted migration changed: {name}: {actual}")
    if len(list((ROOT / "database/migrations").glob("*.sql"))) != 7:
        fail("PHASE 10 must not add or remove a database migration")

    for path in tracked_files():
        relative = path.relative_to(ROOT).as_posix()
        suffix = path.suffix.lower()
        if path.name in SECRET_NAMES or path.name.startswith(".env."):
            fail(f"environment file is tracked: {relative}")
        if suffix in SECRET_SUFFIXES:
            fail(f"private signing/credential material is tracked: {relative}")
        if suffix in RUNTIME_SUFFIXES:
            fail(f"runtime database artifact is tracked: {relative}")
        if suffix in ARCHIVE_SUFFIXES:
            fail(f"archive artifact is tracked: {relative}")
        if path.stat().st_size > 3_000_000:
            fail(f"oversized tracked artifact: {relative}")
        if suffix not in TEXT_SUFFIXES:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        private_key_marker = "-----BEGIN " + "PRIVATE KEY-----"
        token_pattern = (
            r"\b(?:github" + r"_pat_|gh" + r"p_|sk" + r"-proj-)[A-Za-z0-9_-]+"
        )
        if private_key_marker in text or re.search(token_pattern, text):
            fail(f"secret-like value found: {relative}")
        if relative.startswith(".github/workflows/"):
            normalized = re.sub(r"\s+", " ", text.lower())
            if re.search(r"^\s*contents\s*:\s*write\s*(?:#.*)?$", text, re.MULTILINE | re.IGNORECASE):
                fail(f"workflow has write permission: {relative}")
            if re.search(r"\bgit\s+(?:push|commit)\b", normalized):
                fail(f"workflow writes repository history: {relative}")

    config = (ROOT / "src-tauri/tauri.conf.json").read_text(encoding="utf-8")
    for forbidden in ("downloadBootstrapper", "embedBootstrapper", "createUpdaterArtifacts", "certificateThumbprint", "signCommand"):
        if forbidden in config:
            fail(f"unsafe or online production bundle option found: {forbidden}")
    print(
        "PHASE10 POLICY PASS: migrations frozen; owned release surface only; "
        "no signing secret, runtime database, archive, write workflow, updater, or online bootstrapper"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
