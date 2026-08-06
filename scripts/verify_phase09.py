#!/usr/bin/env python3
"""Verify the additive POSMAN PHASE 09 database and source contracts."""
from __future__ import annotations

import hashlib
import json
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MIGRATIONS = ROOT / "database" / "migrations"
FROZEN = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
    "0006_accounting_payments_hardening.sql": "08763076ce7cbd77e585bf06b10bc856e7b8f02193484b1db974db95143cebd0",
}
REQUIRED_PERMISSIONS = {
    "documents.templates.view": 0,
    "documents.templates.manage": 1,
    "documents.render": 0,
    "documents.print": 0,
    "documents.export": 0,
    "reports.view": 0,
    "reports.export": 0,
    "audit.view": 0,
    "audit.export": 1,
    "backup.view": 0,
    "backup.create": 1,
    "backup.restore": 1,
    "backup.manage": 1,
}
DOCUMENT_TYPES = {
    "SALES_ORDER", "DELIVERY_NOTE", "SALES_INVOICE", "SALES_CREDIT_NOTE",
    "PURCHASE_ORDER", "GOODS_RECEIPT", "SUPPLIER_INVOICE", "PURCHASE_RETURN",
    "CUSTOMER_RECEIPT", "SUPPLIER_PAYMENT",
}
REPORTS = {
    "SALES_SUMMARY", "SALES_BY_PRODUCT", "SALES_BY_CUSTOMER", "PURCHASES_SUMMARY",
    "PURCHASES_BY_SUPPLIER", "STOCK_ON_HAND", "STOCK_VALUATION", "STOCK_MOVEMENTS",
    "LOW_STOCK", "OPEN_RECEIVABLES", "OPEN_PAYABLES", "CASH_BANK_REGISTER", "TRIAL_BALANCE",
}
COMMANDS = {
    "phase09_list_templates", "phase09_get_template", "phase09_create_template_draft",
    "phase09_update_template_draft", "phase09_publish_template", "phase09_retire_template",
    "phase09_preview_document", "phase09_render_document", "phase09_list_rendered_documents",
    "phase09_get_rendered_document", "phase09_verify_rendered_document",
    "phase09_export_rendered_pdf", "phase09_print_rendered_document", "phase09_list_reports",
    "phase09_run_report", "phase09_export_report_csv", "phase09_export_report_pdf",
    "phase09_list_audit_events", "phase09_export_audit_csv", "phase09_get_backup_settings",
    "phase09_update_backup_settings", "phase09_create_backup", "phase09_list_backups",
    "phase09_verify_backup", "phase09_export_backup", "phase09_import_backup",
    "phase09_restore_backup", "phase09_delete_backup",
}


