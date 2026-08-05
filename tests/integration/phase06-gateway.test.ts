import assert from "node:assert/strict";
import test from "node:test";

import {
  PHASE06_COMMANDS,
  Phase06GatewayError,
  createPhase06Gateway,
  normalizePhase06Error,
  validateEntityResult,
  validateReconciliation,
  validateStockBalances,
} from "../../src/platform/tauri/phase06.ts";

const entity = {
  id: "doc-1",
  documentNumber: "OUV-000001",
  status: "POSTED",
  rowVersion: 2,
  replayed: false,
};

const balance = {
  productId: "product-1",
  productCode: "P-1",
  productName: "Article",
  warehouseId: "warehouse-1",
  warehouseName: "Dépôt",
  onHandScaled: 2_000_000,
  reservedScaled: 250_000,
  availableScaled: 1_750_000,
  averageCostScaled: 12_345,
  inventoryValueMinor: 2_469,
  rowVersion: 1,
};

test("PHASE 06 gateway uses exact commands and request envelopes", async () => {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const gateway = createPhase06Gateway(async (command, args) => {
    calls.push({ command, args });
    if (command === "list_stock_balances") return [balance];
    if (command === "list_active_stock_reservations") return [];
    if (command === "reconcile_stock_balances") {
      return { rows: [], mismatchCount: 0, rebuilt: false };
    }
    return entity;
  });

  const result = await gateway.call("post_opening_stock", {
    idempotencyKey: "opening-key-0001",
    payload: { documentId: "doc-1", rowVersion: 1 },
  });
  assert.deepEqual(result, entity);
  await gateway.balances({ warehouseId: "warehouse-1", limit: 20 });
  await gateway.call("list_active_stock_reservations");
  await gateway.call("reconcile_stock_balances");

  assert.deepEqual(calls[0], {
    command: "post_opening_stock",
    args: {
      request: {
        idempotencyKey: "opening-key-0001",
        payload: { documentId: "doc-1", rowVersion: 1 },
      },
    },
  });
  assert.deepEqual(calls[1], {
    command: "list_stock_balances",
    args: { request: { warehouseId: "warehouse-1", limit: 20 } },
  });
  assert.deepEqual(calls[2], {
    command: "list_active_stock_reservations",
    args: undefined,
  });
});

test("all exact PHASE 06 command names are registered in the gateway contract", () => {
  assert.equal(PHASE06_COMMANDS.length, 32);
  for (const command of [
    "list_stock_balances",
    "list_stock_movements",
    "post_opening_stock",
    "post_stock_transfer",
    "post_inventory_count",
    "direct_receive_and_invoice",
    "post_purchase_return",
    "rebuild_stock_balances",
  ]) {
    assert.ok(PHASE06_COMMANDS.includes(command as never), command);
  }
});

test("DTO validation rejects unsafe or malformed payloads", () => {
  assert.deepEqual(validateEntityResult(entity), entity);
  assert.deepEqual(validateStockBalances([balance]), [balance]);
  assert.deepEqual(
    validateReconciliation({ rows: [], mismatchCount: 0, rebuilt: false }),
    { rows: [], mismatchCount: 0, rebuilt: false },
  );
  assert.throws(
    () => validateEntityResult({ ...entity, rowVersion: 1.5 }),
    (error: unknown) =>
      error instanceof Phase06GatewayError && error.code === "OPERATION_FAILED",
  );
  assert.throws(
    () => validateStockBalances([{ ...balance, availableScaled: "1750000" }]),
    (error: unknown) =>
      error instanceof Phase06GatewayError && error.code === "OPERATION_FAILED",
  );
});

test("unknown and sensitive errors normalize to safe stable codes", async () => {
  const known = normalizePhase06Error({
    code: "INSUFFICIENT_STOCK",
    message: "SELECT password_hash FROM users /mnt/customer/posman.sqlite3",
  });
  assert.equal(known.code, "INSUFFICIENT_STOCK");
  assert.ok(!known.message.includes("SELECT"));
  assert.ok(!known.message.includes("/mnt/"));

  const gateway = createPhase06Gateway(async () => {
    throw new Error("sqlite: SELECT * FROM stock_movements at /private/db");
  });
  await assert.rejects(
    gateway.balances(),
    (error: unknown) =>
      error instanceof Phase06GatewayError &&
      error.code === "OPERATION_FAILED" &&
      !error.message.includes("sqlite"),
  );
});

test("production gateway source keeps the browser invoker behind DEV detection", async () => {
  const fs = await import("node:fs/promises");
  const source = await fs.readFile("src/platform/tauri/phase06.ts", "utf8");
  assert.match(source, /import\.meta\.env\.DEV/);
  assert.match(source, /isTauri\(\)/);
  assert.match(source, /@tauri-apps\/api\/core/);
  assert.ok(!source.includes("fetch("));
  assert.ok(!source.includes("XMLHttpRequest"));
  assert.ok(!source.includes("WebSocket("));
});
