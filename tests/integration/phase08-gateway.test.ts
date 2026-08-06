import assert from "node:assert/strict";
import test from "node:test";
import { createPhase08Gateway, normalizePhase08Error, PHASE08_COMMANDS } from "../../src/platform/tauri/phase08.ts";

test("PHASE 08 exposes one typed gateway for every registered command", () => {
  assert.equal(PHASE08_COMMANDS.length, 35);
  assert.equal(new Set(PHASE08_COMMANDS).size, PHASE08_COMMANDS.length);
});

test("PHASE 08 gateway wraps payloads and validates account rows", async () => {
  const calls: Array<{ command: string; args?: unknown }> = [];
  const gateway = createPhase08Gateway(async (command, args) => {
    calls.push({ command, args });
    return [{ id: "acc-1", code: "411", nameAr: "العملاء", nameFr: "Clients", accountType: "ASSET", normalSide: "DEBIT", allowPosting: true, isActive: true, rowVersion: 1 }];
  });
  const rows = await gateway.call<Array<{ code: string }>>("list_accounts");
  assert.equal(rows[0]?.code, "411");
  assert.deepEqual(calls, [{ command: "list_accounts", args: undefined }]);
});

test("PHASE 08 gateway rejects malformed runtime payloads", async () => {
  const gateway = createPhase08Gateway(async () => [{ id: "acc-1" }]);
  await assert.rejects(() => gateway.call("list_accounts"), /local accounting operation/);
});

test("PHASE 08 gateway preserves safe error codes and normalizes unsafe details", () => {
  assert.equal(normalizePhase08Error({ code: "POSTING_RULE_MISSING", message: "sql" }).code, "POSTING_RULE_MISSING");
  assert.equal(normalizePhase08Error({ code: "SQLITE_CONSTRAINT", message: "/secret/path" }).code, "ACCOUNTING_INTERNAL");
});

test("PHASE 08 gateway honors stale-response aborts", async () => {
  const gateway = createPhase08Gateway(async () => await new Promise(resolve => setTimeout(() => resolve([]), 20)));
  const controller = new AbortController();
  const operation = gateway.call("list_accounts", undefined, controller.signal);
  controller.abort();
  await assert.rejects(operation, error => error instanceof DOMException && error.name === "AbortError");
});
