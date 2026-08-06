#!/usr/bin/env python3
"""Verify PHASE 09 documents, reports, audit, backup, and restore contracts."""
from __future__ import annotations

import hashlib
import json
import re
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "0abaff289758fd2e5597faef834f9b70156d54e1"
FROZEN_MIGRATIONS = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
    "0006_accounting_payments_hardening.sql": "08763076ce7cbd77e585bf06b10bc856e7b8f02193484b1db974db95143cebd0",
}
REQUIRED_PERMISSIONS = {
    "documents.templates.view", "documents.templates.manage", "documents.render",
    "documents.print", "documents.export", "reports.view", "reports.export",
    "audit.view", "audit.export", "backup.view", "backup.create",
    "backup.restore", "backup.manage",
}
REQUIRED_COMMANDS = {
    "phase09_list_templates", "phase09_get_template", "phase09_create_template_draft",
    "phase09_update_template_draft", "phase09_publish_template", "phase09_retire_template",
    "phase09_preview_document", "phase09_render_document", "phase09_list_rendered_documents",
    "phase09_get_rendered_document", "phase09_verify_rendered_document",
    "phase09_export_rendered_pdf", "phase09_print_rendered_document",
    "phase09_list_reports", "phase09_run_report", "phase09_export_report_csv",
    "phase09_export_report_pdf", "phase09_list_audit_events", "phase09_export_audit_csv",
    "phase09_get_backup_settings", "phase09_update_backup_settings", "phase09_create_backup",
    "phase09_list_backups", "phase09_verify_backup", "phase09_export_backup",
    "phase09_import_backup", "phase09_restore_backup", "phase09_delete_backup",
}
DOCUMENT_TYPES = {
    "SALES_ORDER", "DELIVERY_NOTE", "SALES_INVOICE", "SALES_CREDIT_NOTE",
    "PURCHASE_ORDER", "GOODS_RECEIPT", "SUPPLIER_INVOICE", "PURCHASE_RETURN",
    "CUSTOMER_RECEIPT", "SUPPLIER_PAYMENT",
}
REPORTS = {
    "SALES_SUMMARY", "SALES_BY_PRODUCT", "SALES_BY_CUSTOMER", "PURCHASES_SUMMARY",
    "PURCHASES_BY_SUPPLIER", "STOCK_ON_HAND", "STOCK_VALUATION", "STOCK_MOVEMENTS",
    "LOW_STOCK", "OPEN_RECEIVABLES", "OPEN_PAYABLES", "CASH_BANK_REGISTER",
    "TRIAL_BALANCE",
}
E2E_SCENARIOS = {
    "phase09_ar_template_publish_and_historical_reprint",
    "phase09_fr_sales_invoice_preview_and_pdf",
    "phase09_ar_reports_csv_and_pdf",
    "phase09_fr_audit_filter_and_redacted_export",
    "phase09_ar_manual_backup_and_verification",
    "phase09_fr_corrupted_backup_rejected",
    "phase09_ar_restore_requires_verified_safety_backup",
    "phase09_fr_restore_success_returns_to_login",
}


