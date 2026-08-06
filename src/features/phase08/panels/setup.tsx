import { useCallback, useState, type FormEvent } from "react";
import { Phase08GatewayError, type AccountView, type EntityVersion } from "../../../platform/tauri/phase08";
import {
  Button, Field, RULE_TEMPLATES, StateBlock, Table, optional, previewAccounts, str, useResource,
  type Copy, type Gateway, type Notice, type RuleLineDraft,
} from "../shared";

function errorText(error: unknown) {
  return error instanceof Phase08GatewayError ? error.code : "ACCOUNTING_INTERNAL";
}

export function Overview({ gateway, c, setNotice }: { gateway: Gateway; c: Copy; setNotice: (notice: Notice) => void }) {
  const issues = useResource(useCallback(signal => gateway ? gateway.call<string[]>("validate_posting_configuration", undefined, signal) : Promise.resolve([]), [gateway]), []);
  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      const roles = [
        ["CUSTOMER_RECEIVABLE", str(form, "receivable")], ["SUPPLIER_PAYABLE", str(form, "payable")],
        ["CASH", str(form, "cash")], ["BANK", str(form, "bank")], ["SALES_REVENUE", str(form, "sales")],
        ["COLLECTED_TAX", str(form, "outputTax")], ["RECOVERABLE_TAX", str(form, "inputTax")],
        ["INVENTORY", str(form, "inventory")], ["COGS", str(form, "cogs")],
      ].filter((item): item is [string, string] => Boolean(item[1])).map(([roleCode, accountId]) => ({ roleCode, accountId }));
      const paymentMethods = [
        { paymentMethodId: str(form, "cashMethod"), accountRoleCode: "CASH" },
        { paymentMethodId: str(form, "bankMethod"), accountRoleCode: "BANK" },
      ].filter(item => Boolean(item.paymentMethodId));
      if (gateway) await gateway.call<EntityVersion>("install_accounting_template", { enabled: true, currentFiscalYearId: optional(form, "year"), roles, paymentMethods });
      setNotice({ tone: "success", text: c.success }); issues.reload();
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  return <div className="p8-split">
    <section className="p8-paper"><h3>{c.configureTemplate}</h3><form className="p8-form" onSubmit={event => void submit(event)}>
      <Field label="Fiscal year"><input name="year" defaultValue="fy-2026" /></Field>
      <Field label="Customer receivable"><input name="receivable" defaultValue="acc-411" required /></Field>
      <Field label="Supplier payable"><input name="payable" defaultValue="acc-401" required /></Field>
      <Field label="Cash"><input name="cash" defaultValue="acc-530" required /></Field>
      <Field label="Bank"><input name="bank" defaultValue="acc-512" required /></Field>
      <Field label="Sales revenue"><input name="sales" defaultValue="acc-700" required /></Field>
      <Field label="Output tax"><input name="outputTax" defaultValue="acc-4457" required /></Field>
      <Field label="Input tax"><input name="inputTax" defaultValue="acc-4456" required /></Field>
      <Field label="Inventory"><input name="inventory" defaultValue="acc-310" required /></Field>
      <Field label="COGS"><input name="cogs" defaultValue="acc-600" required /></Field>
      <Field label={`${c.paymentMapping} · Cash`}><input name="cashMethod" defaultValue="method-cash" required /></Field>
      <Field label={`${c.paymentMapping} · Bank`}><input name="bankMethod" defaultValue="method-bank" required /></Field>
      <div className="p8-actions"><Button type="submit">{c.save}</Button></div>
    </form></section>
    <section className="p8-paper"><h3>{c.setupIssues}</h3><StateBlock state={issues.state} retry={issues.reload} c={c}>{issues.value.length === 0 ? <p className="p8-ok">{c.noIssues}</p> : <ul className="p8-issues">{issues.value.map(issue => <li key={issue}>{issue}</li>)}</ul>}</StateBlock>
      <div className="p8-atomic"><strong>BEGIN IMMEDIATE</strong><span>source → stock → accounting → audit → idempotency → commit</span></div>
    </section>
  </div>;
}

