import assert from "node:assert/strict";
import test from "node:test";
import {
  RUNTIME_STATUS_COMMAND,
  RUNTIME_STATUS_INVALID_RESPONSE,
  RUNTIME_STATUS_REQUEST_FAILED,
  RUNTIME_STATUS_UNAVAILABLE,
  RuntimeGatewayError,
  createRuntimeStatusGateway,
  type RuntimeStatus,
} from "../../src/platform/tauri/runtime-status.ts";
import {
  RuntimeStatusController,
  type RuntimeViewState,
} from "../../src/features/runtime/runtime-state.ts";

const readyStatus: RuntimeStatus = {
  databaseReady: true,
  schemaVersion: "0004",
  migrationCount: 4,
  foreignKeysEnabled: true,
  journalMode: "wal",
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

async function flushMicrotasks(): Promise<void> {
  await Promise.resolve();
}

async function waitFor(
  controller: RuntimeStatusController,
  predicate: (state: RuntimeViewState) => boolean,
): Promise<RuntimeViewState> {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    const state = controller.getSnapshot();
    if (predicate(state)) {
      return state;
    }
    await new Promise<void>((resolve) => setImmediate(resolve));
  }
  throw new Error(`Timed out waiting for runtime state: ${controller.getSnapshot().kind}`);
}

test("gateway invokes get_runtime_status exactly without arguments", async () => {
  const calls: unknown[][] = [];
  const gateway = createRuntimeStatusGateway(async (...args: unknown[]) => {
    calls.push(args);
    return readyStatus;
  });

  const status = await gateway.getRuntimeStatus();

  assert.deepEqual(status, readyStatus);
  assert.equal(RUNTIME_STATUS_COMMAND, "get_runtime_status");
  assert.deepEqual(calls, [["get_runtime_status"]]);
});

test("gateway accepts a structurally valid future schema version", async () => {
  const gateway = createRuntimeStatusGateway(async () => ({
    ...readyStatus,
    schemaVersion: "0012",
    migrationCount: 12,
    journalMode: "delete",
  }));

  assert.deepEqual(await gateway.getRuntimeStatus(), {
    ...readyStatus,
    schemaVersion: "0012",
    migrationCount: 12,
    journalMode: "delete",
  });
});

test("gateway rejects null and arrays", async (t) => {
  for (const payload of [null, [], [readyStatus]]) {
    await t.test(String(payload), async () => {
      const gateway = createRuntimeStatusGateway(async () => payload);
      await assert.rejects(
        gateway.getRuntimeStatus(),
        (error: unknown) =>
          error instanceof RuntimeGatewayError &&
          error.code === RUNTIME_STATUS_INVALID_RESPONSE,
      );
    });
  }
});

test("gateway rejects every field with the wrong type", async (t) => {
  const malformed: Array<[string, Record<string, unknown>]> = [
    ["databaseReady", { ...readyStatus, databaseReady: "true" }],
    ["schemaVersion", { ...readyStatus, schemaVersion: 4 }],
    ["migrationCount", { ...readyStatus, migrationCount: "4" }],
    ["foreignKeysEnabled", { ...readyStatus, foreignKeysEnabled: "true" }],
    ["journalMode", { ...readyStatus, journalMode: false }],
  ];

  for (const [field, payload] of malformed) {
    await t.test(field, async () => {
      const gateway = createRuntimeStatusGateway(async () => payload);
      await assert.rejects(
        gateway.getRuntimeStatus(),
        (error: unknown) =>
          error instanceof RuntimeGatewayError &&
          error.code === RUNTIME_STATUS_INVALID_RESPONSE,
      );
    });
  }
});

test("gateway rejects negative, fractional, and non-finite migration counts", async (t) => {
  for (const migrationCount of [-1, 1.5, Number.NaN, Number.POSITIVE_INFINITY]) {
    await t.test(String(migrationCount), async () => {
      const gateway = createRuntimeStatusGateway(async () => ({ ...readyStatus, migrationCount }));
      await assert.rejects(
        gateway.getRuntimeStatus(),
        (error: unknown) =>
          error instanceof RuntimeGatewayError &&
          error.code === RUNTIME_STATUS_INVALID_RESPONSE,
      );
    });
  }
});

test("gateway rejects strings that are empty after trimming", async (t) => {
  for (const payload of [
    { ...readyStatus, schemaVersion: "   " },
    { ...readyStatus, journalMode: "\t\n" },
  ]) {
    await t.test(JSON.stringify(payload), async () => {
      const gateway = createRuntimeStatusGateway(async () => payload);
      await assert.rejects(
        gateway.getRuntimeStatus(),
        (error: unknown) =>
          error instanceof RuntimeGatewayError &&
          error.code === RUNTIME_STATUS_INVALID_RESPONSE,
      );
    });
  }
});

test("gateway preserves the known structured Rust error code without exposing its message", async () => {
  const rawMessage = "C:\\Users\\merchant\\POSMAN\\data\\posman.sqlite3: SQL failure";
  const gateway = createRuntimeStatusGateway(async () => {
    throw { code: RUNTIME_STATUS_UNAVAILABLE, message: rawMessage };
  });

  await assert.rejects(gateway.getRuntimeStatus(), (error: unknown) => {
    assert.ok(error instanceof RuntimeGatewayError);
    assert.equal(error.code, RUNTIME_STATUS_UNAVAILABLE);
    assert.equal(error.message.includes(rawMessage), false);
    return true;
  });
});

