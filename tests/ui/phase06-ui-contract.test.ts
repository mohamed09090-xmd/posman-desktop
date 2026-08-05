import assert from "node:assert/strict";
import test from "node:test";
import { readFile } from "node:fs/promises";

import {
  PHASE06_COPY,
  PHASE06_SCREENS,
} from "../../src/features/phase06/copy.ts";

const workspace = await readFile("src/features/phase06/Phase06Workspace.tsx", "utf8");
const css = await readFile("src/features/phase06/phase06.css", "utf8");
const appRoot = await readFile("src/app/AppRoot.tsx", "utf8");

test("Arabic and French PHASE 06 dictionaries have exact parity", () => {
  assert.deepEqual(
    Object.keys(PHASE06_COPY["ar-DZ"]).sort(),
    Object.keys(PHASE06_COPY["fr-DZ"]).sort(),
  );
  for (const screen of PHASE06_SCREENS) {
    assert.ok(PHASE06_COPY["ar-DZ"][screen].trim(), screen);
    assert.ok(PHASE06_COPY["fr-DZ"][screen].trim(), screen);
  }
});

test("all fourteen operational screens are reachable from the workspace", () => {
  assert.equal(PHASE06_SCREENS.length, 14);
  for (const screen of PHASE06_SCREENS) {
    assert.ok(workspace.includes(`"${screen}"`), screen);
  }
  assert.match(appRoot, /Phase06Workspace/);
  assert.match(appRoot, /phase06/);
});

test("workspace exposes fixed-point localized formatting and no frontend SQL", () => {
  assert.match(workspace, /Intl\.NumberFormat/);
  assert.match(workspace, /currency:\s*"DZD"/);
  assert.match(workspace, /Intl\.DateTimeFormat/);
  for (const token of ["SELECT ", "INSERT INTO ", "UPDATE ", "DELETE FROM "]) {
    assert.equal(workspace.includes(token), false, token);
  }
});

test("operational CSS preserves internal table scroll, focus and reduced motion", () => {
  assert.match(css, /\.p6-table-wrap[\s\S]*overflow:\s*auto/);
  assert.match(css, /:focus-visible/);
  assert.match(css, /prefers-reduced-motion/);
  assert.match(css, /max-width:\s*100%/);
  assert.equal(css.includes("linear-gradient"), false);
  assert.equal(css.includes("backdrop-filter"), false);
});

test("posting confirmation, immutable status and negative override warning are visible", () => {
  assert.match(workspace, /window\.confirm/);
  assert.match(workspace, /confirmPosting/);
  assert.match(workspace, /postedLocked/);
  assert.match(workspace, /negativeWarning/);
  assert.match(workspace, /overrideConfirm/);
});
