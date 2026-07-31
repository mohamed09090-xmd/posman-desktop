#!/usr/bin/env python3
"""Build and verify the POSMAN SQLite schema through PHASE 05."""

from __future__ import annotations

import argparse
import hashlib
import sqlite3
import tempfile
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
MIGRATIONS_DIR = ROOT / "database" / "migrations"
SCHEMA_FILE = ROOT / "database" / "schema.sql"
INVARIANTS_FILE = ROOT / "database" / "tests" / "invariants.sql"
NOW = "2026-07-31T12:00:00Z"
ACCEPTED_MIGRATION_HASHES = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
}
EXPECTED_TABLES = 52
EXPECTED_TEXT_PRIMARY_KEYS = 51


class VerificationError(RuntimeError):
    pass


def migration_files() -> list[Path]:
    files = sorted(MIGRATIONS_DIR.glob("[0-9][0-9][0-9][0-9]_*.sql"))
    expected = [f"{number:04d}" for number in range(1, len(files) + 1)]
    actual = [path.name[:4] for path in files]
    if actual != expected or actual[-1:] != ["0005"]:
        raise VerificationError(f"expected contiguous migrations through 0005, got {actual}")
    for name, expected_hash in ACCEPTED_MIGRATION_HASHES.items():
        path = MIGRATIONS_DIR / name
        actual_hash = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual_hash != expected_hash:
            raise VerificationError(
                f"accepted migration changed: {name}: {actual_hash} != {expected_hash}"
            )
    return files


def generated_schema_text(files: Iterable[Path]) -> str:
    sections = [
        "-- GENERATED FILE: ordered migrations are authoritative.",
        "-- Regenerate with: python scripts/verify_schema.py --write-schema",
        "",
    ]
    for path in files:
        sections.extend(
            [
                f"-- BEGIN MIGRATION {path.name}",
                path.read_text(encoding="utf-8").rstrip(),
                f"-- END MIGRATION {path.name}",
                "",
            ]
        )
    return "\n".join(sections).rstrip() + "\n"