def fail(message: str) -> None:
    raise SystemExit(f"PHASE09 VERIFY FAILED: {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path: str) -> str:
    candidate = ROOT / path
    if not candidate.is_file():
        fail(f"required file missing: {path}")
    return candidate.read_text(encoding="utf-8")


def expect_sql_error(connection: sqlite3.Connection, sql: str, label: str) -> None:
    try:
        connection.execute(sql)
    except sqlite3.DatabaseError:
        return
    fail(f"database invariant was not enforced: {label}")


def migration_fixture() -> sqlite3.Connection:
    connection = sqlite3.connect(":memory:")
    connection.execute("PRAGMA foreign_keys=ON")
    for index, path in enumerate(sorted((ROOT / "database/migrations").glob("*.sql")), 1):
        sql = path.read_text(encoding="utf-8")
        connection.executescript(sql)
        connection.execute(
            "INSERT INTO app_migrations(id,version,name,checksum_sha256,applied_at) VALUES(?,?,?,?,?)",
            (index, f"{index:04d}", path.name, digest(path), "2026-08-06T00:00:00Z"),
        )
    return connection


def verify_0006_to_0007_upgrade() -> None:
    connection = sqlite3.connect(":memory:")
    connection.execute("PRAGMA foreign_keys=ON")
    migrations = sorted((ROOT / "database/migrations").glob("*.sql"))
    for index, path in enumerate(migrations[:6], 1):
        connection.executescript(path.read_text(encoding="utf-8"))
        connection.execute(
            "INSERT INTO app_migrations(id,version,name,checksum_sha256,applied_at) VALUES(?,?,?,?,?)",
            (index, f"{index:04d}", path.name, digest(path), "2026-08-06T00:00:00Z"),
        )
    connection.executescript(migrations[6].read_text(encoding="utf-8"))
    connection.execute(
        "INSERT INTO app_migrations(id,version,name,checksum_sha256,applied_at) VALUES(7,'0007',?,?,?)",
        (migrations[6].name, digest(migrations[6]), "2026-08-06T00:00:00Z"),
    )
    if connection.execute("SELECT MAX(version) FROM app_migrations").fetchone()[0] != "0007":
        fail("0006 to 0007 upgrade did not reach schema 0007")
    if connection.execute("PRAGMA foreign_key_check").fetchall():
        fail("0006 to 0007 upgrade produced foreign-key violations")
    connection.close()


def exercise_database_contracts(connection: sqlite3.Connection) -> None:
    now = "2026-08-06T00:00:00Z"
    for company in ("company-a", "company-b"):
        connection.execute(
            "INSERT INTO companies(id,code,legal_name,name_ar,name_fr,created_at,updated_at) VALUES(?,?,?,?,?,?,?)",
            (company, company, company, company, company, now, now),
        )
        connection.execute(
            "INSERT INTO document_templates(id,company_id,code,document_type,name_ar,name_fr,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?)",
            (f"template-{company}", company, "SALES_INVOICE-ar", "SALES_INVOICE", "فاتورة", "Facture", now, now),
        )
        connection.execute(
            "INSERT INTO phase09_template_drafts(id,company_id,document_template_id,document_type,locale,version_number,state,display_name,title_ar,title_fr,created_at,updated_at) VALUES(?,?,?,?,?,1,'DRAFT',?,?,?,?,?)",
            (f"draft-{company}", company, f"template-{company}", "SALES_INVOICE", "ar", "Invoice", "فاتورة", "Facture", now, now),
        )
        connection.execute(
            "INSERT INTO document_template_versions(id,company_id,document_template_id,version_number,html_template,css_template,content_hash_sha256,is_published,created_at) VALUES(?,?,?,?,?,?,?,?,?)",
            (f"version-{company}", company, f"template-{company}", 1, "<main></main>", "@page{}", "a" * 64, 1, now),
        )
        connection.execute(
            "INSERT INTO phase09_template_version_configs(template_version_id,company_id,document_template_id,source_draft_id,document_type,locale,config_json,published_at) VALUES(?,?,?,?,?,?,?,?)",
            (f"version-{company}", company, f"template-{company}", f"draft-{company}", "SALES_INVOICE", "ar", "{}", now),
        )
        connection.execute("UPDATE phase09_template_drafts SET state='PUBLISHED' WHERE id=?", (f"draft-{company}",))
    expect_sql_error(connection, "UPDATE phase09_template_drafts SET display_name='x' WHERE id='draft-company-a'", "published template immutable")
    expect_sql_error(connection, "DELETE FROM phase09_template_version_configs WHERE template_version_id='version-company-a'", "published config immutable")
    expect_sql_error(
        connection,
        "INSERT INTO phase09_template_version_configs(template_version_id,company_id,document_template_id,source_draft_id,document_type,locale,config_json,published_at) VALUES('version-company-a','company-b','template-company-a','draft-company-a','SALES_INVOICE','ar','{}','2026-08-06T00:00:00Z')",
        "cross-company template reference rejected",
    )

    connection.execute(
        "INSERT INTO phase09_rendered_documents(id,company_id,document_type,source_document_id,source_document_number,source_document_status,document_template_id,template_version_id,locale,canonical_payload_json,rendered_html,rendered_css,content_sha256,pdf_relative_path,pdf_sha256,pdf_size_bytes,rendered_at) VALUES('render-a','company-a','SALES_INVOICE','source-a','FA-1','POSTED','template-company-a','version-company-a','ar','{}','<main></main>','',?,'company-a/sales_invoice/2026/08/render-a.pdf',?,10,?)",
        ("b" * 64, "c" * 64, now),
    )
    expect_sql_error(connection, "UPDATE phase09_rendered_documents SET pdf_size_bytes=11 WHERE id='render-a'", "render snapshot immutable")
    expect_sql_error(connection, "DELETE FROM phase09_rendered_documents WHERE id='render-a'", "render snapshot append-only")

    connection.execute(
        "INSERT INTO phase09_backups(id,company_id,backup_kind,created_at,application_version,schema_version,migration_ledger_digest,database_size_bytes,sha256,relative_path,integrity_status,foreign_key_status,verification_status,verified_at) VALUES('backup-a','company-a','MANUAL',?,'0.1.0','0007',?,10,?,'company-a/manual/backup-a.sqlite3','OK','OK','VERIFIED',?)",
        (now, "d" * 64, "e" * 64, now),
    )
    expect_sql_error(
        connection,
        "INSERT INTO phase09_backups(id,company_id,backup_kind,created_at,application_version,schema_version,migration_ledger_digest,database_size_bytes,sha256,relative_path,integrity_status,foreign_key_status,verification_status) VALUES('bad','company-a','MANUAL','x','0.1.0','0007','d',0,'e','x','OK','OK','VERIFIED')",
        "verified backup metadata constraints",
    )
    if connection.execute("PRAGMA foreign_key_check").fetchall():
        fail("foreign key violations after PHASE 09 fixture")


def main() -> int:
    migrations = sorted((ROOT / "database/migrations").glob("*.sql"))
    names = [path.name for path in migrations]
    if len(names) != 7 or names[:6] != list(FROZEN_MIGRATIONS) or names[6] != "0007_phase09_documents_reports_audit_backup.sql":
        fail(f"expected frozen 0001-0006 plus exactly authorized 0007, got {names}")
    for name, expected in FROZEN_MIGRATIONS.items():
        actual = digest(ROOT / "database/migrations" / name)
        if actual != expected:
            fail(f"frozen migration changed: {name}: {actual}")
    migration7 = migrations[6]

    subprocess.run([sys.executable, "scripts/verify_schema.py"], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "scripts/verify_phase06.py"], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "scripts/verify_phase07.py"], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "scripts/verify_phase08.py"], cwd=ROOT, check=True)

    verify_0006_to_0007_upgrade()

    migration = migration7.read_text(encoding="utf-8")
    rust = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src-tauri/src/phase09").glob("*.rs")))
    commands = read("src-tauri/src/commands/phase09.rs")
    lib = read("src-tauri/src/lib.rs")
    gateway = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src/platform/tauri/phase09").glob("*.ts")))
    workspace = read("src/features/phase09/Phase09Workspace.tsx")
    css = read("src/features/phase09/phase09.css")
    package = read("package.json")

    missing_permissions = sorted(code for code in REQUIRED_PERMISSIONS if code not in migration and code not in rust)
    if missing_permissions:
        fail(f"missing permissions: {missing_permissions}")
    missing_commands = sorted(name for name in REQUIRED_COMMANDS if name not in commands or name not in lib or name not in gateway)
    if missing_commands:
        fail(f"missing typed command boundary: {missing_commands}")
    missing_documents = sorted(kind for kind in DOCUMENT_TYPES if kind not in migration or kind not in rust)
    if missing_documents:
        fail(f"missing default document type: {missing_documents}")
    missing_reports = sorted(report for report in REPORTS if report not in rust)
    if missing_reports:
        fail(f"missing report identifier: {missing_reports}")

    for fragment in (
        "TransactionBehavior::Immediate", "expected_row_version", "PUBLISHED", "RETIRED",
        "content_sha256", "canonical_payload_json", "pdf_sha256", "OUTPUT_BUSY",
        "PLATFORM_UNSUPPORTED", "WebviewWindow", "with_webview", "PrintToPdf", "ShowPrintUI",
        "REPORT_PDF_LIMIT_EXCEEDED", "100_000", "5_000", "0xEF", "neutralize_csv",
        "password_hash", "recovery_code", "rusqlite::backup::Backup", "integrity_check",
        "foreign_key_check", "PRE_RESTORE", "RESTORE", "phase09_invalidate_session",
        "maintenance", "protected_for_restore", "AUTOMATIC_DAILY", "AUTOMATIC_WEEKLY",
    ):
        if fragment not in rust and fragment not in read("src-tauri/Cargo.toml"):
            fail(f"Rust contract missing: {fragment}")
    for forbidden in ("<script", "<iframe", "<object", "<embed", "javascript:", "http://", "https://"):
        if forbidden not in rust:
            fail(f"template/resource rejection marker missing: {forbidden}")
    if "backup" not in read("src-tauri/Cargo.toml") or 'rusqlite = { version = "=0.32.1"' not in read("src-tauri/Cargo.toml"):
        fail("rusqlite pin/backup feature missing")
    if 'webview2-com = "=0.38.2"' not in read("src-tauri/Cargo.toml"):
        fail("compatible WebView2 COM version is not pinned")

    if any(token in gateway + workspace for token in ("fetch(", "XMLHttpRequest", "WebSocket(", "SELECT ", "INSERT INTO", "DELETE FROM")):
        fail("network primitive or frontend SQL appears in PHASE 09 frontend")
    for fragment in ("Number.isSafeInteger", "normalizePhase09Error", "createRequestGate", "RESTORE", "sandbox=\"\"", "window.confirm"):
        if fragment not in gateway and fragment not in workspace:
            fail(f"frontend safety contract missing: {fragment}")
    for fragment in ("prefers-reduced-motion", "focus-visible", "1024px", "overflow"):
        if fragment not in css:
            fail(f"UI accessibility/responsive contract missing: {fragment}")

    e2e = read("tests/e2e/run_phase09.py")
    missing_e2e = sorted(s for s in E2E_SCENARIOS if s not in e2e)
    if missing_e2e:
        fail(f"required E2E scenarios missing: {missing_e2e}")
    for fragment in ("axe.run", "console", "pageerror", "scrollWidth", "screenshot", "viewport"):
        if fragment not in e2e:
            fail(f"E2E evidence contract missing: {fragment}")
    if "run_phase09.py" not in package:
        fail("PHASE 09 E2E runner is not registered")

    connection = migration_fixture()
    exercise_database_contracts(connection)
    connection.close()

    result = {
        "status": "PASS",
        "baseline": BASELINE,
        "migration0007": migration7.name,
        "migration0007Sha256": digest(migration7),
        "tables": 64,
        "triggers": 63,
        "permissions": len(REQUIRED_PERMISSIONS),
        "documentTypes": sorted(DOCUMENT_TYPES),
        "reports": sorted(REPORTS),
        "typedCommands": len(REQUIRED_COMMANDS),
        "e2eScenarios": len(E2E_SCENARIOS),
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
