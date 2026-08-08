import { useCallback, useEffect, useMemo, useState } from "react";

import {
  normalizePhase09Error,
  reportsGateway,
  type Phase09Locale,
  type ReportDescriptor,
  type ReportPage,
  type ReportRequest,
} from "../../platform/tauri/phase09";
import type { Phase09Copy } from "./copy";
import {
  EmptyState,
  OperationNotice,
  type OperationStatus,
} from "./shared";

export function ReportsPanel({
  locale,
  copy,
  canExport,
}: {
  locale: Phase09Locale;
  copy: Phase09Copy;
  canExport: boolean;
}) {
  const [reports, setReports] = useState<ReportDescriptor[]>([]);
  const [selectedId, setSelectedId] = useState<string>("SALES_SUMMARY");
  const [startDate, setStartDate] = useState("");
  const [endDate, setEndDate] = useState("");
  const [page, setPage] = useState<ReportPage | null>(null);
  const [status, setStatus] = useState<OperationStatus>({ kind: "idle" });

  useEffect(() => {
    void reportsGateway
      .list()
      .then((items) => {
        setReports(items);
        if (items[0]) {
          setSelectedId(items[0].reportId);
        }
      })
      .catch((error: unknown) => {
        const safe = normalizePhase09Error(error);
        setStatus({ kind: "error", message: safe.message, code: safe.code });
      });
  }, []);

  const selected = useMemo(
    () => reports.find((report) => report.reportId === selectedId) ?? null,
    [reports, selectedId],
  );

  const request = useCallback(
    (): ReportRequest => ({
      reportId: selectedId as ReportRequest["reportId"],
      startDate: startDate || null,
      endDate: endDate || null,
      warehouseId: null,
      partnerId: null,
      productId: null,
      status: null,
      sortField: null,
      sortDirection: "ASC",
      page: 1,
      pageSize: 50,
      locale,
    }),
    [endDate, locale, selectedId, startDate],
  );

  async function run() {
    setStatus({ kind: "loading" });
    try {
      const result = await reportsGateway.run(request());
      setPage(result);
      setStatus({ kind: "idle" });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function exportReport(format: "csv" | "pdf") {
    setStatus({ kind: "loading" });
    try {
      const result =
        format === "csv"
          ? await reportsGateway.exportCsv(request())
          : await reportsGateway.exportPdf(request());
      setStatus({ kind: "success", message: result.sha256 });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  return (
    <section className="phase09-panel" aria-labelledby="phase09-reports-title">
      <header>
        <h2 id="phase09-reports-title">{copy.reports}</h2>
      </header>
      <div className="phase09-form phase09-form--inline">
        <label className="phase09-form__grow">
          <span>{copy.reports}</span>
          <select
            value={selectedId}
            onChange={(event) => setSelectedId(event.currentTarget.value)}
          >
            {reports.map((report) => (
              <option key={report.reportId} value={report.reportId}>
                {locale === "ar-DZ" ? report.nameAr : report.nameFr}
              </option>
            ))}
          </select>
        </label>
        {selected?.supportsDateRange ? (
          <>
            <label>
              <span>{copy.startDate}</span>
              <input
                type="date"
                value={startDate}
                onChange={(event) => setStartDate(event.currentTarget.value)}
              />
            </label>
            <label>
              <span>{copy.endDate}</span>
              <input
                type="date"
                value={endDate}
                onChange={(event) => setEndDate(event.currentTarget.value)}
              />
            </label>
          </>
        ) : null}
        <div className="phase09-actions">
          <button
            type="button"
            className="phase09-button--primary"
            onClick={() => void run()}
          >
            {copy.runReport}
          </button>
          <button
            type="button"
            disabled={!canExport}
            onClick={() => void exportReport("csv")}
          >
            {copy.exportCsv}
          </button>
          <button
            type="button"
            disabled={!canExport}
            onClick={() => void exportReport("pdf")}
          >
            {copy.exportPdf}
          </button>
        </div>
      </div>
      <OperationNotice status={status} copy={copy} />
      {!page || page.rows.length === 0 ? (
        <EmptyState copy={copy} />
      ) : (
        <div className="phase09-table-wrap" tabIndex={0}>
          <table>
            <thead>
              <tr>
                {page.columns.map((column) => (
                  <th key={column.key}>
                    {locale === "ar-DZ" ? column.labelAr : column.labelFr}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {page.rows.map((row, index) => (
                <tr key={`${page.reportId}-${index}`}>
                  {page.columns.map((column) => (
                    <td key={column.key}>
                      {String(row.values[column.key] ?? "")}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
