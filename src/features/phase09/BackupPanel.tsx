import { useCallback, useEffect, useState } from "react";

import {
  backupGateway,
  normalizePhase09Error,
  type BackupSettingsView,
  type BackupView,
  type Phase09Locale,
} from "../../platform/tauri/phase09";
import type { Phase09Copy } from "./copy";
import {
  EmptyState,
  IntegrityBadge,
  OperationNotice,
  type OperationStatus,
} from "./shared";

export function BackupPanel({
  locale,
  copy,
  canCreate,
  canRestore,
  canManage,
}: {
  locale: Phase09Locale;
  copy: Phase09Copy;
  canCreate: boolean;
  canRestore: boolean;
  canManage: boolean;
}) {
  const [settings, setSettings] = useState<BackupSettingsView | null>(null);
  const [backups, setBackups] = useState<BackupView[]>([]);
  const [password, setPassword] = useState("");
  const [confirmationText, setConfirmationText] = useState("");
  const [status, setStatus] = useState<OperationStatus>({ kind: "idle" });

  const reload = useCallback(async () => {
    setStatus({ kind: "loading" });
    try {
      const [nextSettings, page] = await Promise.all([
        backupGateway.getSettings(),
        backupGateway.list({ page: 1, pageSize: 50, backupKind: null }),
      ]);
      setSettings(nextSettings);
      setBackups(page.items);
      setStatus({ kind: "idle" });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  async function updateSettings(
    automaticEnabled: boolean,
    weeklyEnabled: boolean,
  ) {
    if (!settings) {
      return;
    }
    setStatus({ kind: "loading" });
    try {
      const updated = await backupGateway.updateSettings({
        automaticEnabled,
        weeklyEnabled,
        expectedRowVersion: settings.rowVersion,
      });
      setSettings(updated);
      setStatus({ kind: "success", message: copy.enabled });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function createBackup() {
    setStatus({ kind: "loading" });
    try {
      const created = await backupGateway.create({ backupKind: "MANUAL" });
      setStatus({ kind: "success", message: created.sha256 });
      await reload();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function importBackup() {
    setStatus({ kind: "loading" });
    try {
      const imported = await backupGateway.import();
      setStatus({ kind: "success", message: imported.sha256 });
      await reload();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function verify(backupId: string) {
    setStatus({ kind: "loading" });
    try {
      const result = await backupGateway.verify({ backupId });
      setStatus({ kind: "success", message: result.sha256 });
      await reload();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function exportBackup(backupId: string) {
    setStatus({ kind: "loading" });
    try {
      const result = await backupGateway.export({ backupId });
      setStatus({ kind: "success", message: result.sha256 });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function restore(backupId: string) {
    if (
      password.length === 0 ||
      confirmationText !== "RESTORE" ||
      !window.confirm(copy.restoreWarning)
    ) {
      setStatus({
        kind: "error",
        message: copy.restoreWarning,
        code: "RESTORE_CONFIRMATION_REQUIRED",
      });
      return;
    }
    setStatus({ kind: "loading" });
    try {
      await backupGateway.restore({
        backupId,
        currentPassword: password,
        confirmationText: "RESTORE",
        confirmed: true,
      });
      setPassword("");
      setConfirmationText("");
      setStatus({ kind: "success", message: copy.restore });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function remove(backupId: string) {
    if (!window.confirm(copy.confirmation)) {
      return;
    }
    setStatus({ kind: "loading" });
    try {
      await backupGateway.delete({ backupId });
      setStatus({ kind: "success", message: copy.deleteBackup });
      await reload();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  return (
    <section className="phase09-panel" aria-labelledby="phase09-backup-title">
      <header>
        <h2 id="phase09-backup-title">{copy.backup}</h2>
        <p className="phase09-warning">{copy.unencrypted}</p>
      </header>
      {settings ? (
        <section className="phase09-card phase09-settings" aria-label={copy.automatic}>
          <label className="phase09-checkbox">
            <input
              type="checkbox"
              checked={settings.automaticEnabled}
              disabled={!canManage}
              onChange={(event) =>
                void updateSettings(
                  event.currentTarget.checked,
                  settings.weeklyEnabled,
                )
              }
            />
            <span>{copy.automatic}</span>
          </label>
          <label className="phase09-checkbox">
            <input
              type="checkbox"
              checked={settings.weeklyEnabled}
              disabled={!canManage}
              onChange={(event) =>
                void updateSettings(
                  settings.automaticEnabled,
                  event.currentTarget.checked,
                )
              }
            />
            <span>{copy.weekly}</span>
          </label>
          <span>{settings.timezoneName}</span>
          <IntegrityBadge state={settings.encryptionStatus} />
        </section>
      ) : null}
      <div className="phase09-actions">
        <button
          type="button"
          className="phase09-button--primary"
          disabled={!canCreate}
          onClick={() => void createBackup()}
        >
          {copy.createBackup}
        </button>
        <button
          type="button"
          disabled={!canCreate}
          onClick={() => void importBackup()}
        >
          {copy.importBackup}
        </button>
      </div>
      <OperationNotice status={status} copy={copy} />
      {backups.length === 0 ? (
        <EmptyState copy={copy} />
      ) : (
        <div className="phase09-backup-layout">
          <div className="phase09-table-wrap" tabIndex={0}>
            <table>
              <thead>
                <tr>
                  <th>Kind</th>
                  <th>Date</th>
                  <th>Schema</th>
                  <th>{copy.state}</th>
                  <th>{copy.hash}</th>
                  <th aria-label="Actions" />
                </tr>
              </thead>
              <tbody>
                {backups.map((backup) => (
                  <tr key={backup.backupId}>
                    <td>{backup.backupKind}</td>
                    <td>{backup.createdAt}</td>
                    <td>{backup.schemaVersion}</td>
                    <td>
                      <IntegrityBadge state={backup.verificationStatus} />
                    </td>
                    <td>
                      <code title={backup.sha256}>
                        {backup.sha256.slice(0, 12)}…
                      </code>
                    </td>
                    <td>
                      <div className="phase09-row-actions">
                        <button
                          type="button"
                          onClick={() => void verify(backup.backupId)}
                        >
                          {copy.verify}
                        </button>
                        <button
                          type="button"
                          onClick={() => void exportBackup(backup.backupId)}
                        >
                          {copy.export}
                        </button>
                        <button
                          type="button"
                          disabled={!canRestore}
                          onClick={() => void restore(backup.backupId)}
                        >
                          {copy.restore}
                        </button>
                        <button
                          type="button"
                          className="phase09-button--danger"
                          disabled={!canManage || backup.selectedForRestore}
                          onClick={() => void remove(backup.backupId)}
                        >
                          {copy.deleteBackup}
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <aside className="phase09-card phase09-restore-box">
            <h3>{copy.restore}</h3>
            <p>{copy.restoreWarning}</p>
            <label>
              <span>{copy.password}</span>
              <input
                type="password"
                value={password}
                onChange={(event) => setPassword(event.currentTarget.value)}
                autoComplete="current-password"
              />
            </label>
            <label>
              <span>{copy.confirmationText}</span>
              <input
                dir="ltr"
                value={confirmationText}
                onChange={(event) => setConfirmationText(event.currentTarget.value)}
                autoComplete="off"
                spellCheck={false}
              />
            </label>
            <p lang={locale === "ar-DZ" ? "ar" : "fr"}>
              {confirmationText === "RESTORE" ? copy.confirmation : "RESTORE"}
            </p>
          </aside>
        </div>
      )}
    </section>
  );
}
