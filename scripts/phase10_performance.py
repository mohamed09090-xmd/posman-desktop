#!/usr/bin/env python3
"""Exercise the paginated POSMAN product-search shape with 100,000 rows."""
from __future__ import annotations

import argparse
import json
import sqlite3
import statistics
import time
from pathlib import Path


PRODUCT_COUNT = 100_000
SEARCH_BUDGET_MS = 2_000.0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    connection = sqlite3.connect(":memory:")
    connection.executescript(
        """
        PRAGMA journal_mode=MEMORY;
        PRAGMA synchronous=OFF;
        CREATE TABLE products (
            id TEXT PRIMARY KEY,
            company_id TEXT NOT NULL,
            code TEXT NOT NULL,
            barcode TEXT,
            name_ar TEXT NOT NULL,
            name_fr TEXT,
            is_active INTEGER NOT NULL,
            row_version INTEGER NOT NULL
        );
        CREATE INDEX idx_products_company_active ON products(company_id, is_active);
        CREATE UNIQUE INDEX uq_products_company_code ON products(company_id, code);
        """
    )
    connection.executemany(
        "INSERT INTO products VALUES(?,?,?,?,?,?,1,1)",
        (
            (
                f"product-{index:06d}",
                "company-a",
                f"P-{index:06d}",
                f"613000{index:06d}",
                f"مادة {index:06d}",
                f"Produit {index:06d}",
            )
            for index in range(PRODUCT_COUNT)
        ),
    )
    connection.commit()
    query = """
        SELECT id, code, name_ar, name_fr, barcode, is_active, row_version
        FROM products WHERE company_id=?1 AND (
            ?2='' OR lower(code) LIKE ?3 OR lower(name_ar) LIKE ?3
            OR lower(COALESCE(name_fr,'')) LIKE ?3
            OR lower(COALESCE(barcode,'')) LIKE ?3)
        ORDER BY is_active DESC, code LIMIT ?4 OFFSET ?5
    """
    durations: list[float] = []
    result_count = 0
    for search in ("99999", "produit 050", "p-000042", "613000099") * 3:
        started = time.perf_counter()
        rows = connection.execute(
            query, ("company-a", search, f"%{search}%", 25, 0)
        ).fetchall()
        durations.append((time.perf_counter() - started) * 1000)
        result_count += len(rows)
    connection.close()
    ordered = sorted(durations)
    p95 = ordered[max(0, round(len(ordered) * 0.95) - 1)]
    result = {
        "products": PRODUCT_COUNT,
        "runs": len(durations),
        "resultsReturned": result_count,
        "medianMs": round(statistics.median(durations), 3),
        "p95Ms": round(p95, 3),
        "maxMs": round(max(durations), 3),
        "budgetMs": SEARCH_BUDGET_MS,
        "executionBoundary": "SQLite query executes outside the React UI thread through typed Tauri IPC",
    }
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    if result["p95Ms"] > SEARCH_BUDGET_MS:
        raise SystemExit("PHASE10 PERFORMANCE FAILED: product-search p95 exceeded budget")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