def fail(message: str) -> None:
    raise SystemExit(f"PHASE09 VERIFY FAILED: {message}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def apply(connection: sqlite3.Connection, paths: list[Path]) -> None:
    for path in paths:
        sql = path.read_text(encoding="utf-8")
        checksum = hashlib.sha256(sql.encode()).hexdigest()
        version = path.name[:4]
        connection.executescript(
            "BEGIN IMMEDIATE;\n" + sql + "\n" +
            "INSERT INTO app_migrations(id,version,name,checksum_sha256,applied_at) VALUES(" +
            f"{int(version)},'{version}','{path.stem[5:]}','{checksum}','2026-08-06T00:00:00Z');\nCOMMIT;"
        )


def expect_rejected(connection: sqlite3.Connection, sql: str, label: str) -> None:
    try:
        connection.execute(sql)
    except sqlite3.DatabaseError:
        connection.rollback()
        return
    fail(f"constraint was not enforced: {label}")


def database_contracts(paths: list[Path]) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="posman-p09-") as tmp:
        fresh_path = Path(tmp) / "fresh.sqlite3"
        connection = sqlite3.connect(fresh_path)
        connection.execute("PRAGMA foreign_keys=ON")
        apply(connection, paths)
        connection.executescript((ROOT / "database/seed/reference_data.sql").read_text(encoding="utf-8"))

        tables = {row[0] for row in connection.execute("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'")}
        for table in {
            "document_template_defaults", "document_template_drafts", "document_template_publications",
            "document_render_snapshots", "backup_policies", "verified_backups",
        }:
            if table not in tables:
                fail(f"required table missing: {table}")
        if len(tables) != 63:
            fail(f"expected 63 application tables, found {len(tables)}")

        actual_permissions = dict(connection.execute(
            "SELECT code,is_sensitive FROM permissions WHERE code IN (%s)" % ",".join("?" for _ in REQUIRED_PERMISSIONS),
            tuple(REQUIRED_PERMISSIONS),
        ))
        if actual_permissions != REQUIRED_PERMISSIONS:
            fail(f"permission contract mismatch: {actual_permissions}")

        defaults = connection.execute(
            "SELECT document_type,locale,configuration_json FROM document_template_defaults"
        ).fetchall()
        if len(defaults) != 20 or {row[0] for row in defaults} != DOCUMENT_TYPES or {row[1] for row in defaults} != {"ar-DZ", "fr-DZ"}:
            fail("default template matrix must be 10 document types x 2 locales")
        for _, _, config in defaults:
            parsed = json.loads(config)
            if not isinstance(parsed, dict) or any(token in config.lower() for token in ("<script", "http://", "https://", "javascript:")):
                fail("default template configuration is unsafe")

        now = "2026-08-06T10:00:00Z"
        connection.execute("INSERT INTO companies(id,code,legal_name,name_ar,name_fr,created_at,updated_at) VALUES('c1','C1','Company','شركة','Société',?,?)", (now, now))
        connection.execute("INSERT INTO users(id,company_id,username,display_name,password_hash,created_at,updated_at) VALUES('u1','c1','admin','Admin',?, ?, ?)", ("x"*32, now, now))
        connection.execute("INSERT INTO document_templates(id,company_id,document_type,code,name_ar,name_fr,created_at,created_by,updated_at,updated_by,locale) VALUES('t1','c1','SALES_INVOICE','invoice-ar','فاتورة','Invoice',?,'u1',?,'u1','ar-DZ')", (now, now))
        connection.execute("INSERT INTO document_template_versions(id,company_id,document_template_id,version_number,html_template,css_template,content_hash_sha256,is_published,created_at,created_by,locale,configuration_json,published_at,published_by) VALUES('v1','c1','t1',1,'safe','safe',?,1,?,'u1','ar-DZ','{}',?,'u1')", ("a"*64, now, now))
        connection.execute("INSERT INTO document_template_publications(id,company_id,document_template_id,template_version_id,locale,status,activated_at,activated_by) VALUES('p1','c1','t1','v1','ar-DZ','PUBLISHED',?,'u1')", (now,))
        connection.execute("INSERT INTO document_render_snapshots(id,company_id,document_type,source_entity_kind,source_document_id,source_document_number,source_document_status,template_id,template_version_id,locale,canonical_payload_json,rendered_html,rendered_css,content_sha256,pdf_relative_path,pdf_sha256,pdf_size_bytes,rendered_at,rendered_by) VALUES('r1','c1','SALES_INVOICE','COMMERCIAL_DOCUMENT','d1','FV-1','POSTED','t1','v1','ar-DZ','{}','<html></html>','',?,'c1/SALES_INVOICE/2026/08/r1.pdf',?,100,?,'u1')", ("b"*64, "c"*64, now))
        connection.commit()
        expect_rejected(connection, "UPDATE document_template_publications SET activated_at='x' WHERE id='p1'", "published template identity immutable")
        expect_rejected(connection, "DELETE FROM document_template_publications WHERE id='p1'", "published template cannot be deleted")
        expect_rejected(connection, "UPDATE document_render_snapshots SET rendered_css='x' WHERE id='r1'", "render snapshot immutable")
        expect_rejected(connection, "DELETE FROM document_render_snapshots WHERE id='r1'", "render snapshot cannot be deleted")
        expect_rejected(connection, "INSERT INTO verified_backups(id,company_id,backup_kind,created_at,application_version,schema_version,migration_ledger_digest,database_size_bytes,sha256,relative_path,integrity_status,foreign_key_status,verification_status) VALUES('b1','c1','MANUAL',?, '0.1.0','0007',?,1,?,'../escape','OK','OK','VERIFIED')".replace("?", "'"+now+"'", 1).replace("?", "'"+"d"*64+"'", 1).replace("?", "'"+"e"*64+"'", 1), "backup traversal rejected")

        if connection.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            fail("fresh database integrity_check failed")
        if connection.execute("PRAGMA foreign_key_check").fetchall():
            fail("fresh database foreign_key_check returned rows")
        connection.close()

        upgrade_path = Path(tmp) / "upgrade.sqlite3"
        upgrade = sqlite3.connect(upgrade_path)
        upgrade.execute("PRAGMA foreign_keys=ON")
        apply(upgrade, paths[:6])
        apply(upgrade, paths[6:])
        if upgrade.execute("SELECT MAX(version) FROM app_migrations").fetchone()[0] != "0007":
            fail("0006 to 0007 upgrade did not reach version 0007")
        if upgrade.execute("PRAGMA foreign_key_check").fetchall():
            fail("0006 to 0007 upgrade has foreign-key violations")
        upgrade.close()

        return {"tables": 63, "defaults": len(defaults), "permissions": len(actual_permissions)}


