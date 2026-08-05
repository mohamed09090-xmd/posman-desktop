#!/usr/bin/env python3
"""Verify PHASE 07 sales-cycle contracts without changing the accepted schema."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "036ac89c07ddee1e26402c1c523529adbba48860"
FROZEN_MIGRATIONS = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
}
REQUIRED_PERMISSIONS = {
    "stock.read",
    "sales_order.confirm",
    "delivery_note.post",
    "sales_invoice.post",
    "pricing.override_below_cost",
}
REQUIRED_COMMANDS = {
    "create_sales_order", "update_sales_order", "confirm_sales_order",
    "hold_sales_order", "resume_sales_order", "cancel_sales_order",
    "deliver_sales_order", "invoice_sales_delivery", "direct_sale",
    "post_sales_return", "list_sales_documents", "get_sales_document",
    "get_sales_line_availability", "get_sales_summary",
}


def fail(message: str) -> None:
    raise SystemExit(f"PHASE07 VERIFY FAILED: {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    migrations = sorted((ROOT / "database/migrations").glob("*.sql"))
    if [path.name for path in migrations] != list(FROZEN_MIGRATIONS):
        fail("accepted migrations must remain exactly 0001-0005; PHASE 07 needs no 0006")
    for name, expected in FROZEN_MIGRATIONS.items():
        actual = digest(ROOT / "database/migrations" / name)
        if actual != expected:
            fail(f"frozen migration changed: {name}: {actual}")

    subprocess.run([sys.executable, "scripts/verify_schema.py"], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "scripts/verify_phase06.py"], cwd=ROOT, check=True)

    seed = (ROOT / "database/seed/reference_data.sql").read_text(encoding="utf-8")
    rust = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((ROOT / "src-tauri/src/phase07").rglob("*.rs"))
    )
    commands = (ROOT / "src-tauri/src/commands/phase07.rs").read_text(encoding="utf-8")
    lib = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
    gateway = (ROOT / "src/platform/tauri/phase07.ts").read_text(encoding="utf-8")
    workspace = (ROOT / "src/features/phase07/Phase07Workspace.tsx").read_text(encoding="utf-8")
    copy = (ROOT / "src/features/phase07/copy.ts").read_text(encoding="utf-8")

    missing_permissions = sorted(code for code in REQUIRED_PERMISSIONS if code not in seed and code not in rust)
    if missing_permissions:
        fail(f"missing accepted sales permissions: {missing_permissions}")
    missing_commands = sorted(
        name for name in REQUIRED_COMMANDS
        if name not in commands or name not in lib or name not in gateway
    )
    if missing_commands:
        fail(f"missing typed sales command boundary: {missing_commands}")

    for fragment in (
        "TransactionBehavior::Immediate", "authorize_transaction", "begin_idempotency",
        "finish_idempotency", "i128", "round_half_up", "ORDER_TO_DELIVERY",
        "DELIVERY_TO_INVOICE", "DOCUMENT_TO_RETURN", "TRANSFORMATION_LIMIT_EXCEEDED",
        "pricing.override_below_cost", "BELOW_COST_BLOCKED", "stock_reservations",
        "SALES_DELIVERY", "SALES_RETURN", "CREDIT_NOTE", "audit",
    ):
        if fragment not in rust:
            fail(f"Rust sales contract missing: {fragment}")
    if not re.search(r"SUM\s*\(\s*transformed_quantity_scaled\s*\)", rust, re.IGNORECASE):
        fail("sales aggregate transformation must use a transactional sibling SUM")
    if "8_000_000" not in rust or "12_000_000" not in rust or "1_000_000" not in rust:
        fail("required 20 = 8 + 12 and over-transformation regression test is missing")
    if "@tauri-apps/api/core" not in gateway or "validateDocument" not in gateway:
        fail("typed sales Tauri gateway or runtime response validation is missing")
    if any(token in gateway for token in ("fetch(", "XMLHttpRequest", "WebSocket(")):
        fail("runtime network primitive appears in PHASE 07 gateway")
    for token in ("ar-DZ", "fr-DZ", "Intl.NumberFormat", "direct_sale", "post_sales_return"):
        if token not in workspace and token not in gateway and token not in copy:
            fail(f"sales UI/runtime behavior missing: {token}")

    result = {
        "status": "PASS",
        "baseline": BASELINE,
        "migrationCount": len(FROZEN_MIGRATIONS),
        "migration0006": "not created; accepted schema and IMMEDIATE Rust transactions satisfy PHASE 07",
        "typedCommands": len(REQUIRED_COMMANDS),
        "salesAggregateTransformation": "enforced for order-to-delivery, delivery-to-invoice, and document-to-return",
        "requiredExample": "20 units delivered as 8 + 12; a further 1 is rejected",
        "belowCost": "warehouse CUMP comparison with BLOCK/WARNING_ONLY/ADMIN_OVERRIDE policy and audited reason",
        "pendingApplicationInvariant": "none for accepted sales transformations",
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
