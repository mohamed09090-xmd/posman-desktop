import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const root = new URL("../../", import.meta.url);

function source(path: string): string {
  return readFileSync(new URL(path, root), "utf8");
}

const workspace = source("src/features/phase09/Phase09Workspace.tsx");
const copy = source("src/features/phase09/copy.ts");
const styles = source("src/features/phase09/phase09.css");
const documents = source("src/features/phase09/DocumentsPanel.tsx");
const templates = source("src/features/phase09/TemplatesPanel.tsx");
const reports = source("src/features/phase09/ReportsPanel.tsx");
const audit = source("src/features/phase09/AuditPanel.tsx");
const backup = source("src/features/phase09/BackupPanel.tsx");
const appRoot = source("src/app/AppRoot.tsx");
const shared = source("src/features/phase09/shared.tsx");
const all = [workspace, copy, styles, documents, templates, reports, audit, backup].join("\n");

test("Arabic is the default locale and uses RTL", () => {
  assert.match(workspace, /locale = "ar-DZ"/);
  assert.match(workspace, /locale === "ar-DZ" \? "rtl" : "ltr"/);
  assert.match(copy, /"ar-DZ"/);
  assert.match(copy, /الوثائق/);
});

test("French copy and LTR locale are present", () => {
  assert.match(copy, /"fr-DZ"/);
  assert.match(copy, /Sauvegarde et restauration/);
  assert.match(workspace, /locale === "ar-DZ" \? "ar" : "fr"/);
});

test("all five PHASE 09 sections are operationally represented", () => {
  for (const section of [
    "DocumentsPanel",
    "TemplatesPanel",
    "ReportsPanel",
    "AuditPanel",
    "BackupPanel",
  ]) {
    assert.match(workspace, new RegExp(section));
  }
});

test("permissions are enforced in the workspace rather than hidden only by CSS", () => {
  for (const permission of [
    "documents.templates.view",
    "documents.templates.manage",
    "documents.render",
    "documents.export",
    "documents.print",
    "reports.view",
    "reports.export",
    "audit.view",
    "audit.export",
    "backup.view",
    "backup.create",
    "backup.restore",
    "backup.manage",
  ]) {
    assert.match(workspace, new RegExp(permission.replaceAll(".", "\\.")));
  }
  assert.match(workspace, /PermissionBoundary/);
});

test("loading, empty, error, integrity, and destructive states are present", () => {
  assert.match(all, /OperationNotice/);
  assert.match(all, /EmptyState/);
  assert.match(all, /IntegrityBadge/);
  assert.match(backup, /RESTORE/);
  assert.match(backup, /current-password/);
  assert.match(backup, /window\.confirm/);
});

test("template editing is structured and never exposes raw HTML or CSS", () => {
  assert.match(templates, /showLogo/);
  assert.match(templates, /showTradeRegister/);
  assert.match(templates, /orientation/);
  assert.match(templates, /footerTextAr/);
  assert.doesNotMatch(templates, /dangerouslySetInnerHTML|htmlTemplate|cssTemplate/);
});

test("keyboard focus and reduced motion are explicit", () => {
  assert.match(styles, /:focus-visible/);
  assert.match(styles, /prefers-reduced-motion/);
  assert.match(all, /tabIndex=\{0\}/);
  assert.match(workspace, /aria-current/);
});

test("responsive layouts cover desktop and constrained viewports", () => {
  assert.match(styles, /@media \(max-width: 70rem\)/);
  assert.match(styles, /@media \(max-width: 48rem\)/);
  assert.match(styles, /overflow-x: auto/);
  assert.match(styles, /max-width: 100%/);
});

test("frontend remains offline and does not gain filesystem authority", () => {
  assert.doesNotMatch(all, /\bfetch\s*\(|XMLHttpRequest|WebSocket|https?:\/\//);
  assert.doesNotMatch(all, /@tauri-apps\/plugin-fs|readFile|writeFile/);
});

test("AppRoot exposes PHASE 09 through the authenticated session permission set", () => {
  assert.match(appRoot, /#phase09/);
  assert.match(appRoot, /getCurrentSession\(\)/);
  assert.match(appRoot, /permissions=\{session\.permissions\}/);
  assert.match(shared, /permissions\.includes\("\*"\)/);
});
