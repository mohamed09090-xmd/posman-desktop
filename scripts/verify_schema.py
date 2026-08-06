#!/usr/bin/env python3
"""Build and verify the accepted POSMAN SQLite foundation through PHASE 08."""

from __future__ import annotations

import argparse
import hashlib
import sqlite3
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Iterable

ROOT = Path(__file__).resolve().parents[1]
MIGRATIONS_DIR = ROOT / "database" / "migrations"
SEED_FILE = ROOT / "database" / "seed" / "reference_data.sql"
INVARIANTS_FILE = ROOT / "database" / "tests" / "invariants.sql"
SCHEMA_FILE = ROOT / "database" / "schema.sql"
BLUEPRINT_FILE = ROOT / "docs" / "spec" / "POSMAN-Blueprint-v1.md"
EXPECTED_BLUEPRINT_SHA256 = "d932aa0b36099d5ad5dbbb873abc39c957393349af7e1dd6565af06f08be8a84"
ERD_FILE = ROOT / "docs" / "architecture" / "erd.md"
NOW = "2026-07-31T12:00:00Z"
TODAY = "2026-01-15"

ACCEPTED_MIGRATION_HASHES = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
}

EXPECTED_TABLES = {
    "app_migrations",
    "companies",
    "company_settings",
    "fiscal_years",
    "fiscal_periods",
    "document_sequences",
    "users",
    "roles",
    "permissions",
    "user_roles",
    "role_permissions",
    "sessions",
    "units",
    "tax_rates",
    "payment_terms",
    "payment_methods",
    "warehouses",
    "warehouse_locations",
    "product_families",
    "products",
    "price_lists",
    "product_prices",
    "partners",
    "partner_addresses",
    "partner_contacts",
    "commercial_documents",
    "commercial_document_lines",
    "document_line_links",
    "document_status_history",
    "payments",
    "payment_allocations",
    "stock_movements",
    "stock_balances",
    "stock_reservations",
    "inventory_counts",
    "inventory_count_lines",
    "accounts",
    "accounting_journals",
    "posting_rules",
    "journal_entries",
    "journal_entry_lines",
    "posting_attempts",
    "document_templates",
    "document_template_versions",
    "rendered_documents",
    "attachments",
    "audit_logs",
    "idempotency_keys",
    "backup_history",
    "setup_drafts",
    "initial_setup_requests",
    "user_recovery_codes",
    "accounting_setups",
    "accounting_account_roles",
    "posting_rule_lines",
    "payment_method_accounting",
    "fiscal_period_events",
}
EXPECTED_TEXT_PRIMARY_KEYS = 55
EXPECTED_TRIGGER_COUNT = 47


class VerificationError(RuntimeError):
    """Raised when a required verification condition fails."""


@dataclass
class Evidence:
    passed: list[str] = field(default_factory=list)
    pending: list[str] = field(default_factory=list)

    def record(self, name: str) -> None:
        self.passed.append(name)

    def require(self, condition: bool, name: str, detail: str | None = None) -> None:
        if not condition:
            raise VerificationError(detail or name)
        self.record(name)


def migration_files(evidence: Evidence) -> list[Path]:
    files = sorted(MIGRATIONS_DIR.glob("[0-9][0-9][0-9][0-9]_*.sql"))
    if not files:
        raise VerificationError("no migration files found")
    expected_versions = [f"{index:04d}" for index in range(1, len(files) + 1)]
    actual_versions = [path.name[:4] for path in files]
    evidence.require(
        actual_versions == expected_versions == ["0001", "0002", "0003", "0004", "0005", "0006"],
        "six contiguous ordered migrations through 0006",
        f"migration versions must be contiguous through 0006: got {actual_versions}",
    )
    for name, expected_hash in ACCEPTED_MIGRATION_HASHES.items():
        actual_hash = hashlib.sha256((MIGRATIONS_DIR / name).read_bytes()).hexdigest()
        evidence.require(
            actual_hash == expected_hash,
            f"accepted migration hash unchanged: {name}",
            f"accepted migration changed: {name}: {actual_hash} != {expected_hash}",
        )
    return files


def generated_schema_text(files: Iterable[Path]) -> str:
    sections = [
        "-- GENERATED FILE: ordered migrations are authoritative.",
        "-- Regenerate with: python scripts/verify_schema.py --write-schema",
        "",
    ]
    for path in files:
        sections.append(f"-- BEGIN MIGRATION {path.name}")
        sections.append(path.read_text(encoding="utf-8").rstrip())
        sections.append(f"-- END MIGRATION {path.name}")
        sections.append("")
    return "\n".join(sections).rstrip() + "\n"


def verify_schema_snapshot(files: list[Path], write_schema: bool, evidence: Evidence) -> None:
    expected = generated_schema_text(files)
    if write_schema:
        SCHEMA_FILE.write_text(expected, encoding="utf-8", newline="\n")
    if not SCHEMA_FILE.exists():
        raise VerificationError(
            "database/schema.sql is missing; run python scripts/verify_schema.py --write-schema"
        )
    actual = SCHEMA_FILE.read_text(encoding="utf-8")
    evidence.require(
        actual == expected,
        "database/schema.sql exactly matches ordered migrations",
        "database/schema.sql does not match ordered migrations; run python scripts/verify_schema.py --write-schema",
    )


