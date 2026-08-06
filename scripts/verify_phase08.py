#!/usr/bin/env python3
"""Verify PHASE 08 accounting/payments implementation and compatibility contracts."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "ae133cea9c3b6760a5fd22b38d3169aa2f976dc6"
FROZEN_MIGRATIONS = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
}
REQUIRED_COMMANDS = {
    "install_accounting_template", "list_accounts", "create_account", "update_account",
    "list_accounting_journals", "create_accounting_journal", "update_accounting_journal",
    "list_posting_rules", "save_posting_rule", "validate_posting_configuration",
    "list_accounting_posting_queue", "post_source_event", "retry_posting_attempt",
    "list_journal_entries", "get_journal_entry", "create_manual_journal_entry",
    "update_manual_journal_entry", "post_manual_journal_entry", "reverse_journal_entry",
    "post_customer_receipt", "post_supplier_payment", "allocate_payment",
    "reverse_payment_allocation", "reverse_payment", "list_payments",
    "get_partner_statement", "get_cash_bank_register", "get_trial_balance",
    "get_general_ledger", "get_account_ledger", "get_open_receivables",
    "get_open_payables", "list_fiscal_periods", "close_fiscal_period", "reopen_fiscal_period",
}
REQUIRED_RUST_TESTS = {
    "balanced_sales_posting", "balanced_purchase_posting", "tax_lines_are_separate",
    "delivery_cogs_posting", "direct_sale_compound_posting",
    "purchase_receive_invoice_integration_posting", "sales_return_credit_compensation",
    "purchase_return_compensation", "same_idempotency_key_and_hash_replays_without_duplicate",
    "same_key_different_hash_is_rejected", "missing_posting_rule_is_actionable",
    "ambiguous_posting_rules_are_rejected", "inactive_mapped_account_is_rejected",
    "closed_fiscal_period_is_rejected", "unbalanced_generated_entry_is_rejected",
    "mid_posting_failure_rolls_back_header_and_lines",
    "business_source_stock_and_accounting_failure_roll_back_together",
    "failed_attempt_survives_without_partial_journal", "posted_journal_and_lines_are_immutable",
    "linked_balanced_reversal", "partial_payment_allocation", "full_payment_allocation",
    "over_allocation_is_rejected", "allocation_reversal_is_compensating_and_append_only",
    "company_scope_isolation",
}
E2E_SCENARIOS = {
    "ar-accounting-setup-rules-1280x800",
    "fr-sales-source-journal-trace-1280x800",
    "ar-purchase-posting-supplier-payment-1280x800",
    "fr-manual-journal-post-reversal-1024x640",
    "ar-customer-payment-partial-full-allocation-1024x640",
    "fr-missing-rule-correction-retry-1280x800",
}


def fail(message: str) -> None:
    raise SystemExit(f"PHASE08 VERIFY FAILED: {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path: str) -> str:
    candidate = ROOT / path
    if not candidate.is_file():
        fail(f"required file missing: {path}")
    return candidate.read_text(encoding="utf-8")


def main() -> int:
    migrations = sorted((ROOT / "database/migrations").glob("*.sql"))
    names = [path.name for path in migrations]
    if names[:5] != list(FROZEN_MIGRATIONS) or len(names) not in {6, 7} or not names[5].startswith("0006_"):
        fail(f"expected frozen 0001-0005 and accepted 0006, got {names}")
    if len(names) == 7 and not names[6].startswith("0007_"):
        fail(f"expected the authorized PHASE 09 migration 0007, got {names[6]}")
    for name, expected in FROZEN_MIGRATIONS.items():
        actual = digest(ROOT / "database/migrations" / name)
        if actual != expected:
            fail(f"frozen migration changed: {name}: {actual}")
    migration6 = migrations[5]
    migration6_sha = digest(migration6)

    subprocess.run([sys.executable, "scripts/verify_schema.py"], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "scripts/verify_phase06.py"], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "scripts/verify_phase07.py"], cwd=ROOT, check=True)

    migration = migration6.read_text(encoding="utf-8")
    all_migrations = "\n".join(path.read_text(encoding="utf-8") for path in migrations)
    rust = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src-tauri/src/phase08").rglob("*.rs")))
    phase06 = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src-tauri/src/phase06").rglob("*.rs")))
    command_source = read("src-tauri/src/commands/phase08.rs")
    lib_source = "\n".join(read(path) for path in ("src-tauri/src/lib.rs", "src-tauri/src/ipc_tests.rs"))
    gateway = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src/platform/tauri").glob("phase08*.ts")))
    workspace = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src/features/phase08").rglob("*.tsx")))
    copy = read("src/features/phase08/copy.ts")
    css = read("src/features/phase08/phase08.css")
    e2e = read("tests/e2e/run_phase08.py")
    integration_tests = read("tests/integration/phase08-gateway.test.ts")
    ui_tests = read("tests/ui/phase08-ui-contract.test.ts")

    missing_commands = sorted(
        name for name in REQUIRED_COMMANDS
        if name not in command_source or name not in lib_source or name not in gateway
    )
    if missing_commands:
        fail(f"missing typed command boundary: {missing_commands}")

    for fragment in (
        "accounting_account_roles", "accounting_setups", "posting_rule_lines",
        "payment_method_accounting", "fiscal_period_events", "request_hash_sha256",
        "retry_of_attempt_id", "journal_entry_id", "reversal_of_payment_id",
        "trg_posting_attempts_append_only_update", "trg_allocation_append_only_update",
        "trg_payment_history_immutable_update", "trg_journal_entries_posted_no_update",
    ):
        if fragment not in all_migrations:
            fail(f"ordered migration contract missing: {fragment}")

    for fragment in (
        "post_source_event_in_tx", "TransactionBehavior::Immediate", "record_failed_attempt_after_rollback",
        "commercial_event_plan", "document_source_event_in_tx", "finish_idempotency",
        "POSTING_RULE_MISSING", "POSTING_RULE_AMBIGUOUS", "ACCOUNT_INACTIVE",
        "FISCAL_PERIOD_CLOSED", "UNBALANCED_GENERATED_ENTRY", "INJECTED_POSTING_FAILURE",
        "checked_add", "i128", "reversal_of_entry_id", "reversal_of_payment_id",
        "payment_allocations", "OVER_ALLOCATION", "request_hash_sha256",
    ):
        if fragment not in rust and fragment not in phase06:
            fail(f"Rust accounting contract missing: {fragment}")

    test_names = set(re.findall(r"fn\s+([a-zA-Z0-9_]+)\s*\(", rust))
    missing_tests = sorted(REQUIRED_RUST_TESTS - test_names)
    if missing_tests:
        fail(f"required real SQLite tests missing: {missing_tests}")
    if "Connection::open_in_memory" not in rust or "TEST_MIGRATIONS" not in rust or "app_migrations" not in rust:
        fail("PHASE 08 tests must execute ordered migrations against real SQLite fixtures")
    if "tauri::test::get_ipc_response" not in lib_source or 'cmd: "list_accounts"' not in lib_source:
        fail("real Tauri IPC test for PHASE 08 is missing")
    if "#[ignore]" in rust:
        fail("ignored Rust tests are forbidden")

    for fragment in ("@tauri-apps/api/core", "AbortSignal", "normalizePhase08Error", "Number.isSafeInteger"):
        if fragment not in gateway:
            fail(f"typed frontend gateway contract missing: {fragment}")
    if any(token in gateway + workspace for token in ("fetch(", "XMLHttpRequest", "WebSocket(", "axios.")):
        fail("runtime network primitive appears in PHASE 08 frontend")
    if re.search(r"\b(SELECT|INSERT INTO|UPDATE\s+\w+\s+SET|DELETE FROM)\b", workspace):
        fail("SQL appears in the PHASE 08 React workspace")
    for fragment in (
        "ar-DZ", "fr-DZ", "setLocale", "formatMoney", "formatDate", "AbortController",
        "window.confirm", "busy", "retry", "prefers-reduced-motion", "overflow:auto",
    ):
        if fragment not in workspace and fragment not in copy and fragment not in css:
            fail(f"UI operational/accessibility behavior missing: {fragment}")
    if "text-overflow:ellipsis" in css.replace(" ", ""):
        fail("primary accounting text must not be ellipsized")

    missing_e2e = sorted(name for name in E2E_SCENARIOS if name not in e2e)
    if missing_e2e:
        fail(f"required E2E evidence scenarios missing: {missing_e2e}")
    for fragment in ("axe.run", "unresolvedCriticalSeriousIncomplete", "scrollWidth", "direction", "screenshot"):
        if fragment not in e2e:
            fail(f"E2E/Axe evidence contract missing: {fragment}")

    if "PHASE08_COMMANDS" not in integration_tests or "AbortController" not in integration_tests:
        fail("gateway integration coverage is incomplete")
    if "prefers-reduced-motion" not in ui_tests or "text-overflow:ellipsis" not in ui_tests:
        fail("UI policy coverage is incomplete")

    result = {
        "status": "PASS",
        "baseline": BASELINE,
        "frozenMigrationCount": len(FROZEN_MIGRATIONS),
        "schemaMigrationCount": len(migrations),
        "migration0006": migration6.name,
        "migration0006Sha256": migration6_sha,
        "typedCommands": len(REQUIRED_COMMANDS),
        "realSqliteTests": len(REQUIRED_RUST_TESTS),
        "e2eScenarios": len(E2E_SCENARIOS),
        "transactionBoundary": "source mutation + stock + accounting + success attempt + audit/idempotency in one SQLite IMMEDIATE transaction",
        "failedAttempt": "business transaction rolls back; a short independent transaction appends FAILED metadata without a partial journal",
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
