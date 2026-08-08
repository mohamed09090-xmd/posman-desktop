import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  Phase09GatewayError,
  RequestGate,
  normalizePhase09Error,
} from "../../src/platform/tauri/phase09.ts";

const root = new URL("../../", import.meta.url);

function source(path: string): string {
  return readFileSync(new URL(path, root), "utf8");
}

const gatewaySources = ["src/platform/tauri/phase09.ts"].map(source);

const combined = gatewaySources.join("\n");

const requiredCommands = [
  "phase09_list_templates",
  "phase09_get_template",
  "phase09_create_template_draft",
  "phase09_update_template_draft",
  "phase09_publish_template",
  "phase09_retire_template",
  "phase09_preview_document",
  "phase09_render_document",
  "phase09_list_rendered_documents",
  "phase09_get_rendered_document",
  "phase09_verify_rendered_document",
  "phase09_export_rendered_pdf",
  "phase09_print_rendered_document",
  "phase09_list_reports",
  "phase09_run_report",
  "phase09_export_report_csv",
  "phase09_export_report_pdf",
  "phase09_list_audit_events",
  "phase09_export_audit_csv",
  "phase09_get_backup_settings",
  "phase09_update_backup_settings",
  "phase09_create_backup",
  "phase09_list_backups",
  "phase09_verify_backup",
  "phase09_export_backup",
  "phase09_import_backup",
  "phase09_restore_backup",
  "phase09_delete_backup",
];

test("PHASE 09 gateway uses every required command name exactly", () => {
  for (const command of requiredCommands) {
    assert.match(combined, new RegExp(`\\b${command}\\b`), command);
  }
});

test("PHASE 09 gateway contains no SQL, runtime network client, or filesystem API", () => {
  assert.doesNotMatch(combined, /\b(?:SELECT|INSERT|UPDATE|DELETE\s+FROM|PRAGMA)\b/i);
  assert.doesNotMatch(combined, /\bfetch\s*\(/);
  assert.doesNotMatch(combined, /XMLHttpRequest|WebSocket|EventSource/);
  assert.doesNotMatch(combined, /@tauri-apps\/plugin-fs|readFile|writeFile|removeFile/);
  assert.doesNotMatch(combined, /https?:\/\//);
});

test("RequestGate suppresses stale responses", () => {
  const gate = new RequestGate();
  const first = gate.begin();
  const second = gate.begin();
  assert.equal(gate.isCurrent(first), false);
  assert.equal(gate.isCurrent(second), true);
  gate.invalidate();
  assert.equal(gate.isCurrent(second), false);
});

test("safe errors are normalized without leaking arbitrary objects", () => {
  const normalized = normalizePhase09Error({
    code: "OUTPUT_BUSY",
    message: "The output engine is busy.",
    retryable: true,
  });
  assert.ok(normalized instanceof Phase09GatewayError);
  assert.equal(normalized.code, "OUTPUT_BUSY");
  assert.equal(normalized.retryable, true);

  const unknown = normalizePhase09Error({ password_hash: "hidden" });
  assert.equal(unknown.code, "INTERNAL_ERROR");
  assert.doesNotMatch(unknown.message, /hidden|password/i);
});

test("restore gateway requires exact RESTORE confirmation", () => {
  const backup = source("src/platform/tauri/phase09.ts");
  assert.match(backup, /confirmationText !== "RESTORE"/);
  assert.match(backup, /currentPassword\.length === 0/);
  assert.match(backup, /confirmed !== true/);
});

test("payloads remain wrapped in typed request objects", () => {
  for (const gateway of gatewaySources) {
    const invocations = gateway.match(/invokePhase09<[^>]+>\([\s\S]*?\);/g) ?? [];
    assert.ok(invocations.length > 0);
    assert.doesNotMatch(gateway, /companyId\s*:/);
  }
});

test("browser test injection is development-only and production calls stay on Tauri", () => {
  const gateway = source("src/platform/tauri/phase09.ts");
  assert.match(gateway, /import\.meta\.env\.DEV/);
  assert.match(gateway, /__POSMAN_DEV_PHASE09_INVOKER__/);
  assert.match(gateway, /isTauri\(\)/);
  assert.match(gateway, /return invoke/);
});