def source_contracts() -> dict[str, int]:
    rust_root = ROOT / "src-tauri/src/phase09"
    rust = "\n".join(path.read_text(encoding="utf-8") for path in sorted(rust_root.rglob("*.rs"))) + "\n" + (ROOT / "src-tauri/src/phase05/mod.rs").read_text(encoding="utf-8") + "\n" + (ROOT / "src-tauri/src/infrastructure/maintenance.rs").read_text(encoding="utf-8")
    commands = (ROOT / "src-tauri/src/commands/phase09.rs").read_text(encoding="utf-8")
    lib = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
    gateway = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src/platform/tauri/phase09").glob("*.ts")))
    ui = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src/features/phase09").rglob("*")) if path.is_file())

    missing = sorted(command for command in COMMANDS if command not in commands or command not in lib or command not in gateway)
    if missing:
        fail(f"typed command boundary missing: {missing}")
    for report in REPORTS:
        if report not in rust or report not in gateway:
            fail(f"report identifier missing: {report}")
    required_rust = (
        "TransactionBehavior::Immediate", "documents.templates.manage", "OUTPUT_BUSY", "PLATFORM_UNSUPPORTED",
        "PrintToPdf", "ShowPrintUI", "canonical_payload_json", "verify_existing_pdf", "UTF8_BOM",
        "formula", "integrity_check", "foreign_key_check", "backup::Backup", "PRE_RESTORE",
        "RESTORE", "verify_current_password", "phase09_invalidate_session", "phase09_begin_restore",
    )
    for fragment in required_rust:
        if fragment not in rust and fragment not in commands:
            fail(f"Rust PHASE 09 contract missing: {fragment}")
    if any(token in gateway for token in ("fetch(", "XMLHttpRequest", "WebSocket(", "SELECT ", "INSERT ", "UPDATE ", "DELETE FROM")):
        fail("frontend PHASE 09 gateway contains network or SQL authority")
    for token in ("dir={content.direction}", "phase09", "RESTORE", "aria-live", "prefers-reduced-motion"):
        if token not in ui:
            fail(f"PHASE 09 UI contract missing: {token}")
    return {"commands": len(COMMANDS), "reports": len(REPORTS), "rustFiles": len(list(rust_root.glob("*.rs")))}


def main() -> int:
    paths = sorted(MIGRATIONS.glob("[0-9][0-9][0-9][0-9]_*.sql"))
    if [path.name[:4] for path in paths] != [f"{index:04d}" for index in range(1, 8)]:
        fail("exactly seven ordered migrations through 0007 are required")
    for name, expected in FROZEN.items():
        actual = sha(MIGRATIONS / name)
        if actual != expected:
            fail(f"accepted migration changed: {name}: {actual}")
    if paths[6].name != "0007_phase09_documents_reports_audit_backup.sql":
        fail(f"unexpected migration 0007 name: {paths[6].name}")

    subprocess.run([sys.executable, "scripts/verify_schema.py"], cwd=ROOT, check=True)
    db = database_contracts(paths)
    source = source_contracts()
    result = {
        "status": "PASS",
        "migration0007": paths[6].name,
        "migration0007Sha256": sha(paths[6]),
        **db,
        **source,
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
