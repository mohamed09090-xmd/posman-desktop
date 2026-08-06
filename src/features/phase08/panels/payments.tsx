import { useCallback, useState, type FormEvent } from "react";
import { Phase08GatewayError, idempotent, type AllocationResult, type PaymentResult } from "../../../platform/tauri/phase08";
import {
  Button, Field, StateBlock, TODAY, Table, minor, optional, previewPayments, str, useResource,
  type Copy, type Gateway, type Notice,
} from "../shared";

function errorText(error: unknown) {
  return error instanceof Phase08GatewayError ? error.code : "ACCOUNTING_INTERNAL";
}

export function Payments({ gateway, c, setNotice, formatMoney }: { gateway: Gateway; c: Copy; setNotice: (notice: Notice) => void; formatMoney: (n: number) => string }) {
  const resource = useResource(useCallback(signal => gateway ? gateway.call<PaymentResult[]>("list_payments", undefined, signal) : Promise.resolve(previewPayments), [gateway]), previewPayments);
  const [lastPayment, setLastPayment] = useState<PaymentResult>();
  const [lastAllocation, setLastAllocation] = useState<AllocationResult>();
  const submitPayment = async (event: FormEvent<HTMLFormElement>, command: "post_customer_receipt" | "post_supplier_payment") => {
    event.preventDefault(); if (!window.confirm(c.confirmPost)) return; const form = new FormData(event.currentTarget);
    try {
      const payload = { partnerId: str(form, "partner"), paymentMethodId: str(form, "method"), commercialDate: str(form, "date"), amountMinor: minor(form, "amount"), externalReference: optional(form, "reference"), notes: optional(form, "notes") };
      const result = gateway ? await gateway.call<PaymentResult>(command, await idempotent(command, payload)) : { paymentId: crypto.randomUUID(), journalEntryId: crypto.randomUUID(), amountMinor: payload.amountMinor, unallocatedMinor: payload.amountMinor, replayed: false };
      setLastPayment(result); setLastAllocation(undefined); setNotice({ tone: "success", text: c.success }); resource.reload();
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  const allocate = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); const form = new FormData(event.currentTarget); const paymentId = str(form, "payment") || lastPayment?.paymentId;
    if (!paymentId) return;
    try {
      const payload = { paymentId, documentId: str(form, "document"), amountMinor: minor(form, "amount") };
      const result = gateway ? await gateway.call<AllocationResult>("allocate_payment", await idempotent("allocation", payload)) : { allocationId: crypto.randomUUID(), paymentId, documentId: payload.documentId, amountMinor: payload.amountMinor, paymentUnallocatedMinor: 0, documentOpenMinor: 0 };
      setLastAllocation(result); setNotice({ tone: "success", text: `${c.success} ${c.unallocated}: ${formatMoney(result.paymentUnallocatedMinor)}` }); resource.reload();
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  const reverseAllocation = async () => {
    if (!lastAllocation || !window.confirm(c.confirmReverse)) return;
    try { if (gateway) await gateway.call<AllocationResult>("reverse_payment_allocation", await idempotent("allocation-reversal", { allocationId: lastAllocation.allocationId, reason: "Correction validée" })); setLastAllocation(undefined); setNotice({ tone: "success", text: c.success }); resource.reload(); }
    catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  const reversePayment = async () => {
    if (!lastPayment || !window.confirm(c.confirmReverse)) return;
    try { if (gateway) await gateway.call<PaymentResult>("reverse_payment", await idempotent("payment-reversal", { paymentId: lastPayment.paymentId, reversalDate: TODAY, reason: "Correction validée" })); setLastPayment(undefined); setLastAllocation(undefined); setNotice({ tone: "success", text: c.success }); resource.reload(); }
    catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  return <div className="p8-stack"><div className="p8-split"><PaymentForm title={c.customerReceipt} c={c} onSubmit={event => void submitPayment(event, "post_customer_receipt")} defaultPartner="customer-1" /><PaymentForm title={c.supplierPayment} c={c} onSubmit={event => void submitPayment(event, "post_supplier_payment")} defaultPartner="supplier-1" /></div>
    <form className="p8-form p8-paper" onSubmit={event => void allocate(event)}><h3>{c.partialAllocation}</h3><Field label="Payment id"><input name="payment" value={lastPayment?.paymentId ?? "pay-1"} onChange={() => undefined} required /></Field><Field label={c.document}><input name="document" defaultValue="invoice-7" required /></Field><Field label={c.amount}><input name="amount" type="number" min="0.01" step="0.01" defaultValue="200" required /></Field><div className="p8-actions"><Button type="submit">{c.allocate}</Button><Button type="button" disabled={!lastAllocation} onClick={() => void reverseAllocation()}>{c.reverseAllocation}</Button><Button type="button" disabled={!lastPayment || Boolean(lastAllocation)} onClick={() => void reversePayment()}>{c.reversePayment}</Button></div></form>
    <StateBlock state={resource.state} retry={resource.reload} c={c}><Table headers={["Payment", c.amount, c.unallocated, "Journal"]} rows={resource.value.map(payment => [payment.paymentId, formatMoney(payment.amountMinor), formatMoney(payment.unallocatedMinor), payment.journalEntryId])} /></StateBlock>
  </div>;
}

function PaymentForm({ title, c, onSubmit, defaultPartner }: { title: string; c: Copy; onSubmit: (event: FormEvent<HTMLFormElement>) => void; defaultPartner: string }) {
  return <form className="p8-form p8-paper" onSubmit={onSubmit}><h3>{title}</h3><Field label={c.partner}><input name="partner" defaultValue={defaultPartner} required /></Field><Field label={c.paymentMethod}><input name="method" defaultValue="method-cash" required /></Field><Field label={c.date}><input type="date" name="date" defaultValue={TODAY} required /></Field><Field label={c.amount}><input name="amount" type="number" min="0.01" step="0.01" defaultValue="600" required /></Field><Field label={c.reference}><input name="reference" /></Field><Field label="Notes"><input name="notes" /></Field><div className="p8-actions"><Button type="submit">{c.post}</Button></div></form>;
}
