import assert from "node:assert/strict";
import test from "node:test";
import {
  PHASE05_COMMANDS,
  Phase05GatewayError,
  createPhase05Gateway,
} from "../../src/platform/tauri/phase05.ts";

test("PHASE 05 gateway wraps request DTOs under the typed request boundary", async () => {
  const calls: unknown[][] = [];
  const gateway = createPhase05Gateway(async (...args: unknown[]) => {
    calls.push(args);
    return { items: [], page: 1, pageSize: 50, total: 0 };
  });
  await gateway.call("list_products", { page: 1, pageSize: 50 });
  assert.deepEqual(calls, [["list_products", { request: { page: 1, pageSize: 50 } }]]);
});

test("no-argument commands are invoked without an empty payload", async () => {
  const calls: unknown[][] = [];
  const gateway = createPhase05Gateway(async (...args: unknown[]) => {
    calls.push(args);
    return { setupRequired: true };
  });
  await gateway.getSetupStatus();
  assert.deepEqual(calls, [["get_setup_status"]]);
});

test("command inventory includes setup, security, settings, and reference data only", () => {
  for (const command of [
    "complete_initial_setup", "login", "recover_admin_password", "update_company_profile",
    "update_fiscal_setup", "update_document_sequence", "list_users", "list_roles",
    "list_product_families", "list_units", "list_warehouses", "list_warehouse_locations",
    "list_tax_rates", "list_partners", "list_products", "set_product_price",
  ]) assert.ok(PHASE05_COMMANDS.includes(command as never), command);
  assert.equal(PHASE05_COMMANDS.some((command) => /invoice|sale|purchase|payment|stock_post/i.test(command)), false);
});

test("gateway preserves only structured safe codes", async () => {
  const safe = createPhase05Gateway(async () => { throw { code: "PERMISSION_DENIED", message: "secret SQL" }; });
  await assert.rejects(safe.getSetupStatus(), (error: unknown) => error instanceof Phase05GatewayError && error.code === "PERMISSION_DENIED" && !error.message.includes("secret"));
  const unsafe = createPhase05Gateway(async () => { throw { code: "SQLITE_FAILURE", message: "C:\\data\\posman.sqlite3" }; });
  await assert.rejects(unsafe.getSetupStatus(), (error: unknown) => error instanceof Phase05GatewayError && error.code === "OPERATION_FAILED");
});