export function Accounts({ gateway, c, setNotice }: { gateway: Gateway; c: Copy; setNotice: (notice: Notice) => void }) {
  const resource = useResource(useCallback(signal => gateway ? gateway.call<AccountView[]>("list_accounts", undefined, signal) : Promise.resolve(previewAccounts), [gateway]), previewAccounts);
  const journals = useResource(useCallback(signal => gateway ? gateway.call<EntityVersion[]>("list_accounting_journals", undefined, signal) : Promise.resolve([{ id: "journal-general", rowVersion: 1 }]), [gateway]), [{ id: "journal-general", rowVersion: 1 }]);
  const [busy, setBusy] = useState(false);
  const createAccount = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (busy) return; setBusy(true);
    const form = new FormData(event.currentTarget);
    try {
      const input = { code: str(form, "code"), nameAr: str(form, "nameAr"), nameFr: optional(form, "nameFr"), accountType: str(form, "type"), normalSide: str(form, "side"), allowPosting: true, isActive: true };
      if (gateway) await gateway.call<EntityVersion>("create_account", input);
      else resource.setValue(current => [...current, { id: crypto.randomUUID(), ...input, rowVersion: 1 }]);
      event.currentTarget.reset(); setNotice({ tone: "success", text: c.success }); resource.reload();
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); } finally { setBusy(false); }
  };
  const createJournal = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); const form = new FormData(event.currentTarget);
    try {
      if (gateway) await gateway.call<EntityVersion>("create_accounting_journal", { code: str(form, "code"), nameAr: str(form, "nameAr"), nameFr: optional(form, "nameFr"), journalType: str(form, "type"), isActive: true });
      setNotice({ tone: "success", text: c.success }); journals.reload();
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  return <div className="p8-stack"><div className="p8-split"><form className="p8-form p8-paper" onSubmit={event => void createAccount(event)}><h3>{c.accounts}</h3>
    <Field label={c.accountCode}><input name="code" required /></Field><Field label={c.arabicName}><input name="nameAr" required /></Field><Field label={c.frenchName}><input name="nameFr" /></Field>
    <Field label={c.accountType}><select name="type" defaultValue="ASSET"><option>ASSET</option><option>LIABILITY</option><option>EQUITY</option><option>REVENUE</option><option>EXPENSE</option></select></Field>
    <Field label={c.normalSide}><select name="side" defaultValue="DEBIT"><option>DEBIT</option><option>CREDIT</option></select></Field>
    <div className="p8-actions"><Button disabled={busy} type="submit">{c.create}</Button></div>
  </form><form className="p8-form p8-paper" onSubmit={event => void createJournal(event)}><h3>{c.journalSetup}</h3>
    <Field label={c.journalCode}><input name="code" defaultValue="OD" required /></Field><Field label={c.arabicName}><input name="nameAr" defaultValue="عمليات متنوعة" required /></Field><Field label={c.frenchName}><input name="nameFr" defaultValue="Opérations diverses" /></Field><Field label={c.accountType}><select name="type" defaultValue="GENERAL"><option>GENERAL</option><option>SALES</option><option>PURCHASE</option><option>CASH</option><option>BANK</option></select></Field><div className="p8-actions"><Button type="submit">{c.create}</Button></div>
  </form></div><StateBlock state={resource.state} retry={resource.reload} c={c}><Table headers={[c.accountCode, c.arabicName, c.frenchName, c.accountType, c.status]} rows={resource.value.map(account => [account.code, account.nameAr, account.nameFr ?? "—", account.accountType, account.isActive ? "ACTIVE" : "INACTIVE"])} /></StateBlock><StateBlock state={journals.state} retry={journals.reload} c={c}><Table headers={[c.journalCode, c.status]} rows={journals.value.map(journal => [journal.id, `v${journal.rowVersion}`])} /></StateBlock></div>;
}