def sql_literal(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


def apply_migrations(
    connection: sqlite3.Connection,
    files: list[Path],
    *,
    start_index: int = 0,
) -> None:
    for path in files[start_index:]:
        sql = path.read_text(encoding="utf-8")
        checksum = hashlib.sha256(sql.encode("utf-8")).hexdigest()
        version = path.name[:4]
        logical_name = path.stem[5:]
        ledger = (
            "INSERT INTO app_migrations "
            "(id, version, name, checksum_sha256, applied_at) VALUES ("
            f"{int(version)}, {sql_literal(version)}, {sql_literal(logical_name)}, "
            f"{sql_literal(checksum)}, {sql_literal(NOW)});"
        )
        try:
            connection.executescript(f"BEGIN IMMEDIATE;\n{sql}\n{ledger}\nCOMMIT;")
        except sqlite3.DatabaseError:
            connection.rollback()
            raise


def apply_seed_twice(connection: sqlite3.Connection, evidence: Evidence) -> None:
    seed_sql = SEED_FILE.read_text(encoding="utf-8")
    connection.executescript(f"BEGIN;\n{seed_sql}\nCOMMIT;")
    first_counts = connection.execute(
        "SELECT (SELECT COUNT(*) FROM roles WHERE company_id IS NULL), "
        "       (SELECT COUNT(*) FROM permissions), "
        "       (SELECT COUNT(*) FROM role_permissions)"
    ).fetchone()
    connection.executescript(f"BEGIN;\n{seed_sql}\nCOMMIT;")
    second_counts = connection.execute(
        "SELECT (SELECT COUNT(*) FROM roles WHERE company_id IS NULL), "
        "       (SELECT COUNT(*) FROM permissions), "
        "       (SELECT COUNT(*) FROM role_permissions)"
    ).fetchone()
    evidence.require(
        first_counts == second_counts,
        "applied deterministic reference seed twice",
        f"reference seed is not deterministic: first={first_counts}, second={second_counts}",
    )


def validate_repository_documents(evidence: Evidence) -> None:
    if not BLUEPRINT_FILE.exists():
        raise VerificationError("authoritative Blueprint copy is missing")
    blueprint_hash = hashlib.sha256(BLUEPRINT_FILE.read_bytes()).hexdigest()
    evidence.require(
        blueprint_hash == EXPECTED_BLUEPRINT_SHA256,
        "authoritative Blueprint SHA-256 matched supplied source",
        f"Blueprint checksum mismatch: expected {EXPECTED_BLUEPRINT_SHA256}, got {blueprint_hash}",
    )

    erd = ERD_FILE.read_text(encoding="utf-8")
    mermaid_fences = erd.count("```mermaid")
    closing_fences = erd.count("```")
    evidence.require(
        mermaid_fences >= 2 and closing_fences >= mermaid_fences,
        f"validated {mermaid_fences} Mermaid blocks structurally",
        "ERD Mermaid fences are structurally incomplete",
    )
    evidence.require(
        "erDiagram" in erd and "flowchart" in erd,
        "ERD includes domain erDiagram and lineage flowchart",
        "ERD must include both domain erDiagram and lineage flowchart blocks",
    )

    prohibited_names = {".env", ".env.local", ".env.production"}
    prohibited_suffixes = {
        ".sqlite",
        ".sqlite3",
        ".db",
        ".pem",
        ".p12",
        ".pfx",
        ".wal",
        ".shm",
    }
    secret_markers = (
        "-----BEGIN " + "PRIVATE KEY-----",
        "gh" + "p_",
        "github" + "_pat_",
        "sk" + "-proj-",
    )
    ignored_roots = {".git", "node_modules", "target", "dist"}
    for path in ROOT.rglob("*"):
        if not path.is_file() or any(part in ignored_roots for part in path.relative_to(ROOT).parts):
            continue
        relative = path.relative_to(ROOT)
        if path.name in prohibited_names or path.suffix.lower() in prohibited_suffixes:
            raise VerificationError(f"prohibited secret/database-like file present: {relative}")
        if path.suffix.lower() in {
            ".md",
            ".sql",
            ".py",
            ".rs",
            ".ts",
            ".tsx",
            ".json",
            ".yml",
            ".yaml",
            ".txt",
        }:
            text = path.read_text(encoding="utf-8", errors="ignore")
            for marker in secret_markers:
                if marker in text:
                    raise VerificationError(f"secret-like marker {marker!r} found in {relative}")
    evidence.record("found no prohibited secret-like or database artifact files")


def assert_core_schema(connection: sqlite3.Connection, evidence: Evidence) -> None:
    evidence.require(
        connection.execute("PRAGMA foreign_keys").fetchone()[0] == 1,
        "PRAGMA foreign_keys is enabled",
    )

    actual_tables = {
        row[0]
        for row in connection.execute(
            "SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
    }
    evidence.require(
        actual_tables == EXPECTED_TABLES,
        f"found exactly {len(EXPECTED_TABLES)} expected tables",
        f"table mismatch: missing={sorted(EXPECTED_TABLES - actual_tables)}, "
        f"unexpected={sorted(actual_tables - EXPECTED_TABLES)}",
    )

    violations = connection.execute("PRAGMA foreign_key_check").fetchall()
    evidence.require(
        not violations,
        "PRAGMA foreign_key_check returned no rows",
        f"foreign key violations: {violations}",
    )

    real_columns: list[str] = []
    nullable_text_primary_keys: list[str] = []
    text_primary_key_count = 0
    for table_name in sorted(actual_tables):
        for column in connection.execute(f'PRAGMA table_info("{table_name}")'):
            column_name = column[1]
            declared_type = (column[2] or "").strip().upper()
            explicitly_not_null = column[3]
            primary_key_position = column[5]
            if "REAL" in declared_type:
                real_columns.append(f"{table_name}.{column_name}:{declared_type}")
            if column_name == "id" and declared_type == "TEXT" and primary_key_position > 0:
                text_primary_key_count += 1
                if explicitly_not_null != 1:
                    nullable_text_primary_keys.append(table_name)
    evidence.require(
        not real_columns,
        "found no application column declared as REAL",
        f"REAL columns are prohibited: {real_columns}",
    )
    evidence.require(
        text_primary_key_count == EXPECTED_TEXT_PRIMARY_KEYS,
        f"found {EXPECTED_TEXT_PRIMARY_KEYS} TEXT business primary keys",
        f"expected {EXPECTED_TEXT_PRIMARY_KEYS} TEXT business primary keys, found {text_primary_key_count}",
    )
    evidence.require(
        not nullable_text_primary_keys,
        "all TEXT business primary keys are explicitly NOT NULL",
        "TEXT business primary key id is nullable in: "
        + ", ".join(sorted(nullable_text_primary_keys)),
    )

    trigger_count = connection.execute(
        "SELECT COUNT(*) FROM sqlite_schema WHERE type='trigger'"
    ).fetchone()[0]
    evidence.require(
        trigger_count == EXPECTED_TRIGGER_COUNT,
        f"found exactly {EXPECTED_TRIGGER_COUNT} accepted integrity triggers",
        f"expected {EXPECTED_TRIGGER_COUNT} triggers, found {trigger_count}",
    )

    ledger = connection.execute(
        "SELECT id, version, name, checksum_sha256 FROM app_migrations ORDER BY id"
    ).fetchall()
    evidence.require(
        [row[1] for row in ledger] == ["0001", "0002", "0003", "0004", "0005", "0006"],
        "migration ledger reaches schema version 0006",
        f"unexpected migration ledger: {ledger}",
    )


def insert_document(
    connection: sqlite3.Connection,
    *,
    document_id: str,
    document_type: str,
    document_number: str,
    workflow_status: str,
    partner_id: str | None,
    idempotency_key: str,
    commercial_date: str = TODAY,
) -> None:
    connection.execute(
        """
        INSERT INTO commercial_documents (
            id, company_id, fiscal_year_id, fiscal_period_id, partner_id, warehouse_id,
            document_type, document_number, workflow_status, commercial_date,
            idempotency_key, created_at, created_by, updated_at, updated_by
        ) VALUES (?, 'company-1', 'fy-2026', 'period-open', ?, 'warehouse-main',
                  ?, ?, ?, ?, ?, ?, 'fixture-user', ?, 'fixture-user')
        """,
        (
            document_id,
            partner_id,
            document_type,
            document_number,
            workflow_status,
            commercial_date,
            idempotency_key,
            NOW,
            NOW,
        ),
    )


def insert_line(
    connection: sqlite3.Connection,
    *,
    line_id: str,
    document_id: str,
    line_number: int,
    quantity_scaled: int,
    unit_price_scaled: int = 100000,
    unit_cost_scaled: int = 70000,
) -> None:
    connection.execute(
        """
        INSERT INTO commercial_document_lines (
            id, company_id, document_id, product_id, warehouse_id, unit_id, line_number,
            product_code_snapshot, description_snapshot, unit_code_snapshot, tax_code_snapshot,
            quantity_scaled, unit_price_scaled, unit_cost_scaled, tax_rate_scaled,
            line_ht_minor, line_tax_minor, line_ttc_minor,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?, 'company-1', ?, 'product-1', 'warehouse-main', 'unit-piece', ?,
                  'P-001', 'Fixture product', 'PCE', NULL,
                  ?, ?, ?, 0, 0, 0, 0, ?, 'fixture-user', ?, 'fixture-user')
        """,
        (
            line_id,
            document_id,
            line_number,
            quantity_scaled,
            unit_price_scaled,
            unit_cost_scaled,
            NOW,
            NOW,
        ),
    )


def post_document(connection: sqlite3.Connection, document_id: str, workflow_status: str) -> None:
    connection.execute(
        """
        UPDATE commercial_documents
        SET workflow_status = ?, posting_status = 'POSTED', posting_date = commercial_date,
            posted_at = ?, posted_by = 'fixture-user', updated_at = ?,
            updated_by = 'fixture-user', row_version = row_version + 1
        WHERE id = ?
        """,
        (workflow_status, NOW, NOW, document_id),
    )


def create_positive_fixtures(connection: sqlite3.Connection, evidence: Evidence) -> None:
    connection.execute(
        """
        INSERT INTO companies (
            id, code, legal_name, name_ar, name_fr, created_at, created_by,
            updated_at, updated_by
        ) VALUES ('company-1', 'POSMAN-FIXTURE', 'POSMAN Fixture SARL', 'مؤسسة اختبار بوسمان',
                  'Société de test POSMAN', ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO company_settings (
            id, company_id, created_at, created_by, updated_at, updated_by
        ) VALUES ('settings-1', 'company-1', ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO fiscal_years (
            id, company_id, code, starts_on, ends_on, status,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('fy-2026', 'company-1', '2026', '2026-01-01', '2026-12-31', 'OPEN',
                  ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.executemany(
        """
        INSERT INTO fiscal_periods (
            id, company_id, fiscal_year_id, period_number, name, starts_on, ends_on, status,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?, 'company-1', 'fy-2026', ?, ?, ?, ?, ?, ?, 'fixture-user', ?, 'fixture-user')
        """,
        [
            ("period-open", 1, "January", "2026-01-01", "2026-01-31", "OPEN", NOW, NOW),
            ("period-closed", 2, "February", "2026-02-01", "2026-02-28", "CLOSED", NOW, NOW),
        ],
    )
    connection.execute(
        """
        INSERT INTO warehouses (
            id, company_id, code, name_ar, name_fr, is_default,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('warehouse-main', 'company-1', 'MAIN', 'المستودع الرئيسي', 'Dépôt principal', 1,
                  ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO warehouse_locations (
            id, company_id, warehouse_id, code, name_ar, name_fr,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('location-main', 'company-1', 'warehouse-main', 'DEFAULT', 'الموقع الرئيسي',
                  'Emplacement principal', ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO units (
            id, company_id, code, name_ar, name_fr, decimal_scale,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('unit-piece', 'company-1', 'PCE', 'قطعة', 'Pièce', 0,
                  ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO product_families (
            id, company_id, code, name_ar, name_fr,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('family-1', 'company-1', 'F-001', 'عائلة اختبار', 'Famille de test',
                  ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO products (
            id, company_id, product_family_id, unit_id, code, barcode, name_ar, name_fr,
            product_kind, stock_tracked, minimum_stock_scaled,
            default_purchase_price_scaled, default_sale_price_scaled,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('product-1', 'company-1', 'family-1', 'unit-piece', 'P-001', '613000000001',
                  'مادة اختبار', 'Article de test', 'STOCK_ITEM', 1, 1000000,
                  70000, 100000, ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.executemany(
        """
        INSERT INTO partners (
            id, company_id, code, legal_name, display_name_ar, display_name_fr,
            is_customer, is_supplier, created_at, created_by, updated_at, updated_by
        ) VALUES (?, 'company-1', ?, ?, ?, ?, ?, ?, ?, 'fixture-user', ?, 'fixture-user')
        """,
        [
            ("customer-1", "C-001", "Fixture Customer", "عميل اختبار", "Client de test", 1, 0, NOW, NOW),
            ("supplier-1", "S-001", "Fixture Supplier", "مورد اختبار", "Fournisseur de test", 0, 1, NOW, NOW),
        ],
    )
    connection.execute(
        """
        INSERT INTO payment_methods (
            id, company_id, code, name_ar, name_fr, method_kind,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('payment-cash', 'company-1', 'CASH', 'نقدًا', 'Espèces', 'CASH',
                  ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    evidence.record("created company, fiscal year, open and closed periods")
    evidence.record("created warehouse, location, family, unit, and product")
    evidence.record("created customer and supplier")

    insert_document(
        connection,
        document_id="opening-doc",
        document_type="OPENING_STOCK",
        document_number="OS-000001",
        workflow_status="DRAFT",
        partner_id=None,
        idempotency_key="document:opening-doc",
    )
    insert_line(
        connection,
        line_id="opening-line",
        document_id="opening-doc",
        line_number=1,
        quantity_scaled=10000000,
    )
    connection.execute(
        """
        INSERT INTO stock_movements (
            id, company_id, product_id, warehouse_id, warehouse_location_id,
            source_document_id, source_line_id, movement_type, business_date, occurred_at,
            quantity_delta_scaled, quantity_before_scaled, quantity_after_scaled,
            unit_cost_scaled, average_cost_before_scaled, average_cost_after_scaled,
            extended_cost_minor, posting_event_key, created_by
        ) VALUES ('movement-opening', 'company-1', 'product-1', 'warehouse-main', 'location-main',
                  'opening-doc', 'opening-line', 'OPENING', ?, ?,
                  10000000, 0, 10000000, 70000, 0, 70000, 70000,
                  'opening-stock:opening-doc:opening-line', 'fixture-user')
        """,
        (TODAY, NOW),
    )
    connection.execute(
        """
        INSERT INTO stock_balances (
            id, company_id, product_id, warehouse_id, warehouse_location_id, last_movement_id,
            on_hand_scaled, reserved_scaled, available_scaled, average_cost_scaled, rebuilt_at
        ) VALUES ('balance-1', 'company-1', 'product-1', 'warehouse-main', 'location-main',
                  'movement-opening', 10000000, 0, 10000000, 70000, ?)
        """,
        (NOW,),
    )
    post_document(connection, "opening-doc", "POSTED")
    evidence.record("created and posted opening-stock document and movement")

    insert_document(
        connection,
        document_id="sales-order-1",
        document_type="SALES_ORDER",
        document_number="SO-000001",
        workflow_status="CONFIRMED",
        partner_id="customer-1",
        idempotency_key="document:sales-order-1",
    )
    insert_line(
        connection,
        line_id="line-sales-order-1",
        document_id="sales-order-1",
        line_number=1,
        quantity_scaled=20000000,
    )
    evidence.record("created 20-unit sales order")

    for suffix, quantity in (("1", 8000000), ("2", 12000000)):
        insert_document(
            connection,
            document_id=f"delivery-{suffix}",
            document_type="DELIVERY_NOTE",
            document_number=f"DN-00000{suffix}",
            workflow_status="DRAFT",
            partner_id="customer-1",
            idempotency_key=f"document:delivery-{suffix}",
        )
        insert_line(
            connection,
            line_id=f"line-delivery-{suffix}",
            document_id=f"delivery-{suffix}",
            line_number=1,
            quantity_scaled=quantity,
        )
        connection.execute(
            """
            INSERT INTO document_line_links (
                id, company_id, source_line_id, target_line_id, transformation_type,
                transformed_quantity_scaled, created_at, created_by
            ) VALUES (?, 'company-1', 'line-sales-order-1', ?, 'ORDER_TO_DELIVERY', ?, ?, 'fixture-user')
            """,
            (f"link-order-delivery-{suffix}", f"line-delivery-{suffix}", quantity, NOW),
        )
        post_document(connection, f"delivery-{suffix}", "POSTED")
    evidence.record("created two posted deliveries for 8 and 12 units")

    insert_document(
        connection,
        document_id="sales-invoice-1",
        document_type="SALES_INVOICE",
        document_number="SI-000001",
        workflow_status="DRAFT",
        partner_id="customer-1",
        idempotency_key="document:sales-invoice-1",
    )
    for suffix, quantity in (("1", 8000000), ("2", 12000000)):
        insert_line(
            connection,
            line_id=f"line-invoice-{suffix}",
            document_id="sales-invoice-1",
            line_number=int(suffix),
            quantity_scaled=quantity,
        )
        connection.execute(
            """
            INSERT INTO document_line_links (
                id, company_id, source_line_id, target_line_id, transformation_type,
                transformed_quantity_scaled, created_at, created_by
            ) VALUES (?, 'company-1', ?, ?, 'DELIVERY_TO_INVOICE', ?, ?, 'fixture-user')
            """,
            (
                f"link-delivery-invoice-{suffix}",
                f"line-delivery-{suffix}",
                f"line-invoice-{suffix}",
                quantity,
                NOW,
            ),
        )
    post_document(connection, "sales-invoice-1", "POSTED")
    evidence.record("created posted invoice lines linked to delivered quantities")

    connection.execute(
        """
        INSERT INTO document_status_history (
            id, company_id, document_id, old_status, new_status, reason,
            row_version_snapshot, changed_at, changed_by
        ) VALUES ('status-history-1', 'company-1', 'sales-invoice-1', 'VALIDATED', 'POSTED',
                  'Fixture posting', 2, ?, 'fixture-user')
        """,
        (NOW,),
    )

    connection.executemany(
        """
        INSERT INTO accounts (
            id, company_id, code, name_ar, name_fr, account_type, normal_side,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?, 'company-1', ?, ?, ?, ?, ?, ?, 'fixture-user', ?, 'fixture-user')
        """,
        [
            ("account-debit", "CFG-DEBIT", "حساب مدين اختباري", "Compte débit test", "ASSET", "DEBIT", NOW, NOW),
            ("account-credit", "CFG-CREDIT", "حساب دائن اختباري", "Compte crédit test", "REVENUE", "CREDIT", NOW, NOW),
        ],
    )
    connection.execute(
        """
        INSERT INTO accounting_journals (
            id, company_id, code, name_ar, name_fr, journal_type,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('journal-sales', 'company-1', 'J-SALES', 'يومية المبيعات', 'Journal des ventes', 'SALES',
                  ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO journal_entries (
            id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
            source_document_id, entry_number, entry_date, source_event_type, source_event_id,
            idempotency_key, memo, created_at, created_by, updated_at, updated_by
        ) VALUES ('entry-balanced', 'company-1', 'fy-2026', 'period-open', 'journal-sales',
                  'sales-invoice-1', 'JE-000001', ?, 'SALES_INVOICE_POSTED', 'sales-invoice-1',
                  'posting:sales-invoice-1', 'Balanced fixture entry', ?, 'fixture-user', ?, 'fixture-user')
        """,
        (TODAY, NOW, NOW),
    )
    connection.executemany(
        """
        INSERT INTO journal_entry_lines (
            id, company_id, journal_entry_id, account_id, line_number, description,
            debit_minor, credit_minor, created_at, created_by
        ) VALUES (?, 'company-1', 'entry-balanced', ?, ?, ?, ?, ?, ?, 'fixture-user')
        """,
        [
            ("entry-line-debit", "account-debit", 1, "Fixture debit", 125000, 0, NOW),
            ("entry-line-credit", "account-credit", 2, "Fixture credit", 0, 125000, NOW),
        ],
    )
    connection.execute(
        """
        UPDATE journal_entries
        SET status='POSTED', posted_at=?, posted_by='fixture-user',
            updated_at=?, updated_by='fixture-user', row_version=row_version+1
        WHERE id='entry-balanced'
        """,
        (NOW, NOW),
    )
    evidence.record("created and posted balanced journal entry")

    connection.execute(
        """
        INSERT INTO audit_logs (
            id, company_id, action_code, entity_type, entity_id, occurred_at, outcome, details_json
        ) VALUES ('audit-1', 'company-1', 'sales_invoice.post', 'commercial_document',
                  'sales-invoice-1', ?, 'SUCCESS', '{"fixture":true}')
        """,
        (NOW,),
    )
    connection.execute(
        """
        INSERT INTO idempotency_keys (
            id, company_id, namespace, idempotency_key, request_hash_sha256, status,
            result_entity_type, result_entity_id, created_at, completed_at
        ) VALUES ('idem-1', 'company-1', 'posting', 'sales-invoice-1', ?, 'SUCCEEDED',
                  'journal_entry', 'entry-balanced', ?, ?)
        """,
        ("a" * 64, NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO document_templates (
            id, company_id, code, document_type, name_ar, name_fr,
            created_at, created_by, updated_at, updated_by
        ) VALUES ('template-invoice', 'company-1', 'SALES-INVOICE-A4', 'SALES_INVOICE',
                  'فاتورة A4', 'Facture A4', ?, 'fixture-user', ?, 'fixture-user')
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO document_template_versions (
            id, company_id, document_template_id, version_number, html_template, css_template,
            content_hash_sha256, is_published, created_at, created_by
        ) VALUES ('template-version-1', 'company-1', 'template-invoice', 1,
                  '<main>{{document_number}}</main>', 'main { font-size: 12pt; }', ?, 1, ?, 'fixture-user')
        """,
        ("b" * 64, NOW),
    )
    connection.execute(
        """
        INSERT INTO rendered_documents (
            id, company_id, source_document_id, template_version_id, file_format,
            relative_file_path, content_hash_sha256, rendered_at, rendered_by
        ) VALUES ('rendered-1', 'company-1', 'sales-invoice-1', 'template-version-1', 'PDF',
                  'documents/2026/SI-000001.pdf', ?, ?, 'fixture-user')
        """,
        ("c" * 64, NOW),
    )
    connection.commit()


def expect_rejected(
    connection: sqlite3.Connection,
    evidence: Evidence,
    name: str,
    action: Callable[[], None],
    expected_message: str | None = None,
) -> None:
    savepoint = f"negative_{len(evidence.passed)}"
    connection.execute(f"SAVEPOINT {savepoint}")
    try:
        action()
    except sqlite3.IntegrityError as error:
        message = str(error)
        if expected_message and expected_message not in message:
            raise VerificationError(
                f"{name}: rejected for unexpected reason: {message!r}; expected {expected_message!r}"
            ) from error
        connection.execute(f"ROLLBACK TO {savepoint}")
        connection.execute(f"RELEASE {savepoint}")
        evidence.record(name)
        return
    except Exception:
        connection.execute(f"ROLLBACK TO {savepoint}")
        connection.execute(f"RELEASE {savepoint}")
        raise
    connection.execute(f"ROLLBACK TO {savepoint}")
    connection.execute(f"RELEASE {savepoint}")
    raise VerificationError(f"{name}: invalid write unexpectedly succeeded")


def run_legacy_negative_tests(connection: sqlite3.Connection, evidence: Evidence) -> None:
    expect_rejected(
        connection,
        evidence,
        "null business identifier rejected",
        lambda: connection.execute(
            "INSERT INTO companies (id, code, legal_name, name_ar, created_at, updated_at) "
            "VALUES (NULL, 'NULL-ID', 'Null Identifier', 'معرف فارغ', ?, ?)",
            (NOW, NOW),
        ),
        "NOT NULL constraint failed: companies.id",
    )
    expect_rejected(
        connection,
        evidence,
        "blank business identifier rejected",
        lambda: connection.execute(
            "INSERT INTO companies (id, code, legal_name, name_ar, created_at, updated_at) "
            "VALUES ('   ', 'BLANK-ID', 'Blank Identifier', 'معرف فارغ', ?, ?)",
            (NOW, NOW),
        ),
        "CHECK constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "foreign key violation rejected",
        lambda: connection.execute(
            "INSERT INTO units (id, company_id, code, name_ar, name_fr, created_at, updated_at) "
            "VALUES ('bad-unit', 'missing-company', 'BAD', 'سيئ', 'Mauvais', ?, ?)",
            (NOW, NOW),
        ),
        "FOREIGN KEY constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "duplicate scoped product code rejected",
        lambda: connection.execute(
            """
            INSERT INTO products (
                id, company_id, unit_id, code, name_ar, product_kind, stock_tracked,
                created_at, updated_at
            ) VALUES ('product-duplicate', 'company-1', 'unit-piece', 'P-001', 'مكرر',
                      'STOCK_ITEM', 1, ?, ?)
            """,
            (NOW, NOW),
        ),
        "UNIQUE constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "partner without customer or supplier role rejected",
        lambda: connection.execute(
            """
            INSERT INTO partners (
                id, company_id, code, legal_name, display_name_ar, is_customer, is_supplier,
                created_at, updated_at
            ) VALUES ('partner-invalid', 'company-1', 'X-001', 'Invalid', 'غير صالح', 0, 0, ?, ?)
            """,
            (NOW, NOW),
        ),
        "CHECK constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "duplicate scoped human document number rejected",
        lambda: insert_document(
            connection,
            document_id="duplicate-document-number",
            document_type="SALES_INVOICE",
            document_number="SI-000001",
            workflow_status="DRAFT",
            partner_id="customer-1",
            idempotency_key="document:duplicate-number",
        ),
        "UNIQUE constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "invalid document workflow status rejected",
        lambda: insert_document(
            connection,
            document_id="invalid-status-document",
            document_type="SALES_ORDER",
            document_number="SO-INVALID",
            workflow_status="PAID",
            partner_id="customer-1",
            idempotency_key="document:invalid-status",
        ),
        "CHECK constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "document line zero quantity rejected",
        lambda: insert_line(
            connection,
            line_id="line-zero-quantity",
            document_id="sales-order-1",
            line_number=99,
            quantity_scaled=0,
        ),
        "CHECK constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "self-linking document line rejected",
        lambda: connection.execute(
            """
            INSERT INTO document_line_links (
                id, company_id, source_line_id, target_line_id, transformation_type,
                transformed_quantity_scaled, created_at, created_by
            ) VALUES ('link-self', 'company-1', 'line-sales-order-1', 'line-sales-order-1',
                      'ORDER_TO_DELIVERY', 1000000, ?, 'fixture-user')
            """,
            (NOW,),
        ),
        "CHECK constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "zero transformed quantity rejected",
        lambda: connection.execute(
            """
            INSERT INTO document_line_links (
                id, company_id, source_line_id, target_line_id, transformation_type,
                transformed_quantity_scaled, created_at, created_by
            ) VALUES ('link-zero', 'company-1', 'line-delivery-1', 'line-sales-order-1',
                      'DOCUMENT_TO_RETURN', 0, ?, 'fixture-user')
            """,
            (NOW,),
        ),
        "CHECK constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "duplicate document idempotency key rejected",
        lambda: insert_document(
            connection,
            document_id="duplicate-idempotency-document",
            document_type="SALES_ORDER",
            document_number="SO-000002",
            workflow_status="DRAFT",
            partner_id="customer-1",
            idempotency_key="document:sales-order-1",
        ),
        "UNIQUE constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "duplicate idempotency namespace key rejected",
        lambda: connection.execute(
            """
            INSERT INTO idempotency_keys (
                id, company_id, namespace, idempotency_key, request_hash_sha256,
                status, created_at, completed_at
            ) VALUES ('idem-duplicate', 'company-1', 'posting', 'sales-invoice-1', ?,
                      'SUCCEEDED', ?, ?)
            """,
            ("d" * 64, NOW, NOW),
        ),
        "UNIQUE constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "duplicate stock posting event rejected",
        lambda: connection.execute(
            """
            INSERT INTO stock_movements (
                id, company_id, product_id, warehouse_id, movement_type, business_date, occurred_at,
                quantity_delta_scaled, quantity_before_scaled, quantity_after_scaled,
                posting_event_key
            ) VALUES ('movement-duplicate', 'company-1', 'product-1', 'warehouse-main', 'ADJUSTMENT_IN',
                      ?, ?, 1000000, 10000000, 11000000,
                      'opening-stock:opening-doc:opening-line')
            """,
            (TODAY, NOW),
        ),
        "UNIQUE constraint failed",
    )
    expect_rejected(
        connection,
        evidence,
        "stock balance equation rejected",
        lambda: connection.execute(
            """
            INSERT INTO stock_balances (
                id, company_id, product_id, warehouse_id, on_hand_scaled, reserved_scaled,
                available_scaled, average_cost_scaled, rebuilt_at
            ) VALUES ('balance-invalid', 'company-1', 'product-1', 'warehouse-main',
                      10, 2, 99, 0, ?)
            """,
            (NOW,),
        ),
        "CHECK constraint failed",
    )

    immutability_cases: list[tuple[str, Callable[[], None], str]] = [
        (
            "posted commercial document update rejected",
            lambda: connection.execute(
                "UPDATE commercial_documents SET notes='changed' WHERE id='sales-invoice-1'"
            ),
            "posted commercial document is immutable",
        ),
        (
            "posted commercial document delete rejected",
            lambda: connection.execute(
                "DELETE FROM commercial_documents WHERE id='sales-invoice-1'"
            ),
            "posted commercial document cannot be deleted",
        ),
        (
            "posted commercial line insert rejected",
            lambda: insert_line(
                connection,
                line_id="posted-line-insert",
                document_id="sales-invoice-1",
                line_number=99,
                quantity_scaled=1000000,
            ),
            "cannot add a line to a posted commercial document",
        ),
        (
            "posted commercial line update rejected",
            lambda: connection.execute(
                "UPDATE commercial_document_lines SET notes='changed' WHERE id='line-invoice-1'"
            ),
            "posted commercial document line is immutable",
        ),
        (
            "posted commercial line delete rejected",
            lambda: connection.execute(
                "DELETE FROM commercial_document_lines WHERE id='line-invoice-1'"
            ),
            "posted commercial document line cannot be deleted",
        ),
        (
            "posted lineage insert rejected",
            lambda: connection.execute(
                """
                INSERT INTO document_line_links (
                    id, company_id, source_line_id, target_line_id, transformation_type,
                    transformed_quantity_scaled, created_at, created_by
                ) VALUES ('posted-link-insert', 'company-1', 'line-sales-order-1', 'line-invoice-1',
                          'ORDER_TO_INVOICE', 1000000, ?, 'fixture-user')
                """,
                (NOW,),
            ),
            "cannot add lineage to a posted target commercial document",
        ),
        (
            "posted lineage update rejected",
            lambda: connection.execute(
                "UPDATE document_line_links SET transformed_quantity_scaled=7000000 "
                "WHERE id='link-delivery-invoice-1'"
            ),
            "posted commercial document lineage is immutable",
        ),
        (
            "posted lineage delete rejected",
            lambda: connection.execute(
                "DELETE FROM document_line_links WHERE id='link-delivery-invoice-1'"
            ),
            "posted commercial document lineage cannot be deleted",
        ),
        (
            "document status history update rejected",
            lambda: connection.execute(
                "UPDATE document_status_history SET reason='changed' WHERE id='status-history-1'"
            ),
            "document status history is append-only",
        ),
        (
            "document status history delete rejected",
            lambda: connection.execute(
                "DELETE FROM document_status_history WHERE id='status-history-1'"
            ),
            "document status history is append-only",
        ),
        (
            "stock movement update rejected",
            lambda: connection.execute(
                "UPDATE stock_movements SET notes='changed' WHERE id='movement-opening'"
            ),
            "stock movements are append-only",
        ),
        (
            "stock movement delete rejected",
            lambda: connection.execute(
                "DELETE FROM stock_movements WHERE id='movement-opening'"
            ),
            "stock movements are append-only",
        ),
        (
            "audit record update rejected",
            lambda: connection.execute(
                "UPDATE audit_logs SET outcome='FAILURE' WHERE id='audit-1'"
            ),
            "audit log is append-only",
        ),
        (
            "audit record delete rejected",
            lambda: connection.execute("DELETE FROM audit_logs WHERE id='audit-1'"),
            "audit log is append-only",
        ),
        (
            "posted journal entry update rejected",
            lambda: connection.execute(
                "UPDATE journal_entries SET memo='changed' WHERE id='entry-balanced'"
            ),
            "posted journal entry is immutable",
        ),
        (
            "posted journal entry delete rejected",
            lambda: connection.execute(
                "DELETE FROM journal_entries WHERE id='entry-balanced'"
            ),
            "posted journal entry cannot be deleted",
        ),
        (
            "posted journal line insert rejected",
            lambda: connection.execute(
                """
                INSERT INTO journal_entry_lines (
                    id, company_id, journal_entry_id, account_id, line_number, description,
                    debit_minor, credit_minor, created_at
                ) VALUES ('posted-journal-line-insert', 'company-1', 'entry-balanced',
                          'account-debit', 99, 'Late line', 1, 0, ?)
                """,
                (NOW,),
            ),
            "cannot add a line to a posted journal entry",
        ),
        (
            "posted journal line update rejected",
            lambda: connection.execute(
                "UPDATE journal_entry_lines SET description='changed' WHERE id='entry-line-debit'"
            ),
            "posted journal entry line is immutable",
        ),
        (
            "posted journal line delete rejected",
            lambda: connection.execute(
                "DELETE FROM journal_entry_lines WHERE id='entry-line-debit'"
            ),
            "posted journal entry line cannot be deleted",
        ),
        (
            "document template version update rejected",
            lambda: connection.execute(
                "UPDATE document_template_versions SET is_published=0 WHERE id='template-version-1'"
            ),
            "document template versions are immutable",
        ),
        (
            "document template version delete rejected",
            lambda: connection.execute(
                "DELETE FROM document_template_versions WHERE id='template-version-1'"
            ),
            "document template versions are immutable",
        ),
        (
            "rendered document update rejected",
            lambda: connection.execute(
                "UPDATE rendered_documents SET relative_file_path='changed.pdf' WHERE id='rendered-1'"
            ),
            "rendered document history is immutable",
        ),
        (
            "rendered document delete rejected",
            lambda: connection.execute(
                "DELETE FROM rendered_documents WHERE id='rendered-1'"
            ),
            "rendered document history is immutable",
        ),
    ]
    for name, action, message in immutability_cases:
        expect_rejected(connection, evidence, name, action, message)

    insert_document(
        connection,
        document_id="reparent-source-document",
        document_type="SALES_ORDER",
        document_number="SO-REPARENT",
        workflow_status="DRAFT",
        partner_id="customer-1",
        idempotency_key="document:reparent-source",
    )
    insert_line(
        connection,
        line_id="line-reparent-draft",
        document_id="reparent-source-document",
        line_number=99,
        quantity_scaled=1000000,
    )
    posted_document_before = connection.execute(
        """
        SELECT
            (SELECT COUNT(*) FROM commercial_document_lines WHERE document_id=document.id),
            header_discount_minor, total_ht_minor, total_tax_minor, total_ttc_minor
        FROM commercial_documents AS document WHERE id='sales-invoice-1'
        """
    ).fetchone()
    expect_rejected(
        connection,
        evidence,
        "commercial line reparenting into posted document rejected",
        lambda: connection.execute(
            "UPDATE commercial_document_lines SET document_id='sales-invoice-1' "
            "WHERE id='line-reparent-draft'"
        ),
        "posted commercial document line is immutable",
    )
    posted_document_after = connection.execute(
        """
        SELECT
            (SELECT COUNT(*) FROM commercial_document_lines WHERE document_id=document.id),
            header_discount_minor, total_ht_minor, total_tax_minor, total_ttc_minor
        FROM commercial_documents AS document WHERE id='sales-invoice-1'
        """
    ).fetchone()
    draft_line_parent = connection.execute(
        "SELECT document_id FROM commercial_document_lines WHERE id='line-reparent-draft'"
    ).fetchone()
    evidence.require(
        posted_document_after == posted_document_before
        and draft_line_parent == ("reparent-source-document",),
        "commercial reparenting left posted document line count and totals unchanged",
    )

    expect_rejected(
        connection,
        evidence,
        "journal entry direct posted insert rejected",
        lambda: connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, status, source_event_type, source_event_id,
                idempotency_key, created_at, updated_at
            ) VALUES ('entry-direct-posted', 'company-1', 'fy-2026', 'period-open', 'journal-sales',
                      'JE-DIRECT', ?, 'POSTED', 'TEST', 'direct', 'posting:direct', ?, ?)
            """,
            (TODAY, NOW, NOW),
        ),
        "must be inserted as DRAFT",
    )

    def post_unbalanced() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id,
                idempotency_key, created_at, updated_at
            ) VALUES ('entry-unbalanced', 'company-1', 'fy-2026', 'period-open', 'journal-sales',
                      'JE-UNBALANCED', ?, 'TEST', 'unbalanced', 'posting:unbalanced', ?, ?)
            """,
            (TODAY, NOW, NOW),
        )
        connection.executemany(
            """
            INSERT INTO journal_entry_lines (
                id, company_id, journal_entry_id, account_id, line_number, description,
                debit_minor, credit_minor, created_at
            ) VALUES (?, 'company-1', 'entry-unbalanced', ?, ?, 'Unbalanced test', ?, ?, ?)
            """,
            [
                ("unbalanced-debit", "account-debit", 1, 100, 0, NOW),
                ("unbalanced-credit", "account-credit", 2, 0, 99, NOW),
            ],
        )
        connection.execute(
            "UPDATE journal_entries SET status='POSTED' WHERE id='entry-unbalanced'"
        )

    expect_rejected(
        connection,
        evidence,
        "unbalanced journal entry posting rejected",
        post_unbalanced,
        "not balanced",
    )

    def post_without_two_lines() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id,
                idempotency_key, created_at, updated_at
            ) VALUES ('entry-one-line', 'company-1', 'fy-2026', 'period-open', 'journal-sales',
                      'JE-ONE-LINE', ?, 'TEST', 'one-line', 'posting:one-line', ?, ?)
            """,
            (TODAY, NOW, NOW),
        )
        connection.execute(
            """
            INSERT INTO journal_entry_lines (
                id, company_id, journal_entry_id, account_id, line_number, description,
                debit_minor, credit_minor, created_at
            ) VALUES ('one-line-debit', 'company-1', 'entry-one-line', 'account-debit', 1,
                      'One line test', 100, 0, ?)
            """,
            (NOW,),
        )
        connection.execute(
            "UPDATE journal_entries SET status='POSTED' WHERE id='entry-one-line'"
        )

    expect_rejected(
        connection,
        evidence,
        "journal entry with fewer than two lines rejected",
        post_without_two_lines,
        "at least two lines",
    )

    def insert_both_sides_on_draft() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id,
                idempotency_key, created_at, updated_at
            ) VALUES ('entry-line-checks', 'company-1', 'fy-2026', 'period-open', 'journal-sales',
                      'JE-LINE-CHECKS', ?, 'TEST', 'line-checks', 'posting:line-checks', ?, ?)
            """,
            (TODAY, NOW, NOW),
        )
        connection.execute(
            """
            INSERT INTO journal_entry_lines (
                id, company_id, journal_entry_id, account_id, line_number, description,
                debit_minor, credit_minor, created_at
            ) VALUES ('line-both-draft', 'company-1', 'entry-line-checks', 'account-debit', 1,
                      'Both sides draft', 100, 100, ?)
            """,
            (NOW,),
        )

    expect_rejected(
        connection,
        evidence,
        "journal line cannot have positive debit and credit",
        insert_both_sides_on_draft,
        "CHECK constraint failed",
    )

    def insert_neither_side_on_draft() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id,
                idempotency_key, created_at, updated_at
            ) VALUES ('entry-neither-check', 'company-1', 'fy-2026', 'period-open', 'journal-sales',
                      'JE-NEITHER', ?, 'TEST', 'neither', 'posting:neither', ?, ?)
            """,
            (TODAY, NOW, NOW),
        )
        connection.execute(
            """
            INSERT INTO journal_entry_lines (
                id, company_id, journal_entry_id, account_id, line_number, description,
                debit_minor, credit_minor, created_at
            ) VALUES ('line-neither-draft', 'company-1', 'entry-neither-check', 'account-debit', 1,
                      'Neither side', 0, 0, ?)
            """,
            (NOW,),
        )

    expect_rejected(
        connection,
        evidence,
        "journal line cannot have neither debit nor credit",
        insert_neither_side_on_draft,
        "CHECK constraint failed",
    )

    def post_into_closed_period() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id,
                idempotency_key, created_at, updated_at
            ) VALUES ('entry-closed-period', 'company-1', 'fy-2026', 'period-closed', 'journal-sales',
                      'JE-CLOSED', '2026-02-15', 'TEST', 'closed-period', 'posting:closed-period', ?, ?)
            """,
            (NOW, NOW),
        )
        connection.executemany(
            """
            INSERT INTO journal_entry_lines (
                id, company_id, journal_entry_id, account_id, line_number, description,
                debit_minor, credit_minor, created_at
            ) VALUES (?, 'company-1', 'entry-closed-period', ?, ?, 'Closed period test', ?, ?, ?)
            """,
            [
                ("closed-debit", "account-debit", 1, 100, 0, NOW),
                ("closed-credit", "account-credit", 2, 0, 100, NOW),
            ],
        )
        connection.execute(
            "UPDATE journal_entries SET status='POSTED' WHERE id='entry-closed-period'"
        )

    expect_rejected(
        connection,
        evidence,
        "closed fiscal period posting rejected",
        post_into_closed_period,
        "open fiscal period",
    )

    connection.execute(
        """
        INSERT INTO journal_entries (
            id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
            entry_number, entry_date, source_event_type, source_event_id,
            idempotency_key, created_at, updated_at
        ) VALUES ('entry-reparent-draft', 'company-1', 'fy-2026', 'period-open', 'journal-sales',
                  'JE-REPARENT', ?, 'TEST', 'reparent', 'posting:reparent', ?, ?)
        """,
        (TODAY, NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO journal_entry_lines (
            id, company_id, journal_entry_id, account_id, line_number, description,
            debit_minor, credit_minor, created_at
        ) VALUES ('entry-line-reparent-draft', 'company-1', 'entry-reparent-draft',
                  'account-debit', 99, 'Reparent test', 1, 0, ?)
        """,
        (NOW,),
    )
    posted_entry_before = connection.execute(
        "SELECT COUNT(*), COALESCE(SUM(debit_minor),0), COALESCE(SUM(credit_minor),0) "
        "FROM journal_entry_lines WHERE journal_entry_id='entry-balanced'"
    ).fetchone()
    expect_rejected(
        connection,
        evidence,
        "journal line reparenting into posted entry rejected",
        lambda: connection.execute(
            "UPDATE journal_entry_lines SET journal_entry_id='entry-balanced' "
            "WHERE id='entry-line-reparent-draft'"
        ),
        "posted journal entry line is immutable",
    )
    posted_entry_after = connection.execute(
        "SELECT COUNT(*), COALESCE(SUM(debit_minor),0), COALESCE(SUM(credit_minor),0) "
        "FROM journal_entry_lines WHERE journal_entry_id='entry-balanced'"
    ).fetchone()
    draft_journal_parent = connection.execute(
        "SELECT journal_entry_id FROM journal_entry_lines WHERE id='entry-line-reparent-draft'"
    ).fetchone()
    evidence.require(
        posted_entry_after == posted_entry_before
        and posted_entry_after[1] == posted_entry_after[2]
        and draft_journal_parent == ("entry-reparent-draft",),
        "journal reparenting left posted entry line count and balance unchanged",
    )

    connection.execute("SAVEPOINT pending_over_conversion")
    try:
        insert_document(
            connection,
            document_id="delivery-over",
            document_type="DELIVERY_NOTE",
            document_number="DN-OVER",
            workflow_status="DRAFT",
            partner_id="customer-1",
            idempotency_key="document:delivery-over",
        )
        insert_line(
            connection,
            line_id="line-delivery-over",
            document_id="delivery-over",
            line_number=1,
            quantity_scaled=1000000,
        )
        connection.execute(
            """
            INSERT INTO document_line_links (
                id, company_id, source_line_id, target_line_id, transformation_type,
                transformed_quantity_scaled, created_at, created_by
            ) VALUES ('link-over-conversion', 'company-1', 'line-sales-order-1', 'line-delivery-over',
                      'ORDER_TO_DELIVERY', 1000000, ?, 'fixture-user')
            """,
            (NOW,),
        )
        detected = connection.execute(
            """
            SELECT COALESCE(SUM(link.transformed_quantity_scaled),0) > source.quantity_scaled
            FROM commercial_document_lines AS source
            JOIN document_line_links AS link ON link.source_line_id=source.id
            WHERE source.id='line-sales-order-1'
            """
        ).fetchone()[0]
        if detected != 1:
            raise VerificationError(
                "pending over-conversion detector did not detect aggregate excess"
            )
        evidence.pending.append(
            "application-service invariant: aggregate transformed quantity must not exceed source quantity"
        )
    finally:
        connection.execute("ROLLBACK TO pending_over_conversion")
        connection.execute("RELEASE pending_over_conversion")

    connection.commit()


def run_phase05_constraint_tests(connection: sqlite3.Connection, evidence: Evidence) -> None:
    connection.execute(
        """
        INSERT INTO companies (
            id, code, legal_name, name_ar, created_at, updated_at,
            activity_description, legal_form, social_capital_minor,
            statistical_identifier, tax_article_number, bank_rib,
            wilaya_code, city, postal_code
        ) VALUES ('company-phase05', 'PHASE05', 'Phase 05 Company', 'شركة المرحلة الخامسة',
                  ?, ?, 'Commerce', 'SARL', 1000000, 'NIS-001', 'AI-001',
                  '00799999000000000000', '16', 'Alger', '16000')
        """,
        (NOW, NOW),
    )
    evidence.record("PHASE 05 company legal and location fields accept valid fixed-point data")

    connection.execute(
        """
        INSERT INTO company_settings (
            id, company_id, default_margin_rate_scaled,
            session_idle_timeout_minutes, created_at, updated_at
        ) VALUES ('settings-phase05', 'company-phase05', 200000, 15, ?, ?)
        """,
        (NOW, NOW),
    )
    defaults = connection.execute(
        """
        SELECT default_margin_rate_scaled, below_cost_policy, session_idle_timeout_minutes, default_tax_rate_id
        FROM company_settings WHERE id='settings-phase05'
        """
    ).fetchone()
    evidence.require(
        defaults == (200000, "ADMIN_OVERRIDE", 15, None),
        "PHASE 05 company setting defaults and fixed-point fields are correct",
    )

    phase05_rejections: list[tuple[str, str, tuple, str | None]] = [
        (
            "company social capital rejects negative fixed-point value",
            "UPDATE companies SET social_capital_minor=-1 WHERE id=?",
            ("company-phase05",),
            "CHECK constraint failed",
        ),
        (
            "company wilaya rejects out-of-range code",
            "UPDATE companies SET wilaya_code='59' WHERE id=?",
            ("company-phase05",),
            "CHECK constraint failed",
        ),
        (
            "company postal code rejects non-digits",
            "UPDATE companies SET postal_code='16A00' WHERE id=?",
            ("company-phase05",),
            "CHECK constraint failed",
        ),
        (
            "default margin rejects value above 100 percent",
            "UPDATE company_settings SET default_margin_rate_scaled=1000001 WHERE id=?",
            ("settings-phase05",),
            "CHECK constraint failed",
        ),
        (
            "session timeout rejects value below five minutes",
            "UPDATE company_settings SET session_idle_timeout_minutes=4 WHERE id=?",
            ("settings-phase05",),
            "CHECK constraint failed",
        ),
        (
            "session timeout rejects value above 120 minutes",
            "UPDATE company_settings SET session_idle_timeout_minutes=121 WHERE id=?",
            ("settings-phase05",),
            "CHECK constraint failed",
        ),
        (
            "below-cost policy rejects unknown values",
            "UPDATE company_settings SET below_cost_policy='SILENT_ALLOW' WHERE id=?",
            ("settings-phase05",),
            "CHECK constraint failed",
        ),
        (
            "default tax rate enforces foreign key",
            "UPDATE company_settings SET default_tax_rate_id='missing-tax' WHERE id=?",
            ("settings-phase05",),
            "FOREIGN KEY constraint failed",
        ),
        (
            "setup draft requires JSON object",
            "INSERT INTO setup_drafts (id, draft_schema_version, validated_json, created_at, updated_at) "
            "VALUES (?, 1, '[]', ?, ?)",
            ("draft-array", NOW, NOW),
            "CHECK constraint failed",
        ),
        (
            "setup draft rejects invalid JSON",
            "INSERT INTO setup_drafts (id, draft_schema_version, validated_json, created_at, updated_at) "
            "VALUES (?, 1, '{invalid', ?, ?)",
            ("draft-invalid", NOW, NOW),
            "CHECK constraint failed",
        ),
        (
            "initial setup request rejects short idempotency key",
            "INSERT INTO initial_setup_requests "
            "(id, idempotency_key, request_hash_sha256, status, created_at) "
            "VALUES (?, 'short', ?, 'IN_PROGRESS', ?)",
            ("request-short", "a" * 64, NOW),
            "CHECK constraint failed",
        ),
        (
            "initial setup request rejects non-hex request hash",
            "INSERT INTO initial_setup_requests "
            "(id, idempotency_key, request_hash_sha256, status, created_at) "
            "VALUES (?, 'request-nonhex', ?, 'IN_PROGRESS', ?)",
            ("request-nonhex", "z" * 64, NOW),
            "CHECK constraint failed",
        ),
        (
            "initial setup success requires result and completion timestamp",
            "INSERT INTO initial_setup_requests "
            "(id, idempotency_key, request_hash_sha256, status, created_at) "
            "VALUES (?, 'request-success-invalid', ?, 'SUCCEEDED', ?)",
            ("request-success-invalid", "b" * 64, NOW),
            "CHECK constraint failed",
        ),
        (
            "recovery code hash rejects non-hex material",
            "INSERT INTO user_recovery_codes "
            "(id, company_id, user_id, code_hash, created_at) "
            "VALUES (?, 'company-phase05', 'user-phase05', ?, ?)",
            ("recovery-nonhex", "z" * 64, NOW),
            "CHECK constraint failed",
        ),
    ]

    connection.execute(
        """
        INSERT INTO users (
            id, company_id, username, display_name, password_hash, created_at, updated_at
        ) VALUES ('user-phase05', 'company-phase05', 'Admin', 'Administrator',
                  '$argon2id$v=19$m=19456,t=2,p=1$fixture$fixturehashvalue', ?, ?)
        """,
        (NOW, NOW),
    )
    evidence.record("valid PHASE 05 local user fixture created")

    for name, statement, values, message in phase05_rejections:
        expect_rejected(
            connection,
            evidence,
            name,
            lambda statement=statement, values=values: connection.execute(statement, values),
            message,
        )

    expect_rejected(
        connection,
        evidence,
        "normalized case-insensitive username uniqueness enforced",
        lambda: connection.execute(
            """
            INSERT INTO users (
                id, company_id, username, display_name, password_hash, created_at, updated_at
            ) VALUES ('user-phase05-duplicate', 'company-phase05', '  admin  ', 'Duplicate',
                      '$argon2id$v=19$m=19456,t=2,p=1$fixture$fixturehashvalue', ?, ?)
            """,
            (NOW, NOW),
        ),
        "UNIQUE constraint failed",
    )

    connection.execute(
        """
        INSERT INTO setup_drafts (
            id, draft_schema_version, validated_json, created_at, updated_at
        ) VALUES ('draft-phase05', 1, '{"companyCode":"PHASE05","language":"ar"}', ?, ?)
        """,
        (NOW, NOW),
    )
    evidence.record("typed non-secret setup draft JSON persisted")
    expect_rejected(
        connection,
        evidence,
        "only one active setup draft is allowed",
        lambda: connection.execute(
            """
            INSERT INTO setup_drafts (
                id, draft_schema_version, validated_json, created_at, updated_at
            ) VALUES ('draft-phase05-second', 1, '{}', ?, ?)
            """,
            (NOW, NOW),
        ),
        "UNIQUE constraint failed",
    )

    connection.execute(
        """
        INSERT INTO initial_setup_requests (
            id, idempotency_key, request_hash_sha256, status, created_at
        ) VALUES ('request-phase05', 'setup-request-0001', ?, 'IN_PROGRESS', ?)
        """,
        ("d" * 64, NOW),
    )
    evidence.record("initial setup idempotency request ledger accepts valid in-progress request")
    expect_rejected(
        connection,
        evidence,
        "initial setup idempotency key is unique",
        lambda: connection.execute(
            """
            INSERT INTO initial_setup_requests (
                id, idempotency_key, request_hash_sha256, status, created_at
            ) VALUES ('request-phase05-duplicate', 'setup-request-0001', ?, 'IN_PROGRESS', ?)
            """,
            ("e" * 64, NOW),
        ),
        "UNIQUE constraint failed",
    )

    connection.execute(
        """
        INSERT INTO user_recovery_codes (
            id, company_id, user_id, code_hash, created_at, created_by
        ) VALUES ('recovery-phase05', 'company-phase05', 'user-phase05', ?, ?, 'user-phase05')
        """,
        ("f" * 64, NOW),
    )
    evidence.record("one active hashed recovery code persisted")
    expect_rejected(
        connection,
        evidence,
        "one active recovery code per user is enforced",
        lambda: connection.execute(
            """
            INSERT INTO user_recovery_codes (
                id, company_id, user_id, code_hash, created_at, created_by
            ) VALUES ('recovery-phase05-second', 'company-phase05', 'user-phase05', ?, ?, 'user-phase05')
            """,
            ("1" * 64, NOW),
        ),
        "UNIQUE constraint failed",
    )
    connection.execute(
        "UPDATE user_recovery_codes SET used_at=? WHERE id='recovery-phase05'",
        (NOW,),
    )
    connection.execute(
        """
        INSERT INTO user_recovery_codes (
            id, company_id, user_id, code_hash, created_at, created_by
        ) VALUES ('recovery-phase05-rotated', 'company-phase05', 'user-phase05', ?, ?, 'user-phase05')
        """,
        ("2" * 64, NOW),
    )
    evidence.record("used recovery code permits a single rotated replacement")

    connection.execute(
        """
        INSERT INTO tax_rates (
            id, company_id, code, name_ar, name_fr, rate_scaled, valid_from,
            created_at, updated_at
        ) VALUES ('tax-phase05', 'company-phase05', 'TVA19', 'ضريبة 19%', 'TVA 19%', 190000,
                  '2026-01-01', ?, ?)
        """,
        (NOW, NOW),
    )
    connection.execute(
        "UPDATE company_settings SET default_tax_rate_id='tax-phase05' "
        "WHERE id='settings-phase05'"
    )
    evidence.record("company default tax rate references an active company tax fixture")

    connection.execute(
        """
        INSERT INTO fiscal_years (
            id, company_id, code, starts_on, ends_on, created_at, updated_at
        ) VALUES ('fy-phase05', 'company-phase05', '2026', '2026-01-01', '2026-12-31', ?, ?)
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO document_sequences (
            id, company_id, fiscal_year_id, document_type, prefix,
            next_number, padding_width, created_at, updated_at
        ) VALUES ('sequence-phase05', 'company-phase05', 'fy-phase05', 'SALES_INVOICE',
                  'FAC-2026-', 1, 6, ?, ?)
        """,
        (NOW, NOW),
    )
    evidence.record("document sequence accepts one type/year scope with six-digit padding")
    expect_rejected(
        connection,
        evidence,
        "document sequence is unique per company fiscal year and document type",
        lambda: connection.execute(
            """
            INSERT INTO document_sequences (
                id, company_id, fiscal_year_id, document_type, prefix,
                next_number, padding_width, created_at, updated_at
            ) VALUES ('sequence-phase05-second', 'company-phase05', 'fy-phase05',
                      'SALES_INVOICE', 'ALT-', 1, 6, ?, ?)
            """,
            (NOW, NOW),
        ),
        "UNIQUE constraint failed",
    )
    connection.commit()


def run_invariants_sql(connection: sqlite3.Connection, evidence: Evidence) -> None:
    connection.executescript(INVARIANTS_FILE.read_text(encoding="utf-8"))
    evidence.record("executed additive database/tests/invariants.sql")


def verify_upgrade(files: list[Path], evidence: Evidence) -> None:
    connection = sqlite3.connect(":memory:")
    connection.execute("PRAGMA foreign_keys=ON")
    connection.execute("PRAGMA busy_timeout=5000")
    apply_migrations(connection, files[:4])
    evidence.require(
        connection.execute("SELECT COUNT(*) FROM app_migrations").fetchone()[0] == 4,
        "real upgrade fixture reached accepted schema 0004",
    )
    connection.execute(
        """
        INSERT INTO companies (
            id, code, legal_name, name_ar, created_at, updated_at
        ) VALUES ('upgrade-company', 'UPGRADE', 'Upgrade Company', 'شركة الترقية', ?, ?)
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO company_settings (
            id, company_id, created_at, updated_at
        ) VALUES ('upgrade-settings', 'upgrade-company', ?, ?)
        """,
        (NOW, NOW),
    )
    connection.commit()
    apply_migrations(connection, files, start_index=4)
    evidence.require(
        connection.execute(
            "SELECT version FROM app_migrations ORDER BY id DESC LIMIT 1"
        ).fetchone()[0]
        == "0006",
        "real 0004 database upgraded through schema 0006",
    )
    evidence.require(
        connection.execute(
            "SELECT default_margin_rate_scaled, below_cost_policy, session_idle_timeout_minutes "
            "FROM company_settings WHERE id='upgrade-settings'"
        ).fetchone()
        == (0, "ADMIN_OVERRIDE", 15),
        "0004 company settings received safe PHASE 05 defaults",
    )
    violations = connection.execute("PRAGMA foreign_key_check").fetchall()
    evidence.require(
        not violations,
        "real 0004 to 0006 upgrade has no foreign-key violations",
        f"upgrade foreign-key violations: {violations}",
    )
    connection.close()


def verify_phase05_contract_sensitivity(files: list[Path], evidence: Evidence) -> None:
    migration = files[4].read_text(encoding="utf-8")
    required_fragments = {
        "default tax foreign key": "default_tax_rate_id TEXT",
        "setup draft JSON object contract": "json_type(validated_json) = 'object'",
        "setup singleton index": "uq_setup_drafts_singleton_active",
        "idempotency request hash hex contract": "request_hash_sha256 NOT GLOB '*[^0-9a-f]*'",
        "recovery hash hex contract": "code_hash NOT GLOB '*[^0-9a-f]*'",
        "active recovery uniqueness": "uq_user_recovery_codes_active",
        "normalized username uniqueness": "uq_users_company_username_normalized",
        "document sequence scope uniqueness": "uq_document_sequences_company_year_type",
        "session timeout bounds": "session_idle_timeout_minutes BETWEEN 5 AND 120",
        "margin bounds": "default_margin_rate_scaled BETWEEN 0 AND 1000000",
        "below-cost policy enum": "below_cost_policy IN ('BLOCK', 'ADMIN_OVERRIDE', 'WARNING_ONLY')",
        "no simultaneous recovery use and revoke": "CHECK (used_at IS NULL OR revoked_at IS NULL)",
    }

    def require_fragment(source: str, fragment: str) -> None:
        if fragment not in source:
            raise VerificationError(f"required PHASE 05 migration contract missing: {fragment}")

    for name, fragment in required_fragments.items():
        require_fragment(migration, fragment)
        evidence.record(f"migration contract present: {name}")
        defective = migration.replace(fragment, "", 1)
        try:
            require_fragment(defective, fragment)
        except VerificationError:
            evidence.record(f"verifier sensitivity confirmed for removed defect: {name}")
        else:
            raise VerificationError(f"verifier did not detect deliberate migration defect: {name}")


def run(write_schema: bool) -> int:
    evidence = Evidence()
    files = migration_files(evidence)
    verify_schema_snapshot(files, write_schema, evidence)
    validate_repository_documents(evidence)
    verify_phase05_contract_sensitivity(files, evidence)

    temporary_path: Path | None = None
    connection: sqlite3.Connection | None = None
    try:
        with tempfile.NamedTemporaryFile(
            prefix="posman-schema-", suffix=".sqlite", delete=False
        ) as handle:
            temporary_path = Path(handle.name)
        connection = sqlite3.connect(temporary_path)
        connection.execute("PRAGMA foreign_keys=ON")
        connection.execute("PRAGMA busy_timeout=5000")

        apply_migrations(connection, files)
        evidence.record("applied 6 ordered migrations to a fresh database")
        apply_seed_twice(connection, evidence)
        assert_core_schema(connection, evidence)
        create_positive_fixtures(connection, evidence)
        run_legacy_negative_tests(connection, evidence)
        run_phase05_constraint_tests(connection, evidence)
        run_invariants_sql(connection, evidence)

        final_violations = connection.execute("PRAGMA foreign_key_check").fetchall()
        evidence.require(
            not final_violations,
            "final foreign key check returned no rows",
            f"final foreign key violations: {final_violations}",
        )
        verify_upgrade(files, evidence)

        if len(evidence.passed) <= 67:
            raise VerificationError(
                f"additive verifier must retain more than 67 checks, found {len(evidence.passed)}"
            )
        if evidence.pending != [
            "application-service invariant: aggregate transformed quantity must not exceed source quantity"
        ]:
            raise VerificationError(
                f"pending aggregate invariant record changed unexpectedly: {evidence.pending}"
            )

        trigger_count = connection.execute(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type='trigger'"
        ).fetchone()[0]
        print("POSMAN SQLite verification: PASS")
        print(f"  migrations: {len(files)}")
        print(f"  schema version: {files[-1].name[:4]}")
        print(f"  tables: {len(EXPECTED_TABLES)}")
        print(f"  triggers: {trigger_count}")
        print(f"  passed checks: {len(evidence.passed)}")
        print(f"  pending application invariants: {len(evidence.pending)}")
        for item in evidence.passed:
            print(f"  [PASS] {item}")
        for item in evidence.pending:
            print(f"  [PENDING] {item}")
        return 0
    except (OSError, sqlite3.DatabaseError, VerificationError) as error:
        print("POSMAN SQLite verification: FAIL", file=sys.stderr)
        print(f"  {type(error).__name__}: {error}", file=sys.stderr)
        return 1
    finally:
        if connection is not None:
            connection.close()
        if temporary_path is not None:
            temporary_path.unlink(missing_ok=True)
            for suffix in ("-wal", "-shm", "-journal"):
                Path(str(temporary_path) + suffix).unlink(missing_ok=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--write-schema",
        action="store_true",
        help="regenerate database/schema.sql from ordered migrations before verification",
    )
    return parser.parse_args()


if __name__ == "__main__":
    arguments = parse_args()
    raise SystemExit(run(arguments.write_schema))
