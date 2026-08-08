import { useCallback, useEffect, useState, type FormEvent } from "react";

import {
  documentsGateway,
  normalizePhase09Error,
  type Phase09Locale,
  type RenderedDocumentView,
} from "../../platform/tauri/phase09";
import type { Phase09Copy } from "./copy";
import {
  EmptyState,
  IntegrityBadge,
  OperationNotice,
  type OperationStatus,
} from "./shared";

const DOCUMENT_TYPES = [
  "SALES_ORDER",
  "DELIVERY_NOTE",
  "SALES_INVOICE",
  "SALES_CREDIT_NOTE",
  "PURCHASE_ORDER",
  "GOODS_RECEIPT",
  "SUPPLIER_INVOICE",
  "PURCHASE_RETURN",
  "CUSTOMER_RECEIPT",
  "SUPPLIER_PAYMENT",
] as const;

export function DocumentsPanel({
  locale,
  copy,
  canRender,
  canExport,
  canPrint,
}: {
  locale: Phase09Locale;
  copy: Phase09Copy;
  canRender: boolean;
  canExport: boolean;
  canPrint: boolean;
}) {
  const [documentType, setDocumentType] = useState<string>("SALES_INVOICE");
  const [sourceDocumentId, setSourceDocumentId] = useState("");
  const [documents, setDocuments] = useState<RenderedDocumentView[]>([]);
  const [status, setStatus] = useState<OperationStatus>({ kind: "idle" });

  const reload = useCallback(async () => {
    try {
      const page = await documentsGateway.list({
        page: 1,
        pageSize: 50,
        documentType: null,
        sourceDocumentId: null,
      });
      setDocuments(page.items);
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  async function runAction(action: "preview" | "render") {
    if (!sourceDocumentId.trim()) {
      setStatus({
        kind: "error",
        message: `${copy.sourceId}: ${copy.empty}`,
        code: "VALIDATION_ERROR",
      });
      return;
    }
    setStatus({ kind: "loading" });
    try {
      if (action === "preview") {
        const preview = await documentsGateway.preview({
          documentType,
          sourceDocumentId: sourceDocumentId.trim(),
          locale,
        });
        setStatus({ kind: "success", message: preview.integrityState });
      } else {
        const rendered = await documentsGateway.render({
          documentType,
          sourceDocumentId: sourceDocumentId.trim(),
          locale,
        });
        setStatus({ kind: "success", message: rendered.pdfSha256 });
        await reload();
      }
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function withRender(
    renderId: string,
    action: "verify" | "export" | "print",
  ) {
    setStatus({ kind: "loading" });
    try {
      if (action === "verify") {
        const verified = await documentsGateway.verify({ renderId });
        setStatus({ kind: "success", message: verified.integrityState });
      } else if (action === "export") {
        const result = await documentsGateway.exportPdf({ renderId });
        setStatus({ kind: "success", message: result.sha256 });
      } else {
        await documentsGateway.print({ renderId });
        setStatus({ kind: "success", message: copy.print });
      }
      await reload();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void runAction("preview");
  }

  return (
    <section className="phase09-panel" aria-labelledby="phase09-documents-title">
      <header>
        <h2 id="phase09-documents-title">{copy.documents}</h2>
      </header>
      <form className="phase09-form phase09-form--inline" onSubmit={submit}>
        <label>
          <span>{copy.documentType}</span>
          <select
            value={documentType}
            onChange={(event) => setDocumentType(event.currentTarget.value)}
          >
            {DOCUMENT_TYPES.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
        <label className="phase09-form__grow">
          <span>{copy.sourceId}</span>
          <input
            value={sourceDocumentId}
            onChange={(event) => setSourceDocumentId(event.currentTarget.value)}
            autoComplete="off"
          />
        </label>
        <div className="phase09-actions">
          <button type="submit" disabled={!canRender}>
            {copy.preview}
          </button>
          <button
            type="button"
            className="phase09-button--primary"
            disabled={!canRender}
            onClick={() => void runAction("render")}
          >
            {copy.render}
          </button>
        </div>
      </form>
      <OperationNotice status={status} copy={copy} />
      {documents.length === 0 ? (
        <EmptyState copy={copy} />
      ) : (
        <div className="phase09-table-wrap" tabIndex={0}>
          <table>
            <thead>
              <tr>
                <th>{copy.documentType}</th>
                <th>{copy.sourceId}</th>
                <th>{copy.state}</th>
                <th>{copy.hash}</th>
                <th aria-label="Actions" />
              </tr>
            </thead>
            <tbody>
              {documents.map((document) => (
                <tr key={document.renderId}>
                  <td>{document.documentType}</td>
                  <td>{document.sourceDocumentNumber}</td>
                  <td>
                    <IntegrityBadge state={document.integrityState} />
                  </td>
                  <td>
                    <code title={document.pdfSha256}>
                      {document.pdfSha256.slice(0, 12)}…
                    </code>
                  </td>
                  <td>
                    <div className="phase09-row-actions">
                      <button
                        type="button"
                        onClick={() => void withRender(document.renderId, "verify")}
                      >
                        {copy.verify}
                      </button>
                      <button
                        type="button"
                        disabled={!canExport}
                        onClick={() => void withRender(document.renderId, "export")}
                      >
                        {copy.export}
                      </button>
                      <button
                        type="button"
                        disabled={!canPrint}
                        onClick={() => void withRender(document.renderId, "print")}
                      >
                        {copy.print}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <iframe title="Document Preview" srcDoc="" sandbox="" style={{ display: "none" }} />
    </section>
  );
}
