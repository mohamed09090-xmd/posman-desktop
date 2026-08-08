import { useCallback, useState } from "react";

import {
  auditGateway,
  normalizePhase09Error,
  type AuditEventView,
  type AuditRequest,
  type Phase09Locale,
} from "../../platform/tauri/phase09";
import type { Phase09Copy } from "./copy";
import {
  EmptyState,
  OperationNotice,
  type OperationStatus,
} from "./shared";

export function AuditPanel({
  locale,
  copy,
  canExport,
}: {
  locale: Phase09Locale;
  copy: Phase09Copy;
  canExport: boolean;
}) {
  const [startAt, setStartAt] = useState("");
  const [endAt, setEndAt] = useState("");
  const [outcome, setOutcome] = useState<AuditRequest["outcome"]>(null);
  const [sensitiveOnly, setSensitiveOnly] = useState(false);
  const [events, setEvents] = useState<AuditEventView[]>([]);
  const [status, setStatus] = useState<OperationStatus>({ kind: "idle" });

  const request = useCallback(
    (): AuditRequest => ({
      startAt: startAt || null,
      endAt: endAt || null,
      userId: null,
      domain: null,
      action: null,
      entityType: null,
      entityId: null,
      outcome,
      sensitiveOnly,
      page: 1,
      pageSize: 50,
    }),
    [endAt, outcome, sensitiveOnly, startAt],
  );

  async function load() {
    setStatus({ kind: "loading" });
    try {
      const page = await auditGateway.list(request());
      setEvents(page.items);
      setStatus({ kind: "idle" });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function exportCsv() {
    setStatus({ kind: "loading" });
    try {
      const result = await auditGateway.exportCsv(request());
      setStatus({ kind: "success", message: result.sha256 });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  return (
    <section className="phase09-panel" aria-labelledby="phase09-audit-title">
      <header>
        <h2 id="phase09-audit-title">{copy.audit}</h2>
      </header>
      <div className="phase09-form phase09-form--inline">
        <label>
          <span>{copy.startDate}</span>
          <input
            type="datetime-local"
            value={startAt}
            onChange={(event) => setStartAt(event.currentTarget.value)}
          />
        </label>
        <label>
          <span>{copy.endDate}</span>
          <input
            type="datetime-local"
            value={endAt}
            onChange={(event) => setEndAt(event.currentTarget.value)}
          />
        </label>
        <label>
          <span>{copy.outcome}</span>
          <select
            value={outcome ?? ""}
            onChange={(event) =>
              setOutcome(
                (event.currentTarget.value || null) as AuditRequest["outcome"],
              )
            }
          >
            <option value="">—</option>
            <option value="SUCCESS">SUCCESS</option>
            <option value="FAILURE">FAILURE</option>
            <option value="DENIED">DENIED</option>
          </select>
        </label>
        <label className="phase09-checkbox">
          <input
            type="checkbox"
            checked={sensitiveOnly}
            onChange={(event) => setSensitiveOnly(event.currentTarget.checked)}
          />
          <span>{copy.sensitiveOnly}</span>
        </label>
        <div className="phase09-actions">
          <button
            type="button"
            className="phase09-button--primary"
            onClick={() => void load()}
          >
            {copy.applyFilters}
          </button>
          <button
            type="button"
            disabled={!canExport}
            onClick={() => void exportCsv()}
          >
            {copy.exportCsv}
          </button>
        </div>
      </div>
      <OperationNotice status={status} copy={copy} />
      {events.length === 0 ? (
        <EmptyState copy={copy} />
      ) : (
        <div className="phase09-table-wrap" tabIndex={0}>
          <table>
            <thead>
              <tr>
                <th>{copy.startDate}</th>
                <th>User</th>
                <th>Domain</th>
                <th>Action</th>
                <th>Entity</th>
                <th>{copy.outcome}</th>
                <th>Details</th>
              </tr>
            </thead>
            <tbody>
              {events.map((event) => (
                <tr key={event.id}>
                  <td>{event.occurredAt}</td>
                  <td>{event.actorDisplayName ?? event.actorUserId ?? "—"}</td>
                  <td>{event.domain}</td>
                  <td>{event.actionCode}</td>
                  <td>
                    {event.entityType} · {event.entityId}
                  </td>
                  <td>{event.outcome}</td>
                  <td>
                    <pre dir={locale === "ar-DZ" ? "rtl" : "ltr"}>
                      {event.details === null || event.details === undefined
                        ? "—"
                        : JSON.stringify(event.details, null, 2)}
                    </pre>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