export function Rules({ gateway, c, setNotice }: { gateway: Gateway; c: Copy; setNotice: (notice: Notice) => void }) {
  const rules = useResource(useCallback(signal => gateway ? gateway.call<EntityVersion[]>("list_posting_rules", undefined, signal) : Promise.resolve([{ id: "rule-sales", rowVersion: 1 }]), [gateway]), [{ id: "rule-sales", rowVersion: 1 }]);
  const [eventType, setEventType] = useState("SALES_INVOICE");
  const [lines, setLines] = useState<RuleLineDraft[]>(RULE_TEMPLATES.SALES_INVOICE.map(line => ({ ...line })));
  const selectTemplate = (next: string) => { setEventType(next); setLines((RULE_TEMPLATES[next] ?? RULE_TEMPLATES.SALES_INVOICE).map(line => ({ ...line }))); };
  const updateLine = (index: number, patch: Partial<RuleLineDraft>) => setLines(current => current.map((line, lineIndex) => lineIndex === index ? { ...line, ...patch } : line));
  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); const form = new FormData(event.currentTarget);
    try {
      if (gateway) await gateway.call<EntityVersion>("save_posting_rule", {
        code: str(form, "code"), sourceEventType: eventType, accountingJournalId: str(form, "journal"), priority: 100,
        validFrom: str(form, "date"), isActive: true,
        lines: lines.map((line, index) => ({ lineNumber: index + 1, side: line.side, accountRoleCode: line.accountRoleCode, amountComponent: line.amountComponent, descriptionAr: `${eventType} ${line.side.toLowerCase()} ${index + 1}`, partnerDimension: line.partnerDimension, productDimension: line.productDimension })),
      });
      setNotice({ tone: "success", text: c.success }); rules.reload();
    } catch (error) { setNotice({ tone: "error", text: `${c.error} (${errorText(error)})` }); }
  };
  return <div className="p8-split"><form className="p8-form p8-paper" onSubmit={event => void submit(event)}><h3>{c.rules}</h3>
    <Field label="Rule code"><input name="code" defaultValue="RULE-POST" required /></Field><Field label={c.eventType}><select name="event" value={eventType} onChange={event => selectTemplate(event.target.value)}>{Object.keys(RULE_TEMPLATES).map(name => <option key={name}>{name}</option>)}</select></Field>
    <Field label={c.journalCode}><input name="journal" defaultValue="journal-general" required /></Field><Field label={c.date}><input type="date" name="date" defaultValue="2026-01-01" required /></Field>
    <div className="p8-rule-editor">{lines.map((line, index) => <fieldset key={`${eventType}-${index}`}><legend>{index + 1}</legend><select aria-label={`Side ${index + 1}`} value={line.side} onChange={event => updateLine(index, { side: event.target.value as RuleLineDraft["side"] })}><option>DEBIT</option><option>CREDIT</option></select><input aria-label={`Role ${index + 1}`} value={line.accountRoleCode} onChange={event => updateLine(index, { accountRoleCode: event.target.value })} /><select aria-label={`Component ${index + 1}`} value={line.amountComponent} onChange={event => updateLine(index, { amountComponent: event.target.value })}><option>DOCUMENT_HT</option><option>DOCUMENT_TAX</option><option>DOCUMENT_TTC</option><option>STOCK_COST</option><option>PAYMENT_AMOUNT</option></select><label><input type="checkbox" checked={line.partnerDimension} onChange={event => updateLine(index, { partnerDimension: event.target.checked })} />Partner</label><label><input type="checkbox" checked={line.productDimension} onChange={event => updateLine(index, { productDimension: event.target.checked })} />Product</label><Button type="button" disabled={lines.length <= 2} onClick={() => setLines(current => current.filter((_, lineIndex) => lineIndex !== index))}>{c.removeLine}</Button></fieldset>)}</div>
    <div className="p8-actions"><Button type="button" onClick={() => setLines(current => [...current, { side: "DEBIT", accountRoleCode: "", amountComponent: "DOCUMENT_TTC", partnerDimension: false, productDimension: false }])}>{c.addLine}</Button><Button type="submit">{c.save}</Button></div></form>
    <section className="p8-paper"><h3>{c.sourceTrace}</h3><StateBlock state={rules.state} retry={rules.reload} c={c}><ol className="p8-rule-list">{rules.value.map(rule => <li key={rule.id}><code>{rule.id}</code><span>v{rule.rowVersion}</span></li>)}</ol></StateBlock></section></div>;
}
