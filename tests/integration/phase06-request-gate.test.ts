import assert from "node:assert/strict";
import test from "node:test";

import {
  LatestRequestGate,
  type RequestSnapshot,
} from "../../src/features/phase06/request-gate.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

test("latest PHASE 06 request suppresses stale results", async () => {
  const gate = new LatestRequestGate<string[]>();
  const first = deferred<string[]>();
  const second = deferred<string[]>();
  const snapshots: Array<RequestSnapshot<string[]>> = [];
  gate.activate();

  const firstRun = gate.run(() => first.promise, (value) => value.length === 0, (value) => snapshots.push(value));
  const secondRun = gate.run(() => second.promise, (value) => value.length === 0, (value) => snapshots.push(value));
  second.resolve(["new"]);
  await secondRun;
  first.resolve(["old"]);
  await firstRun;

  assert.deepEqual(snapshots.at(-1), { state: "ready", value: ["new"] });
  assert.equal(snapshots.some((snapshot) => snapshot.value?.[0] === "old"), false);
});

test("deactivation prevents post-unmount publication", async () => {
  const gate = new LatestRequestGate<string[]>();
  const pending = deferred<string[]>();
  const snapshots: Array<RequestSnapshot<string[]>> = [];
  gate.activate();
  const run = gate.run(() => pending.promise, (value) => value.length === 0, (value) => snapshots.push(value));
  gate.deactivate();
  pending.resolve(["late"]);
  await run;
  assert.equal(snapshots.some((snapshot) => snapshot.value?.[0] === "late"), false);
});

test("retry after an error can publish a ready result", async () => {
  const gate = new LatestRequestGate<string[]>();
  const snapshots: Array<RequestSnapshot<string[]>> = [];
  gate.activate();
  await gate.run(async () => { throw new Error("offline local error"); }, () => false, (value) => snapshots.push(value));
  await gate.run(async () => ["recovered"], (value) => value.length === 0, (value) => snapshots.push(value));
  assert.equal(snapshots.some((snapshot) => snapshot.state === "error"), true);
  assert.deepEqual(snapshots.at(-1), { state: "ready", value: ["recovered"] });
});
