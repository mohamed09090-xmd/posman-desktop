import { useCallback } from "react";
import { Phase08GatewayError, idempotent, type EntityVersion, type FiscalPeriodView, type PostingAttemptView } from "../../../platform/tauri/phase08";
import {
  Button, StateBlock, TODAY, previewAttempts, previewPeriods, useResource,
  type Copy, type Gateway, type Notice,
} from "../shared";

function errorText(error: unknown) {
  return error instanceof Phase08GatewayError ? error.code : "ACCOUNTING_INTERNAL";
}

export function Periods({ gateway, c, setNotice, formatDate }: { gateway: Gateway; c: Copy; setNotice: (notice: Notice) => void; formatDate: (s: string) => string }) {
  const resource = useResource(useCallback(signal => gateway ? gateway.call<FiscalPeriodView[]>("list_fiscal_periods", undefined, signal) : Promise.resolve(previewPeriods), [gateway]), previewPeriods);
  const change = async (period: FiscalPeriodView, command: "close_fiscal_period" | "reopen_fiscal_period") => {
    if (!window.confirm(c.confirmPeriod)) return;
    try { if (gateway) await gateway.call<EntityVersion>(command, { fiscalPeriodId: period.id, rowVersion: period.rowVersion, reason: "Contrôle de période approuvé" }); setNotice({ tone: "success", text: c.success }); resource.reload(); }
    catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  return <StateBlock state={resource.state} retry={resource.reload} c={c}><div className="p8-periods">{resource.value.map(period => <article key={period.id} className="p8-period"><div><strong>{period.name}</strong><span>{formatDate(period.startsOn)} — {formatDate(period.endsOn)}</span></div><span className="p8-stamp">{period.status}</span><Button type="button" onClick={() => void change(period, period.status === "OPEN" ? "close_fiscal_period" : "reopen_fiscal_period")}>{period.status === "OPEN" ? c.close : c.reopen}</Button></article>)}</div></StateBlock>;
}

export function Queue({ gateway, c, setNotice }: { gateway: Gateway; c: Copy; setNotice: (notice: Notice) => void }) {
  const resource = useResource(useCallback(signal => gateway ? gateway.call<PostingAttemptView[]>("list_accounting_posting_queue", undefined, signal) : Promise.resolve(previewAttempts), [gateway]), previewAttempts);
  const retry = async (attempt: PostingAttemptView) => {
    if (!window.confirm(c.confirmPost)) return;
    try {
      const payload = { sourceEventType: attempt.sourceEventType, sourceEventId: attempt.sourceEventId, sourceDocumentId: attempt.sourceEventId, eventDate: TODAY, componentsMinor: { DOCUMENT_HT: 100000, DOCUMENT_TAX: 19000, DOCUMENT_TTC: 119000 } };
      if (gateway) await gateway.call("retry_posting_attempt", await idempotent("retry", payload));
      setNotice({ tone: "success", text: c.success }); resource.reload();
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  return <StateBlock state={resource.state} retry={resource.reload} c={c}><div className="p8-queue"><p className="p8-safe">{c.failedSafe}</p>{resource.value.map(attempt => <article key={attempt.id}><div><strong>{attempt.sourceEventType}</strong><span>{attempt.sourceEventId} · #{attempt.attemptNumber}</span></div><span className={`p8-stamp p8-stamp--${attempt.status.toLowerCase()}`}>{attempt.status}</span><code>{attempt.errorCode ?? "—"}</code>{attempt.status === "FAILED" ? <Button type="button" onClick={() => void retry(attempt)}>{c.retry}</Button> : null}</article>)}</div></StateBlock>;
}
