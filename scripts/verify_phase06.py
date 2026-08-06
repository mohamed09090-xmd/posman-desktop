#!/usr/bin/env python3
"""Verify PHASE 06 frozen migrations and application-service contracts."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "ccf2263104455681cc07ecceda2569c4f7ce0de9"
FROZEN_MIGRATIONS = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
}
ACCEPTED_MIGRATION_0006 = "08763076ce7cbd77e585bf06b10bc856e7b8f02193484b1db974db95143cebd0"
REQUIRED_PERMISSIONS = {
    "stock.read", "stock.opening.post", "stock.adjust", "stock.transfer", "stock.count",
    "stock.reservation.manage", "stock.reconcile", "stock.negative.override",
    "purchase_order.confirm", "purchase_receipt.post", "purchase_invoice.post", "purchase_return.post",
}
REQUIRED_COMMANDS = {
    "list_stock_balances", "list_stock_movements", "create_opening_stock", "review_opening_stock",
    "post_opening_stock", "post_stock_adjustment", "post_stock_transfer", "create_inventory_count",
    "update_inventory_count", "review_inventory_count", "post_inventory_count", "get_inventory_count",
    "create_stock_reservation", "release_stock_reservation", "consume_stock_reservation",
    "cancel_stock_reservation", "list_active_stock_reservations", "reconcile_stock_balances",
    "rebuild_stock_balances", "create_purchase_order", "update_purchase_order", "confirm_purchase_order",
    "cancel_purchase_order", "hold_purchase_order", "create_purchase_receipt", "post_purchase_receipt",
    "create_purchase_invoice", "post_purchase_invoice", "direct_receive_and_invoice",
    "post_purchase_return", "list_purchasing_documents", "get_purchasing_document",
}


def fail(message: str) -> None:
    raise SystemExit(f"PHASE06 VERIFY FAILED: {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    migrations = sorted((ROOT / "database/migrations").glob("*.sql"))
    names = [path.name for path in migrations]
    if names[:5] != list(FROZEN_MIGRATIONS) or len(names) not in {5, 6, 7}:
        fail("accepted migrations 0001-0005 must remain ordered and frozen")
    if len(names) >= 6 and (not names[5].startswith("0006_") or digest(migrations[5]) != ACCEPTED_MIGRATION_0006):
        fail(f"accepted migration 0006 changed or is misplaced: {names[5:]}")
    if len(names) == 7 and not names[6].startswith("0007_"):
        fail(f"the only authorized PHASE 09 additive migration is 0007: {names[6]}")
    for name, expected in FROZEN_MIGRATIONS.items():
        actual = digest(ROOT / "database/migrations" / name)
        if actual != expected:
            fail(f"frozen migration changed: {name}: {actual}")

    subprocess.run([sys.executable, "scripts/verify_schema.py"], cwd=ROOT, check=True)

    seed = (ROOT / "database/seed/reference_data.sql").read_text(encoding="utf-8")
    rust = "\n".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "src-tauri/src/phase06").rglob("*.rs")))
    command_source = (ROOT / "src-tauri/src/commands/phase06.rs").read_text(encoding="utf-8")
    lib_source = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
    gateway = (ROOT / "src/platform/tauri/phase06.ts").read_text(encoding="utf-8")

    missing_permissions = sorted(code for code in REQUIRED_PERMISSIONS if code not in seed and code not in rust)
    if missing_permissions:
        fail(f"missing PHASE 06 permissions: {missing_permissions}")
    missing_commands = sorted(name for name in REQUIRED_COMMANDS if name not in command_source or name not in lib_source or name not in gateway)
    if missing_commands:
        fail(f"missing typed command boundary: {missing_commands}")

    for fragment in (
        "TransactionBehavior::Immediate", "authorize_transaction", "IDEMPOTENCY_CONFLICT",
        "i128", "checked_", "round_half_up", "stock_movements", "stock_balances",
        "PURCHASE_ORDER_TO_RECEIPT", "RECEIPT_TO_INVOICE", "DOCUMENT_TO_RETURN", "over_transformation",
        "PRIVILEGED_OVERRIDE", "stock.negative.override", "STALE_INVENTORY_COUNT",
    ):
        if fragment not in rust:
            fail(f"Rust service contract missing: {fragment}")
    if not re.search(r"SUM\s*\(\s*transformed_quantity_scaled\s*\)", rust, re.IGNORECASE):
        fail("purchase aggregate transformation must use a transactional sibling SUM")
    if "@tauri-apps/api/core" not in gateway or "validatePhase06Response" not in gateway:
        fail("typed frontend Tauri gateway or runtime DTO validation is missing")
    if any(token in gateway for token in ("fetch(", "XMLHttpRequest", "WebSocket(")):
        fail("runtime network primitive appears in PHASE 06 gateway")

    result = {
        "status": "PASS",
        "baseline": BASELINE,
        "frozenMigrationCount": len(FROZEN_MIGRATIONS),
        "schemaMigrationCount": len(migrations),
        "migrationSha256": FROZEN_MIGRATIONS,
        "migration0006": "authorized PHASE 08 additive migration; 0001-0005 remain frozen",
        "phase06Permissions": len(REQUIRED_PERMISSIONS),
        "typedCommands": len(REQUIRED_COMMANDS),
        "purchaseAggregateTransformation": "enforced for order-to-receipt, receipt-to-invoice, and document-to-return",
        "pendingApplicationInvariant": "sales-side aggregate transformation only (PHASE 07)",
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
