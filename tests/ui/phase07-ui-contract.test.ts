import assert from "node:assert/strict";
import test from "node:test";
import { readFile } from "node:fs/promises";
import { PHASE07_COPY,PHASE07_SCREENS } from "../../src/features/phase07/copy.ts";

const workspace=await readFile("src/features/phase07/Phase07Workspace.tsx","utf8");
const css=await readFile("src/features/phase07/phase07.css","utf8");
const app=await readFile("src/app/AppRoot.tsx","utf8");
test("Arabic and French sales copy have exact non-empty parity",()=>{assert.deepEqual(Object.keys(PHASE07_COPY["ar-DZ"]).sort(),Object.keys(PHASE07_COPY["fr-DZ"]).sort());for(const key of Object.keys(PHASE07_COPY["ar-DZ"]))assert.ok(PHASE07_COPY["ar-DZ"][key]?.trim()&&PHASE07_COPY["fr-DZ"][key]?.trim(),key)});
test("nine operational sales screens are reachable and AppRoot exposes PHASE 07",()=>{assert.equal(PHASE07_SCREENS.length,9);for(const screen of PHASE07_SCREENS)assert.ok(workspace.includes(`"${screen}"`),screen);assert.match(app,/Phase07Workspace/);assert.match(app,/#phase07/)});
test("sales workbench contains scan-first, partial delivery, direct sale, returns and below-cost controls",()=>{for(const token of ["p7-scan","deliver_sales_order","invoice_sales_delivery","direct_sale","post_sales_return","belowCostOverrideReason","window.confirm"])assert.ok(workspace.includes(token),token)});
test("sales CSS preserves ledger identity, accessible focus, scrolling and reduced motion",()=>{assert.match(css,/\.p7-table-wrap\{[^}]*overflow:auto/);assert.match(css,/:focus-visible/);assert.match(css,/prefers-reduced-motion/);assert.equal(css.includes("linear-gradient"),false);assert.equal(css.includes("backdrop-filter"),false)});