def sql_literal(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


def apply_migrations(
    connection: sqlite3.Connection, files: list[Path], start_index: int = 0
) -> None:
    for path in files[start_index:]:
        sql = path.read_text(encoding="utf-8")
        version = path.name[:4]
        checksum = hashlib.sha256(sql.encode("utf-8")).hexdigest()
        ledger = (
            "INSERT INTO app_migrations "
            "(id, version, name, checksum_sha256, applied_at) VALUES "
            f"({int(version)}, {sql_literal(version)}, "
            f"{sql_literal(path.stem[5:])}, {sql_literal(checksum)}, {sql_literal(NOW)});"
        )
        try:
            connection.executescript(f"BEGIN IMMEDIATE;\n{sql}\n{ledger}\nCOMMIT;")
        except sqlite3.DatabaseError:
            connection.rollback()
            raise


def assert_integrity(connection: sqlite3.Connection) -> int:
    checks = 0
    if connection.execute("PRAGMA foreign_keys").fetchone()[0] != 1:
        raise VerificationError("foreign keys are not enabled")
    checks += 1

    ledger = connection.execute(
        "SELECT id, version, name, checksum_sha256 FROM app_migrations ORDER BY id"
    ).fetchall()
    if [row[1] for row in ledger] != ["0001", "0002", "0003", "0004", "0005"]:
        raise VerificationError(f"unexpected migration ledger: {ledger}")
    checks += 1

    table_count = connection.execute(
        "SELECT COUNT(*) FROM sqlite_schema "
        "WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    ).fetchone()[0]
    if table_count != EXPECTED_TABLES:
        raise VerificationError(f"expected {EXPECTED_TABLES} tables, found {table_count}")
    checks += 1

    real_columns = connection.execute(
        """
        SELECT s.name, p.name
        FROM sqlite_schema AS s, pragma_table_info(s.name) AS p
        WHERE s.type='table' AND upper(trim(p.type)) LIKE '%REAL%'
        """
    ).fetchall()
    if real_columns:
        raise VerificationError(f"REAL business columns are prohibited: {real_columns}")
    checks += 1

    text_pks = connection.execute(
        """
        SELECT s.name, p.name, p."notnull"
        FROM sqlite_schema AS s, pragma_table_info(s.name) AS p
        WHERE s.type='table' AND p.name='id' AND upper(trim(p.type))='TEXT' AND p.pk > 0
        """
    ).fetchall()
    if len(text_pks) != EXPECTED_TEXT_PRIMARY_KEYS or any(row[2] != 1 for row in text_pks):
        raise VerificationError(f"invalid TEXT primary-key contract: {text_pks}")
    checks += 1

    fk_violations = connection.execute("PRAGMA foreign_key_check").fetchall()
    if fk_violations:
        raise VerificationError(f"foreign key violations: {fk_violations}")
    checks += 1

    return checks


def expect_integrity_error(connection: sqlite3.Connection, statement: str, values: tuple) -> None:
    try:
        connection.execute(statement, values)
    except sqlite3.IntegrityError:
        connection.rollback()
        return
    connection.rollback()
    raise VerificationError("expected SQLite integrity error")


def verify_phase05_constraints(connection: sqlite3.Connection) -> int:
    checks = 0
    connection.execute(
        """
        INSERT INTO companies (
            id, code, legal_name, name_ar, created_at, updated_at
        ) VALUES ('company-verify', 'VERIFY', 'Verify Company', 'شركة التحقق', ?, ?)
        """,
        (NOW, NOW),
    )
    connection.execute(
        """
        INSERT INTO company_settings (
            id, company_id, created_at, updated_at
        ) VALUES ('settings-verify', 'company-verify', ?, ?)
        """,
        (NOW, NOW),
    )
    margin, timeout = connection.execute(
        """
        SELECT default_margin_rate_scaled, session_idle_timeout_minutes
        FROM company_settings WHERE id='settings-verify'
        """
    ).fetchone()
    if (margin, timeout) != (0, 15):
        raise VerificationError("company settings PHASE 05 defaults are incorrect")
    checks += 1

    expect_integrity_error(
        connection,
        """
        UPDATE company_settings SET session_idle_timeout_minutes=4
        WHERE id=?
        """,
        ("settings-verify",),
    )
    checks += 1

    connection.execute(
        """
        INSERT INTO users (
            id, company_id, username, display_name, password_hash,
            created_at, updated_at
        ) VALUES ('user-verify', 'company-verify', 'Admin', 'Administrator',
                  '$argon2id$v=19$m=19456,t=2,p=1$fixture$fixturehashvalue', ?, ?)
        """,
        (NOW, NOW),
    )
    connection.commit()
    expect_integrity_error(
        connection,
        """
        INSERT INTO users (
            id, company_id, username, display_name, password_hash,
            created_at, updated_at
        ) VALUES (?, 'company-verify', ?, 'Duplicate',
                  '$argon2id$v=19$m=19456,t=2,p=1$fixture$fixturehashvalue', ?, ?)
        """,
        ("user-duplicate", "  admin  ", NOW, NOW),
    )
    checks += 1

    connection.execute(
        """
        INSERT INTO setup_drafts (
            id, draft_schema_version, validated_json, created_at, updated_at
        ) VALUES ('draft-1', 1, '{"companyCode":"VERIFY"}', ?, ?)
        """,
        (NOW, NOW),
    )
    connection.commit()
    expect_integrity_error(
        connection,
        """
        INSERT INTO setup_drafts (
            id, draft_schema_version, validated_json, created_at, updated_at
        ) VALUES (?, 1, '{}', ?, ?)
        """,
        ("draft-2", NOW, NOW),
    )
    checks += 1

    expect_integrity_error(
        connection,
        """
        INSERT INTO setup_drafts (
            id, draft_schema_version, validated_json, created_at, updated_at
        ) VALUES (?, 1, ?, ?, ?)
        """,
        ("draft-secret", '{"password":"not-allowed"}', NOW, NOW),
    )
    checks += 1

    connection.execute(
        """
        INSERT INTO user_recovery_codes (
            id, company_id, user_id, code_hash, created_at, created_by
        ) VALUES ('recovery-1', 'company-verify', 'user-verify',
                  'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                  ?, 'user-verify')
        """,
        (NOW,),
    )
    connection.commit()
    expect_integrity_error(
        connection,
        """
        INSERT INTO user_recovery_codes (
            id, company_id, user_id, code_hash, created_at, created_by
        ) VALUES (?, 'company-verify', 'user-verify', ?, ?, 'user-verify')
        """,
        (
            "recovery-2",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            NOW,
        ),
    )
    checks += 1

    connection.executescript(INVARIANTS_FILE.read_text(encoding="utf-8"))
    checks += 1
    return checks


def verify_upgrade(files: list[Path]) -> int:
    connection = sqlite3.connect(":memory:")
    connection.execute("PRAGMA foreign_keys=ON")
    apply_migrations(connection, files[:4])
    before = connection.execute(
        "SELECT COUNT(*) FROM app_migrations"
    ).fetchone()[0]
    if before != 4:
        raise VerificationError("0004 upgrade fixture did not reach version 0004")
    apply_migrations(connection, files, start_index=4)
    after = connection.execute(
        "SELECT version FROM app_migrations ORDER BY id DESC LIMIT 1"
    ).fetchone()[0]
    if after != "0005":
        raise VerificationError("upgrade did not reach version 0005")
    assert_integrity(connection)
    connection.close()
    return 2


def verify(write_schema: bool) -> None:
    files = migration_files()
    generated = generated_schema_text(files)
    if write_schema:
        SCHEMA_FILE.write_text(generated, encoding="utf-8", newline="\n")
    elif not SCHEMA_FILE.exists() or SCHEMA_FILE.read_text(encoding="utf-8") != generated:
        raise VerificationError(
            "database/schema.sql is stale; run "
            "python scripts/verify_schema.py --write-schema"
        )

    with tempfile.TemporaryDirectory(prefix="posman-schema-") as directory:
        path = Path(directory) / "fresh.sqlite3"
        connection = sqlite3.connect(path)
        connection.execute("PRAGMA foreign_keys=ON")
        apply_migrations(connection, files)
        checks = assert_integrity(connection)
        checks += verify_phase05_constraints(connection)
        connection.close()

    checks += verify_upgrade(files)
    print("POSMAN SQLite verification: PASS")
    print("migrations: 5")
    print(f"tables: {EXPECTED_TABLES}")
    print("schema version: 0005")
    print(f"passed checks: {checks}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-schema", action="store_true")
    arguments = parser.parse_args()
    try:
        verify(arguments.write_schema)
    except (OSError, sqlite3.DatabaseError, VerificationError) as error:
        print(f"POSMAN SQLite verification: FAIL: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
