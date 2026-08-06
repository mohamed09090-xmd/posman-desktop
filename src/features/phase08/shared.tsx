import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import {
  resolvePhase08Gateway,
  type AccountView,
  type FiscalPeriodView,
  type JournalEntryView,
  type PaymentResult,
  type PostingAttemptView,
  type TrialBalanceRow,
} from "../../platform/tauri/phase08";
import { PHASE08_COPY } from "./copy";

export type Gateway = ReturnType<typeof resolvePhase08Gateway>;
export type LoadState = "loading" | "ready" | "empty" | "error";
export type Notice = { tone: "success" | "error" | "warning"; text: string };
export type Copy = (typeof PHASE08_COPY)[keyof typeof PHASE08_COPY];
export type RuleLineDraft = { side: "DEBIT" | "CREDIT"; accountRoleCode: string; amountComponent: string; partnerDimension: boolean; productDimension: boolean };

export const RULE_TEMPLATES: Record<string, RuleLineDraft[]> = {
  SALES_INVOICE: [
    { side: "DEBIT", accountRoleCode: "CUSTOMER_RECEIVABLE", amountComponent: "DOCUMENT_TTC", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "SALES_REVENUE", amountComponent: "DOCUMENT_HT", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "COLLECTED_TAX", amountComponent: "DOCUMENT_TAX", partnerDimension: true, productDimension: false },
  ],
  PURCHASE_INVOICE: [
    { side: "DEBIT", accountRoleCode: "INVENTORY", amountComponent: "DOCUMENT_HT", partnerDimension: true, productDimension: true },
    { side: "DEBIT", accountRoleCode: "RECOVERABLE_TAX", amountComponent: "DOCUMENT_TAX", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "SUPPLIER_PAYABLE", amountComponent: "DOCUMENT_TTC", partnerDimension: true, productDimension: false },
  ],
  PURCHASE_RECEIVE_INVOICE: [
    { side: "DEBIT", accountRoleCode: "INVENTORY", amountComponent: "DOCUMENT_HT", partnerDimension: true, productDimension: true },
    { side: "DEBIT", accountRoleCode: "RECOVERABLE_TAX", amountComponent: "DOCUMENT_TAX", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "SUPPLIER_PAYABLE", amountComponent: "DOCUMENT_TTC", partnerDimension: true, productDimension: false },
  ],
  DELIVERY_COGS: [
    { side: "DEBIT", accountRoleCode: "COGS", amountComponent: "STOCK_COST", partnerDimension: false, productDimension: true },
    { side: "CREDIT", accountRoleCode: "INVENTORY", amountComponent: "STOCK_COST", partnerDimension: false, productDimension: true },
  ],
  DIRECT_SALE: [
    { side: "DEBIT", accountRoleCode: "CUSTOMER_RECEIVABLE", amountComponent: "DOCUMENT_TTC", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "SALES_REVENUE", amountComponent: "DOCUMENT_HT", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "COLLECTED_TAX", amountComponent: "DOCUMENT_TAX", partnerDimension: true, productDimension: false },
    { side: "DEBIT", accountRoleCode: "COGS", amountComponent: "STOCK_COST", partnerDimension: false, productDimension: true },
    { side: "CREDIT", accountRoleCode: "INVENTORY", amountComponent: "STOCK_COST", partnerDimension: false, productDimension: true },
  ],
  SALES_RETURN: [
    { side: "DEBIT", accountRoleCode: "SALES_REVENUE", amountComponent: "DOCUMENT_HT", partnerDimension: true, productDimension: false },
    { side: "DEBIT", accountRoleCode: "COLLECTED_TAX", amountComponent: "DOCUMENT_TAX", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "CUSTOMER_RECEIVABLE", amountComponent: "DOCUMENT_TTC", partnerDimension: true, productDimension: false },
    { side: "DEBIT", accountRoleCode: "INVENTORY", amountComponent: "STOCK_COST", partnerDimension: false, productDimension: true },
    { side: "CREDIT", accountRoleCode: "COGS", amountComponent: "STOCK_COST", partnerDimension: false, productDimension: true },
  ],
  PURCHASE_RETURN: [
    { side: "DEBIT", accountRoleCode: "SUPPLIER_PAYABLE", amountComponent: "DOCUMENT_TTC", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "INVENTORY", amountComponent: "DOCUMENT_HT", partnerDimension: true, productDimension: true },
    { side: "CREDIT", accountRoleCode: "RECOVERABLE_TAX", amountComponent: "DOCUMENT_TAX", partnerDimension: true, productDimension: false },
  ],
  CUSTOMER_RECEIPT: [
    { side: "DEBIT", accountRoleCode: "CASH", amountComponent: "PAYMENT_AMOUNT", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "CUSTOMER_RECEIVABLE", amountComponent: "PAYMENT_AMOUNT", partnerDimension: true, productDimension: false },
  ],
  SUPPLIER_PAYMENT: [
    { side: "DEBIT", accountRoleCode: "SUPPLIER_PAYABLE", amountComponent: "PAYMENT_AMOUNT", partnerDimension: true, productDimension: false },
    { side: "CREDIT", accountRoleCode: "CASH", amountComponent: "PAYMENT_AMOUNT", partnerDimension: true, productDimension: false },
  ],
};

