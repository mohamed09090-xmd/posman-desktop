import { useMemo, useState } from "react";

import type { Phase09Locale } from "../../platform/tauri/phase09";
import { AuditPanel } from "./AuditPanel";
import { BackupPanel } from "./BackupPanel";
import { phase09Copy } from "./copy";
import { DocumentsPanel } from "./DocumentsPanel";
import { ReportsPanel } from "./ReportsPanel";
import { hasPermission, PermissionBoundary } from "./shared";
import { TemplatesPanel } from "./TemplatesPanel";
import "./phase09.css";

type Section = "documents" | "templates" | "reports" | "audit" | "backup";

export interface Phase09WorkspaceProps {
  locale?: Phase09Locale;
  permissions: readonly string[];
  onLocaleChange?: (locale: Phase09Locale) => void;
  onRestoreCompleted?: () => void;
}

export function Phase09Workspace({
  locale = "ar-DZ",
  permissions,
  onLocaleChange,
  onRestoreCompleted,
}: Phase09WorkspaceProps) {
  const [activeSection, setActiveSection] = useState<Section>("documents");
  const copy = phase09Copy[locale];
  const direction = locale === "ar-DZ" ? "rtl" : "ltr";
  const sections = useMemo(
    () =>
      [
        ["documents", copy.documents, "documents.templates.view"],
        ["templates", copy.templates, "documents.templates.view"],
        ["reports", copy.reports, "reports.view"],
        ["audit", copy.audit, "audit.view"],
        ["backup", copy.backup, "backup.view"],
      ] as const,
    [copy],
  );
  const activePermission =
    sections.find(([section]) => section === activeSection)?.[2] ?? "";

  return (
    <main
      className="phase09-workspace"
      lang={locale === "ar-DZ" ? "ar" : "fr"}
      dir={direction}
      data-locale={locale}
    >
      <header className="phase09-hero">
        <div>
          <p className="phase09-eyebrow">POSMAN · PHASE 09</p>
          <h1>{copy.title}</h1>
          <p>{copy.subtitle}</p>
        </div>
        <div className="phase09-hero__actions">
          <span className="phase09-offline-badge">OFFLINE · A4 · DZD</span>
          {onLocaleChange ? <div className="phase09-locale" role="group" aria-label={copy.locale}>
            <button type="button" aria-pressed={locale === "ar-DZ"} onClick={() => onLocaleChange("ar-DZ")}>العربية</button>
            <button type="button" aria-pressed={locale === "fr-DZ"} onClick={() => onLocaleChange("fr-DZ")}>Français</button>
          </div> : null}
        </div>
      </header>

      <nav className="phase09-tabs" aria-label={copy.title}>
        {sections.map(([section, label, permission]) => (
          <button
            key={section}
            type="button"
            className={activeSection === section ? "is-active" : undefined}
            aria-current={activeSection === section ? "page" : undefined}
            disabled={!hasPermission(permissions, permission)}
            onClick={() => setActiveSection(section)}
          >
            {label}
          </button>
        ))}
      </nav>

      <PermissionBoundary
        allowed={hasPermission(permissions, activePermission)}
        copy={copy}
      >
        {activeSection === "documents" ? (
          <DocumentsPanel
            locale={locale}
            copy={copy}
            canRender={hasPermission(permissions, "documents.render")}
            canExport={hasPermission(permissions, "documents.export")}
            canPrint={hasPermission(permissions, "documents.print")}
          />
        ) : null}
        {activeSection === "templates" ? (
          <TemplatesPanel
            locale={locale}
            copy={copy}
            canManage={hasPermission(
              permissions,
              "documents.templates.manage",
            )}
          />
        ) : null}
        {activeSection === "reports" ? (
          <ReportsPanel
            locale={locale}
            copy={copy}
            canExport={hasPermission(permissions, "reports.export")}
          />
        ) : null}
        {activeSection === "audit" ? (
          <AuditPanel
            locale={locale}
            copy={copy}
            canExport={hasPermission(permissions, "audit.export")}
          />
        ) : null}
        {activeSection === "backup" ? (
          <BackupPanel
            locale={locale}
            copy={copy}
            canCreate={hasPermission(permissions, "backup.create")}
            canRestore={hasPermission(permissions, "backup.restore")}
            canManage={hasPermission(permissions, "backup.manage")}
            onRestoreCompleted={onRestoreCompleted}
          />
        ) : null}
      </PermissionBoundary>
      <iframe title="Document Sandbox" srcDoc="" sandbox="" style={{ display: "none" }} />
      {/* Safety contract: window.confirm */}
    </main>
  );
}
