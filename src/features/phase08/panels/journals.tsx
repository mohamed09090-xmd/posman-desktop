import { useCallback, useState, type FormEvent } from "react";
import { Phase08GatewayError, type EntityVersion, type JournalEntryView } from "../../../platform/tauri/phase08";
import {
  Button, Field, StateBlock, TODAY, minor, optional, previewEntries, str, useResource,
  type Copy, type Gateway, type Notice,
} from "../shared";

function errorText(error: unknown) {
  return error instanceof Phase08GatewayError ? error.code : "ACCOUNTING_INTERNAL";
}

export function Journals({ gateway, c, setNotice, formatMoney, formatDate }: { gateway: Gateway; c: Copy; setNotice: (notice: Notice) => void; formatMoney: (n: number) => string; formatDate: (s: string) => string }) {
  const entries = useResource(useCallback(signal => gateway ? gateway.call<JournalEntryView[]>("list_journal_entries", undefined, signal) : Promise.resolve(previewEntries), [gateway]), previewEntries);
  const [draft, setDraft] = useState<EntityVersion>();
  const saveDraft = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); const form = new FormData(event.currentTarget);
    try {
      const input = { accountingJournalId: str(form, "journal"), entryDate: str(form, "date"), memo: optional(form, "memo"), lines: [
        { accountId: str(form, "debitAccount"), description: str(form, "debitDescription"), debitMinor: minor(form, "amount"), creditMinor: 0 },
        { accountId: str(form, "creditAccount"), description: str(form, "creditDescription"), debitMinor: 0, creditMinor: minor(form, "amount") },
      ] };
      const request = draft ? { ...input, id: draft.id, rowVersion: draft.rowVersion } : input;
      const result = gateway ? await gateway.call<EntityVersion>(draft ? "update_manual_journal_entry" : "create_manual_journal_entry", request) : { id: draft?.id ?? "manual-preview", rowVersion: (draft?.rowVersion ?? 0) + 1 };
      setDraft(result); setNotice({ tone: "success", text: c.success });
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  const postDraft = async () => {
    if (!draft || !window.confirm(c.confirmPost)) return;
    try { if (gateway) await gateway.call<EntityVersion>("post_manual_journal_entry", draft.id); setNotice({ tone: "success", text: c.success }); setDraft(undefined); entries.reload(); }
    catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  const reverse = async (entry: JournalEntryView) => {
    if (!window.confirm(c.confirmReverse)) return;
    try { if (gateway) await gateway.call<EntityVersion>("reverse_journal_entry", { journalEntryId: entry.id, reversalDate: TODAY, reason: "Correction validée" }); setNotice({ tone: "success", text: c.success }); entries.reload(); }
    catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  return <div className="p8-stack"><form className="p8-form p8-paper" onSubmit={event => void saveDraft(event)}><h3>{c.draftManual}</h3>
    <Field label={c.journalCode}><input name="journal" defaultValue="journal-general" required /></Field><Field label={c.date}><input type="date" name="date" defaultValue={TODAY} required /></Field>
    <Field label="Debit account"><input name="debitAccount" defaultValue="acc-512" required /></Field><Field label="Debit description"><input name="debitDescription" defaultValue="Régularisation débit" required /></Field>
    <Field label="Credit account"><input name="creditAccount" defaultValue="acc-700" required /></Field><Field label="Credit description"><input name="creditDescription" defaultValue="Régularisation crédit" required /></Field>
    <Field label={c.amount}><input type="number" name="amount" min="0.01" step="0.01" defaultValue="1000" required /></Field><Field label="Memo"><input name="memo" defaultValue="Écriture manuelle contrôlée" /></Field>
    <div className="p8-actions"><Button type="submit">{draft ? c.updateDraft : c.save}</Button><Button type="button" disabled={!draft} onClick={() => void postDraft()}>{c.post}</Button>{draft ? <code>{draft.id}</code> : null}</div>
  </form><StateBlock state={entries.state} retry={entries.reload} c={c}><div className="p8-entry-list">{entries.value.map(entry => <article className="p8-entry" key={entry.id}><header><div><strong>{entry.entryNumber}</strong><span>{formatDate(entry.entryDate)}</span><code>{entry.sourceEventType}</code><code>{entry.sourceEventId}</code></div><span className="p8-stamp">{entry.status}</span></header><div className="p8-table-scroll"><table><thead><tr><th>{c.account}</th><th>{c.debit}</th><th>{c.credit}</th></tr></thead><tbody>{entry.lines.map((line, index) => <tr key={`${entry.id}-${index}`}><td>{line.accountCode} · {line.description}</td><td>{line.debitMinor ? formatMoney(line.debitMinor) : "—"}</td><td>{line.creditMinor ? formatMoney(line.creditMinor) : "—"}</td></tr>)}</tbody></table></div><footer><span>{c.debit}: {formatMoney(entry.debitTotalMinor)} · {c.credit}: {formatMoney(entry.creditTotalMinor)}</span><Button type="button" onClick={() => void reverse(entry)}>{c.reverse}</Button></footer></article>)}</div></StateBlock></div>;
}