export const TODAY = "2026-08-06";
export const str = (data: FormData, key: string) => String(data.get(key) ?? "").trim();
export const optional = (data: FormData, key: string) => str(data, key) || undefined;
export const minor = (data: FormData, key: string) => Math.round(Number(data.get(key) ?? 0) * 100);

export const previewAccounts: AccountView[] = [
  { id: "acc-411", code: "411", nameAr: "العملاء", nameFr: "Clients", accountType: "ASSET", normalSide: "DEBIT", allowPosting: true, isActive: true, rowVersion: 1 },
  { id: "acc-512", code: "512", nameAr: "البنك", nameFr: "Banque", accountType: "ASSET", normalSide: "DEBIT", allowPosting: true, isActive: true, rowVersion: 1 },
  { id: "acc-700", code: "700", nameAr: "المبيعات", nameFr: "Ventes", accountType: "REVENUE", normalSide: "CREDIT", allowPosting: true, isActive: true, rowVersion: 1 },
];
export const previewEntries: JournalEntryView[] = [{
  id: "je-sale-7", entryNumber: "20260806-000007", entryDate: TODAY, status: "POSTED",
  sourceEventType: "SALES_INVOICE", sourceEventId: "invoice-7", memo: "FAC000007",
  debitTotalMinor: 119000, creditTotalMinor: 119000,
  lines: [
    { accountId: "acc-411", accountCode: "411", description: "Client FAC000007", debitMinor: 119000, creditMinor: 0 },
    { accountId: "acc-700", accountCode: "700", description: "Vente FAC000007", debitMinor: 0, creditMinor: 100000 },
    { accountId: "acc-4457", accountCode: "4457", description: "TVA collectée", debitMinor: 0, creditMinor: 19000 },
  ],
}];
export const previewPayments: PaymentResult[] = [{ paymentId: "pay-1", journalEntryId: "je-pay-1", amountMinor: 60000, unallocatedMinor: 20000, replayed: false }];
export const previewPeriods: FiscalPeriodView[] = [{ id: "period-2026-08", fiscalYearId: "fy-2026", periodNumber: 8, name: "Août 2026", startsOn: "2026-08-01", endsOn: "2026-08-31", status: "OPEN", rowVersion: 1 }];
export const previewAttempts: PostingAttemptView[] = [
  { id: "attempt-ok", sourceEventType: "SALES_INVOICE", sourceEventId: "invoice-7", attemptNumber: 1, status: "SUCCEEDED", startedAt: "2026-08-06T08:00:00Z", completedAt: "2026-08-06T08:00:00Z" },
  { id: "attempt-failed", sourceEventType: "PURCHASE_INVOICE", sourceEventId: "purchase-4", attemptNumber: 1, status: "FAILED", errorCode: "POSTING_RULE_MISSING", startedAt: "2026-08-06T09:00:00Z", completedAt: "2026-08-06T09:00:00Z" },
];
export const previewTrial: TrialBalanceRow[] = [
  { accountId: "acc-411", accountCode: "411", accountNameAr: "العملاء", debitMinor: 119000, creditMinor: 60000, balanceMinor: 59000 },
  { accountId: "acc-700", accountCode: "700", accountNameAr: "المبيعات", debitMinor: 0, creditMinor: 100000, balanceMinor: -100000 },
];

export function Button(props: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button {...props} className={`p8-button ${props.className ?? ""}`.trim()} />;
}
export function Field({ label, children, wide = false }: { label: string; children: ReactNode; wide?: boolean }) {
  return <label className={`p8-field${wide ? " p8-field--wide" : ""}`}><span>{label}</span>{children}</label>;
}
export function StateBlock({ state, retry, children, c }: { state: LoadState; retry: () => void; children: ReactNode; c: Copy }) {
  if (state === "loading") return <div className="p8-state" role="status">{c.loading}</div>;
  if (state === "empty") return <div className="p8-state">{c.empty}</div>;
  if (state === "error") return <div className="p8-state" role="alert"><span>{c.error}</span><Button type="button" onClick={retry}>{c.retry}</Button></div>;
  return <>{children}</>;
}

export function useResource<T>(loader: (signal: AbortSignal) => Promise<T>, fallback: T) {
  const [value, setValue] = useState<T>(fallback);
  const [state, setState] = useState<LoadState>("loading");
  const request = useRef(0);
  const load = useCallback(() => {
    const token = ++request.current;
    const controller = new AbortController();
    setState("loading");
    void loader(controller.signal).then(next => {
      if (request.current !== token || controller.signal.aborted) return;
      setValue(next);
      setState(Array.isArray(next) && next.length === 0 ? "empty" : "ready");
    }).catch(error => {
      if (request.current !== token || controller.signal.aborted || (error instanceof DOMException && error.name === "AbortError")) return;
      setState("error");
    });
    return () => controller.abort();
  }, [loader]);
  useEffect(() => load(), [load]);
  return { value, state, reload: load, setValue };
}


export function Table({ headers, rows }: { headers: string[]; rows: Array<Array<string | number>> }) {
  return <div className="p8-table-scroll"><table className="p8-table"><thead><tr>{headers.map(header => <th key={header}>{header}</th>)}</tr></thead><tbody>{rows.map((row, index) => <tr key={index}>{row.map((cell, cellIndex) => <td key={cellIndex}>{cell}</td>)}</tr>)}</tbody></table></div>;
}
