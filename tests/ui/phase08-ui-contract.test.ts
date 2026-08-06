import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

const workspace = readFileSync("src/features/phase08/Phase08Workspace.tsx", "utf8");
const css = readFileSync("src/features/phase08/phase08.css", "utf8");
const gateway = readFileSync("src/platform/tauri/phase08.ts", "utf8");
const featureSource = readdirSync("src/features/phase08", { recursive: true, withFileTypes: true })
  .filter((entry) => entry.isFile() && entry.name.endsWith(".tsx"))
  .map((entry) => readFileSync(join(entry.parentPath, entry.name), "utf8"))
  .join("\n");

test("accounting workspace covers configuration, journals, payments, statements, ledgers, periods and retry", () => {
  for (const token of ["install_accounting_template", "save_posting_rule", "create_manual_journal_entry", "post_manual_journal_entry", "reverse_journal_entry", "post_customer_receipt", "post_supplier_payment", "allocate_payment", "get_partner_statement", "get_trial_balance", "get_general_ledger", "close_fiscal_period", "retry_posting_attempt"]) {
    assert.ok(featureSource.includes(token), token);
  }
});

test("accounting workspace preserves Arabic RTL and French LTR operational copy", () => {
  assert.ok(workspace.includes('setLocale("ar-DZ")'));
  assert.ok(workspace.includes('setLocale("fr-DZ")'));
  assert.ok(workspace.includes("formatMoney"));
  assert.ok(workspace.includes("formatDate"));
});

test("accounting workspace has confirmations, retry, stale response and duplicate submit protection", () => {
  assert.ok(featureSource.includes("window.confirm"));
  assert.ok(featureSource.includes("AbortController"));
  assert.ok(featureSource.includes("if (busy) return"));
  assert.ok(featureSource.includes("reload"));
});

test("accounting UI prevents page-level overflow and supports reduced motion", () => {
  assert.ok(css.includes("overflow:hidden"));
  assert.ok(css.includes("overflow:auto"));
  assert.ok(css.includes("prefers-reduced-motion"));
  assert.ok(!css.includes("text-overflow:ellipsis"));
});

test("React boundary contains no raw invoke, SQL or network primitive", () => {
  for (const token of ["invoke(", "SELECT ", "INSERT INTO ", "UPDATE ", "DELETE FROM ", "fetch(", "XMLHttpRequest", "WebSocket("]) {
    assert.equal(featureSource.includes(token), false, token);
  }
  assert.ok(gateway.includes('@tauri-apps/api/core'));
});