test("gateway maps unknown thrown values and unrecognized codes to a stable generic code", async (t) => {
  for (const thrownValue of [
    "failure",
    42,
    { message: "raw" },
    { code: "SENSITIVE_INTERNAL_CODE" },
  ]) {
    await t.test(JSON.stringify(thrownValue), async () => {
      const gateway = createRuntimeStatusGateway(async () => {
        throw thrownValue;
      });
      await assert.rejects(
        gateway.getRuntimeStatus(),
        (error: unknown) =>
          error instanceof RuntimeGatewayError &&
          error.code === RUNTIME_STATUS_REQUEST_FAILED,
      );
    });
  }
});

test("runtime state moves from initializing to ready", async () => {
  const response = deferred<RuntimeStatus>();
  const controller = new RuntimeStatusController({ getRuntimeStatus: () => response.promise });

  controller.activate();
  assert.deepEqual(controller.getSnapshot(), { kind: "initializing", retrying: false });
  response.resolve(readyStatus);

  assert.deepEqual(await waitFor(controller, (state) => state.kind === "ready"), {
    kind: "ready",
    status: readyStatus,
  });
  controller.deactivate();
});

test("StrictMode-style activate/deactivate/activate before the microtask invokes once", async () => {
  const response = deferred<RuntimeStatus>();
  let invocationCount = 0;
  const controller = new RuntimeStatusController({
    getRuntimeStatus() {
      invocationCount += 1;
      return response.promise;
    },
  });

  controller.activate();
  controller.deactivate();
  controller.activate();
  await flushMicrotasks();

  assert.equal(invocationCount, 1);
  response.resolve(readyStatus);
  assert.equal((await waitFor(controller, (state) => state.kind === "ready")).kind, "ready");
  controller.deactivate();
});

test("runtime state moves from error through a real retry to ready", async () => {
  let invocation = 0;
  const controller = new RuntimeStatusController({
    async getRuntimeStatus() {
      invocation += 1;
      if (invocation === 1) {
        throw new RuntimeGatewayError(RUNTIME_STATUS_UNAVAILABLE);
      }
      return readyStatus;
    },
  });

  controller.activate();
  assert.equal((await waitFor(controller, (state) => state.kind === "error")).kind, "error");
  assert.equal(controller.retry(), true);
  assert.deepEqual(controller.getSnapshot(), { kind: "initializing", retrying: true });
  assert.equal(controller.retry(), false);
  assert.equal((await waitFor(controller, (state) => state.kind === "ready")).kind, "ready");
  assert.equal(invocation, 2);
  controller.deactivate();
});

test("runtime state rejects structurally valid payloads that are not ready", async () => {
  const controller = new RuntimeStatusController({
    async getRuntimeStatus() {
      return { ...readyStatus, foreignKeysEnabled: false };
    },
  });

  controller.activate();
  const state = await waitFor(controller, (candidate) => candidate.kind === "error");
  assert.deepEqual(state, { kind: "error", code: "RUNTIME_STATUS_NOT_READY" });
  controller.deactivate();
});

test("a stale response from an earlier request cannot overwrite the latest result", async () => {
  const first = deferred<RuntimeStatus>();
  const second = deferred<RuntimeStatus>();
  let invocation = 0;
  const controller = new RuntimeStatusController({
    getRuntimeStatus() {
      invocation += 1;
      return invocation === 1 ? first.promise : second.promise;
    },
  });

  controller.activate();
  await flushMicrotasks();
  assert.equal(invocation, 1);

  controller.deactivate();
  controller.activate();
  await flushMicrotasks();
  assert.equal(invocation, 2);

  second.resolve({ ...readyStatus, schemaVersion: "0005", migrationCount: 5 });
  const latest = await waitFor(controller, (state) => state.kind === "ready");
  assert.equal(latest.kind === "ready" ? latest.status.schemaVersion : "", "0005");

  first.resolve(readyStatus);
  await new Promise<void>((resolve) => setImmediate(resolve));
  const finalState = controller.getSnapshot();
  assert.equal(finalState.kind === "ready" ? finalState.status.schemaVersion : "", "0005");
  controller.deactivate();
});

test("deactivation prevents state updates after unmount", async () => {
  const response = deferred<RuntimeStatus>();
  const controller = new RuntimeStatusController({ getRuntimeStatus: () => response.promise });

  controller.activate();
  await flushMicrotasks();
  controller.deactivate();
  response.resolve(readyStatus);
  await new Promise<void>((resolve) => setImmediate(resolve));

  assert.deepEqual(controller.getSnapshot(), { kind: "initializing", retrying: false });
});

test("absence of a runtime gateway produces preview instead of false readiness", () => {
  const controller = new RuntimeStatusController(null);
  assert.deepEqual(controller.getSnapshot(), { kind: "preview" });
  controller.activate();
  assert.deepEqual(controller.getSnapshot(), { kind: "preview" });
  assert.equal(controller.retry(), false);
});
