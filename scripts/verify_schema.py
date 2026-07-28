#!/usr/bin/env python3
"""Build and verify the POSMAN Phase 01 SQLite data foundation."""

from __future__ import annotations

import argparse
import hashlib
import sqlite3
import sys
import tempfile
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
NOW = "2026-07-28T10:00:00Z"
TODAY = "2026-01-15"

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
}


class VerificationError(RuntimeError):
    """Raised when a required verification condition fails."""


def migration_files() -> list[Path]:
    files = sorted(MIGRATIONS_DIR.glob("[0-9][0-9][0-9][0-9]_*.sql"))
    if not files:
        raise VerificationError("no migration files found")
    expected_versions = [f"{index:04d}" for index in range(1, len(files) + 1)]
    actual_versions = [path.name[:4] for path in files]
    if actual_versions != expected_versions:
        raise VerificationError(
            f"migration versions must be contiguous: expected {expected_versions}, got {actual_versions}"
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


def sql_literal(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


def apply_migrations(connection: sqlite3.Connection, files: list[Path]) -> None:
    for path in files:
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


def apply_seed_twice(connection: sqlite3.Connection) -> None:
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
    if first_counts != second_counts:
        raise VerificationError(
            f"reference seed is not deterministic: first={first_counts}, second={second_counts}"
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


def create_positive_fixtures(connection: sqlite3.Connection) -> None:
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
            (f"link-delivery-invoice-{suffix}", f"line-delivery-{suffix}", f"line-invoice-{suffix}", quantity, NOW),
        )
    post_document(connection, "sales-invoice-1", "POSTED")

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
        SET status = 'POSTED', posted_at = ?, posted_by = 'fixture-user',
            updated_at = ?, updated_by = 'fixture-user', row_version = row_version + 1
        WHERE id = 'entry-balanced'
        """,
        (NOW, NOW),
    )

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


def assert_core_schema(connection: sqlite3.Connection) -> None:
    if connection.execute("PRAGMA foreign_keys").fetchone()[0] != 1:
        raise VerificationError("PRAGMA foreign_keys is not enabled")

    actual_tables = {
        row[0]
        for row in connection.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
        )
    }
    if actual_tables != EXPECTED_TABLES:
        missing = sorted(EXPECTED_TABLES - actual_tables)
        unexpected = sorted(actual_tables - EXPECTED_TABLES)
        raise VerificationError(f"table mismatch: missing={missing}, unexpected={unexpected}")

    violations = connection.execute("PRAGMA foreign_key_check").fetchall()
    if violations:
        raise VerificationError(f"foreign key violations: {violations}")

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
    if real_columns:
        raise VerificationError(f"REAL columns are prohibited: {real_columns}")
    if text_primary_key_count != 48:
        raise VerificationError(
            f"expected 48 TEXT business primary keys, found {text_primary_key_count}"
        )
    if nullable_text_primary_keys:
        raise VerificationError(
            "TEXT business primary key id is nullable in built schema: "
            + ", ".join(sorted(nullable_text_primary_keys))
        )


def run_negative_tests(connection: sqlite3.Connection) -> tuple[list[str], list[str]]:
    passed: list[str] = []
    pending: list[str] = []
    savepoint_number = 0

    def expect_rejected(
        name: str,
        action: Callable[[], None],
        expected_message: str | None = None,
    ) -> None:
        nonlocal savepoint_number
        savepoint_number += 1
        savepoint = f"negative_{savepoint_number}"
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
            passed.append(name)
            return
        except Exception:
            connection.execute(f"ROLLBACK TO {savepoint}")
            connection.execute(f"RELEASE {savepoint}")
            raise
        connection.execute(f"ROLLBACK TO {savepoint}")
        connection.execute(f"RELEASE {savepoint}")
        raise VerificationError(f"{name}: invalid write unexpectedly succeeded")

    expect_rejected(
        "null business identifier rejected",
        lambda: connection.execute(
            """
            INSERT INTO companies (
                id, code, legal_name, name_ar, name_fr, created_at, updated_at
            ) VALUES (NULL, 'NULL-ID', 'Null Identifier Company', 'معرف فارغ',
                      'Identifiant nul', ?, ?)
            """,
            (NOW, NOW),
        ),
        "NOT NULL constraint failed: companies.id",
    )
    expect_rejected(
        "blank business identifier rejected",
        lambda: connection.execute(
            """
            INSERT INTO companies (
                id, code, legal_name, name_ar, name_fr, created_at, updated_at
            ) VALUES ('   ', 'BLANK-ID', 'Blank Identifier Company', 'معرف فارغ',
                      'Identifiant vide', ?, ?)
            """,
            (NOW, NOW),
        ),
        "CHECK constraint failed",
    )

    expect_rejected(
        "foreign key violation rejected",
        lambda: connection.execute(
            "INSERT INTO units (id, company_id, code, name_ar, name_fr, created_at, updated_at) "
            "VALUES ('bad-unit', 'missing-company', 'BAD', 'سيئ', 'Mauvais', ?, ?)",
            (NOW, NOW),
        ),
        "FOREIGN KEY constraint failed",
    )
    expect_rejected(
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

    expect_rejected(
        "posted commercial document update rejected",
        lambda: connection.execute(
            "UPDATE commercial_documents SET notes = 'changed' WHERE id = 'sales-invoice-1'"
        ),
        "posted commercial document is immutable",
    )
    expect_rejected(
        "posted commercial document delete rejected",
        lambda: connection.execute("DELETE FROM commercial_documents WHERE id = 'sales-invoice-1'"),
        "posted commercial document cannot be deleted",
    )
    expect_rejected(
        "posted commercial line insert rejected",
        lambda: insert_line(
            connection,
            line_id="posted-line-insert",
            document_id="sales-invoice-1",
            line_number=99,
            quantity_scaled=1000000,
        ),
        "cannot add a line to a posted commercial document",
    )
    expect_rejected(
        "posted commercial line update rejected",
        lambda: connection.execute(
            "UPDATE commercial_document_lines SET notes = 'changed' WHERE id = 'line-invoice-1'"
        ),
        "posted commercial document line is immutable",
    )

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
            (SELECT COUNT(*) FROM commercial_document_lines WHERE document_id = document.id),
            header_discount_minor, total_ht_minor, total_tax_minor, total_ttc_minor
        FROM commercial_documents AS document
        WHERE id = 'sales-invoice-1'
        """
    ).fetchone()
    expect_rejected(
        "commercial line reparenting into posted document rejected",
        lambda: connection.execute(
            "UPDATE commercial_document_lines SET document_id = 'sales-invoice-1' "
            "WHERE id = 'line-reparent-draft'"
        ),
        "posted commercial document line is immutable",
    )
    posted_document_after = connection.execute(
        """
        SELECT
            (SELECT COUNT(*) FROM commercial_document_lines WHERE document_id = document.id),
            header_discount_minor, total_ht_minor, total_tax_minor, total_ttc_minor
        FROM commercial_documents AS document
        WHERE id = 'sales-invoice-1'
        """
    ).fetchone()
    draft_line_parent = connection.execute(
        "SELECT document_id FROM commercial_document_lines WHERE id = 'line-reparent-draft'"
    ).fetchone()
    if posted_document_after != posted_document_before or draft_line_parent != (
        "reparent-source-document",
    ):
        raise VerificationError(
            "commercial-line reparenting rejection mutated posted document fixture state"
        )
    passed.append("commercial reparenting left posted document line count and totals unchanged")
    expect_rejected(
        "posted commercial line delete rejected",
        lambda: connection.execute("DELETE FROM commercial_document_lines WHERE id = 'line-invoice-1'"),
        "posted commercial document line cannot be deleted",
    )
    expect_rejected(
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
    )
    expect_rejected(
        "posted lineage update rejected",
        lambda: connection.execute(
            "UPDATE document_line_links SET transformed_quantity_scaled = 7000000 "
            "WHERE id = 'link-delivery-invoice-1'"
        ),
        "posted commercial document lineage is immutable",
    )
    expect_rejected(
        "posted lineage delete rejected",
        lambda: connection.execute(
            "DELETE FROM document_line_links WHERE id = 'link-delivery-invoice-1'"
        ),
        "posted commercial document lineage cannot be deleted",
    )
    expect_rejected(
        "document status history update rejected",
        lambda: connection.execute(
            "UPDATE document_status_history SET reason = 'changed' WHERE id = 'status-history-1'"
        ),
        "document status history is append-only",
    )
    expect_rejected(
        "document status history delete rejected",
        lambda: connection.execute("DELETE FROM document_status_history WHERE id = 'status-history-1'"),
        "document status history is append-only",
    )
    expect_rejected(
        "stock movement update rejected",
        lambda: connection.execute(
            "UPDATE stock_movements SET notes = 'changed' WHERE id = 'movement-opening'"
        ),
        "stock movements are append-only",
    )
    expect_rejected(
        "stock movement delete rejected",
        lambda: connection.execute("DELETE FROM stock_movements WHERE id = 'movement-opening'"),
        "stock movements are append-only",
    )
    expect_rejected(
        "audit record update rejected",
        lambda: connection.execute("UPDATE audit_logs SET outcome = 'FAILURE' WHERE id = 'audit-1'"),
        "audit log is append-only",
    )
    expect_rejected(
        "audit record delete rejected",
        lambda: connection.execute("DELETE FROM audit_logs WHERE id = 'audit-1'"),
        "audit log is append-only",
    )

    expect_rejected(
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
                entry_number, entry_date, source_event_type, source_event_id, idempotency_key,
                created_at, updated_at
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
            "UPDATE journal_entries SET status = 'POSTED' WHERE id = 'entry-unbalanced'"
        )

    expect_rejected("unbalanced journal entry posting rejected", post_unbalanced, "not balanced")

    def post_without_two_lines() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id, idempotency_key,
                created_at, updated_at
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
        connection.execute("UPDATE journal_entries SET status = 'POSTED' WHERE id = 'entry-one-line'")

    expect_rejected("journal entry with fewer than two lines rejected", post_without_two_lines, "at least two lines")

    expect_rejected(
        "journal line with debit and credit rejected",
        lambda: connection.execute(
            """
            INSERT INTO journal_entry_lines (
                id, company_id, journal_entry_id, account_id, line_number, description,
                debit_minor, credit_minor, created_at
            ) VALUES ('line-both-sides', 'company-1', 'entry-balanced', 'account-debit', 99,
                      'Both sides', 100, 100, ?)
            """,
            (NOW,),
        ),
        "cannot add a line to a posted journal entry",
    )

    def insert_both_sides_on_draft() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id, idempotency_key,
                created_at, updated_at
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
        "journal line cannot have positive debit and credit",
        insert_both_sides_on_draft,
        "CHECK constraint failed",
    )

    def insert_neither_side_on_draft() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id, idempotency_key,
                created_at, updated_at
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
        "journal line cannot have neither debit nor credit",
        insert_neither_side_on_draft,
        "CHECK constraint failed",
    )

    def post_into_closed_period() -> None:
        connection.execute(
            """
            INSERT INTO journal_entries (
                id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
                entry_number, entry_date, source_event_type, source_event_id, idempotency_key,
                created_at, updated_at
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
        connection.execute("UPDATE journal_entries SET status = 'POSTED' WHERE id = 'entry-closed-period'")

    expect_rejected(
        "closed fiscal period posting rejected",
        post_into_closed_period,
        "open fiscal period",
    )
    expect_rejected(
        "posted journal entry update rejected",
        lambda: connection.execute("UPDATE journal_entries SET memo = 'changed' WHERE id = 'entry-balanced'"),
        "posted journal entry is immutable",
    )
    expect_rejected(
        "posted journal entry delete rejected",
        lambda: connection.execute("DELETE FROM journal_entries WHERE id = 'entry-balanced'"),
        "posted journal entry cannot be deleted",
    )
    expect_rejected(
        "posted journal line insert rejected",
        lambda: connection.execute(
            """
            INSERT INTO journal_entry_lines (
                id, company_id, journal_entry_id, account_id, line_number, description,
                debit_minor, credit_minor, created_at
            ) VALUES ('posted-journal-line-insert', 'company-1', 'entry-balanced', 'account-debit', 99,
                      'Late line', 1, 0, ?)
            """,
            (NOW,),
        ),
        "cannot add a line to a posted journal entry",
    )
    expect_rejected(
        "posted journal line update rejected",
        lambda: connection.execute(
            "UPDATE journal_entry_lines SET description = 'changed' WHERE id = 'entry-line-debit'"
        ),
        "posted journal entry line is immutable",
    )

    connection.execute(
        """
        INSERT INTO journal_entries (
            id, company_id, fiscal_year_id, fiscal_period_id, accounting_journal_id,
            entry_number, entry_date, source_event_type, source_event_id, idempotency_key,
            created_at, updated_at
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
        """
        SELECT COUNT(*), COALESCE(SUM(debit_minor), 0), COALESCE(SUM(credit_minor), 0)
        FROM journal_entry_lines
        WHERE journal_entry_id = 'entry-balanced'
        """
    ).fetchone()
    expect_rejected(
        "journal line reparenting into posted entry rejected",
        lambda: connection.execute(
            "UPDATE journal_entry_lines SET journal_entry_id = 'entry-balanced' "
            "WHERE id = 'entry-line-reparent-draft'"
        ),
        "posted journal entry line is immutable",
    )
    posted_entry_after = connection.execute(
        """
        SELECT COUNT(*), COALESCE(SUM(debit_minor), 0), COALESCE(SUM(credit_minor), 0)
        FROM journal_entry_lines
        WHERE journal_entry_id = 'entry-balanced'
        """
    ).fetchone()
    draft_journal_parent = connection.execute(
        "SELECT journal_entry_id FROM journal_entry_lines WHERE id = 'entry-line-reparent-draft'"
    ).fetchone()
    if (
        posted_entry_after != posted_entry_before
        or posted_entry_after[1] != posted_entry_after[2]
        or draft_journal_parent != ("entry-reparent-draft",)
    ):
        raise VerificationError(
            "journal-line reparenting rejection mutated posted balanced entry fixture state"
        )
    passed.append("journal reparenting left posted entry line count and balance unchanged")
    expect_rejected(
        "posted journal line delete rejected",
        lambda: connection.execute("DELETE FROM journal_entry_lines WHERE id = 'entry-line-debit'"),
        "posted journal entry line cannot be deleted",
    )
    expect_rejected(
        "document template version update rejected",
        lambda: connection.execute(
            "UPDATE document_template_versions SET is_published = 0 WHERE id = 'template-version-1'"
        ),
        "document template versions are immutable",
    )
    expect_rejected(
        "document template version delete rejected",
        lambda: connection.execute(
            "DELETE FROM document_template_versions WHERE id = 'template-version-1'"
        ),
        "document template versions are immutable",
    )
    expect_rejected(
        "rendered document update rejected",
        lambda: connection.execute(
            "UPDATE rendered_documents SET relative_file_path = 'changed.pdf' WHERE id = 'rendered-1'"
        ),
        "rendered document history is immutable",
    )
    expect_rejected(
        "rendered document delete rejected",
        lambda: connection.execute("DELETE FROM rendered_documents WHERE id = 'rendered-1'"),
        "rendered document history is immutable",
    )

    # Aggregate conversion requires a SUM over sibling links and belongs in the future Rust service.
    # The database intentionally allows this write; the detector must identify it before commit.
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
            SELECT COALESCE(SUM(link.transformed_quantity_scaled), 0) > source.quantity_scaled
            FROM commercial_document_lines AS source
            JOIN document_line_links AS link ON link.source_line_id = source.id
            WHERE source.id = 'line-sales-order-1'
            """
        ).fetchone()[0]
        if detected != 1:
            raise VerificationError("pending over-conversion detector did not detect aggregate excess")
        pending.append(
            "application-service invariant: aggregate transformed quantity must not exceed source quantity"
        )
    finally:
        connection.execute("ROLLBACK TO pending_over_conversion")
        connection.execute("RELEASE pending_over_conversion")

    connection.commit()
    return passed, pending


def run_invariants_sql(connection: sqlite3.Connection) -> None:
    connection.executescript(INVARIANTS_FILE.read_text(encoding="utf-8"))



def validate_repository_documents() -> list[str]:
    if not BLUEPRINT_FILE.exists():
        raise VerificationError("authoritative Blueprint copy is missing")
    blueprint_hash = hashlib.sha256(BLUEPRINT_FILE.read_bytes()).hexdigest()
    if blueprint_hash != EXPECTED_BLUEPRINT_SHA256:
        raise VerificationError(
            f"Blueprint checksum mismatch: expected {EXPECTED_BLUEPRINT_SHA256}, got {blueprint_hash}"
        )

    erd = ERD_FILE.read_text(encoding="utf-8")
    mermaid_fences = erd.count("```mermaid")
    closing_fences = erd.count("```")
    if mermaid_fences < 2 or closing_fences < mermaid_fences:
        raise VerificationError("ERD Mermaid fences are structurally incomplete")
    if "erDiagram" not in erd or "flowchart" not in erd:
        raise VerificationError("ERD must include both domain erDiagram and lineage flowchart blocks")

    prohibited_names = {".env", ".env.local", ".env.production"}
    prohibited_suffixes = {".sqlite", ".sqlite3", ".db", ".pem", ".p12", ".pfx"}
    secret_markers = (
        "-----BEGIN " + "PRIVATE KEY-----",
        "gh" + "p_",
        "github" + "_pat_",
        "sk" + "-proj-",
    )
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT)
        if path.name in prohibited_names or path.suffix.lower() in prohibited_suffixes:
            raise VerificationError(f"prohibited secret/database-like file present: {relative}")
        if path.suffix.lower() in {".md", ".sql", ".py", ".yml", ".yaml", ".txt"}:
            text = path.read_text(encoding="utf-8", errors="ignore")
            for marker in secret_markers:
                if marker in text:
                    raise VerificationError(f"secret-like marker {marker!r} found in {relative}")

    return [
        "authoritative Blueprint SHA-256 matched supplied source",
        f"validated {mermaid_fences} Mermaid blocks structurally",
        "found no prohibited secret-like or database artifact files",
    ]

def verify_schema_snapshot(files: list[Path], write_schema: bool) -> None:
    expected = generated_schema_text(files)
    if write_schema:
        SCHEMA_FILE.write_text(expected, encoding="utf-8", newline="\n")
    if not SCHEMA_FILE.exists():
        raise VerificationError(
            "database/schema.sql is missing; run python scripts/verify_schema.py --write-schema"
        )
    actual = SCHEMA_FILE.read_text(encoding="utf-8")
    if actual != expected:
        raise VerificationError(
            "database/schema.sql does not match ordered migrations; "
            "run python scripts/verify_schema.py --write-schema"
        )


def run(write_schema: bool) -> int:
    files = migration_files()
    verify_schema_snapshot(files, write_schema)
    document_checks = validate_repository_documents()

    passed: list[str] = []
    pending: list[str] = []
    temporary_path: Path | None = None
    connection: sqlite3.Connection | None = None
    try:
        with tempfile.NamedTemporaryFile(prefix="posman-schema-", suffix=".sqlite", delete=False) as handle:
            temporary_path = Path(handle.name)
        connection = sqlite3.connect(temporary_path)
        connection.execute("PRAGMA foreign_keys = ON")
        connection.execute("PRAGMA busy_timeout = 5000")

        passed.extend(document_checks)

        apply_migrations(connection, files)
        passed.append(f"applied {len(files)} ordered migrations")

        apply_seed_twice(connection)
        passed.append("applied deterministic reference seed twice")

        assert_core_schema(connection)
        passed.extend(
            [
                f"found exactly {len(EXPECTED_TABLES)} expected tables",
                "PRAGMA foreign_key_check returned no rows",
                "found no application column declared as REAL",
                "found 48 explicitly non-null TEXT business primary keys",
            ]
        )

        create_positive_fixtures(connection)
        passed.extend(
            [
                "created company, fiscal year, open and closed periods",
                "created warehouse, location, family, unit, and product",
                "created customer and supplier",
                "created and posted opening-stock document and movement",
                "created 20-unit sales order",
                "created two posted deliveries for 8 and 12 units",
                "created posted invoice lines linked to delivered quantities",
                "created and posted balanced journal entry",
            ]
        )

        negative_passed, pending_invariants = run_negative_tests(connection)
        passed.extend(negative_passed)
        pending.extend(pending_invariants)

        run_invariants_sql(connection)
        passed.append("executed database/tests/invariants.sql")

        final_fk_violations = connection.execute("PRAGMA foreign_key_check").fetchall()
        if final_fk_violations:
            raise VerificationError(f"final foreign key violations: {final_fk_violations}")
        passed.append("final foreign key check returned no rows")

        print("POSMAN SQLite verification: PASS")
        print(f"  migrations: {len(files)}")
        print(f"  tables: {len(EXPECTED_TABLES)}")
        trigger_count = connection.execute("SELECT COUNT(*) FROM sqlite_schema WHERE type = 'trigger'").fetchone()[0]
        print(f"  triggers: {trigger_count}")
        print(f"  passed checks: {len(passed)}")
        print(f"  pending application invariants: {len(pending)}")
        for item in passed:
            print(f"  [PASS] {item}")
        for item in pending:
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
