import { invoke, isTauri } from "@tauri-apps/api/core";
import type { InvokeFunction } from "./runtime-status";

export type Phase08Command = typeof PHASE08_COMMANDS[number];
export const PHASE08_COMMANDS = [
  "install_accounting_template","list_accounts","create_account","update_account",
  "list_accounting_journals","create_accounting_journal","update_accounting_journal",
  "list_posting_rules","save_posting_rule","validate_posting_configuration",
  "list_accounting_posting_queue","post_source_event","retry_posting_attempt",
  "list_journal_entries","get_journal_entry","create_manual_journal_entry",
  "update_manual_journal_entry","post_manual_journal_entry","reverse_journal_entry",
  "post_customer_receipt","post_supplier_payment","allocate_payment",
  "reverse_payment_allocation","reverse_payment","list_payments","get_partner_statement",
  "get_cash_bank_register","get_trial_balance","get_general_ledger","get_account_ledger",
  "get_open_receivables","get_open_payables","list_fiscal_periods","close_fiscal_period",
  "reopen_fiscal_period",
] as const;

export interface EntityVersion { id:string; rowVersion:number }
export interface AccountView extends EntityVersion { code:string; nameAr:string; nameFr?:string; accountType:string; normalSide:string; allowPosting:boolean; isActive:boolean }
export interface JournalLineView { accountId:string; accountCode:string; description:string; debitMinor:number; creditMinor:number }
export interface JournalEntryView { id:string; entryNumber:string; entryDate:string; status:string; sourceEventType:string; sourceEventId:string; reversalOfEntryId?:string; memo?:string; debitTotalMinor:number; creditTotalMinor:number; lines:JournalLineView[] }
export interface TrialBalanceRow { accountId:string; accountCode:string; accountNameAr:string; debitMinor:number; creditMinor:number; balanceMinor:number }
export interface LedgerRow { journalEntryId:string; entryNumber:string; entryDate:string; accountId:string; accountCode:string; description:string; debitMinor:number; creditMinor:number; runningBalanceMinor:number }
export interface FiscalPeriodView extends EntityVersion { fiscalYearId:string; periodNumber:number; name:string; startsOn:string; endsOn:string; status:string }
export interface PostingAttemptView { id:string; sourceEventType:string; sourceEventId:string; attemptNumber:number; status:string; errorCode?:string; startedAt:string; completedAt?:string }
export interface PaymentResult { paymentId:string; journalEntryId:string; amountMinor:number; unallocatedMinor:number; replayed:boolean }
export interface AllocationResult { allocationId:string; paymentId:string; documentId:string; amountMinor:number; paymentUnallocatedMinor:number; documentOpenMinor:number }
export interface StatementRow { eventDate:string; sourceType:string; sourceId:string; debitMinor:number; creditMinor:number; runningBalanceMinor:number }
export interface OpenBalanceRow { documentId:string; documentNumber:string; documentType:string; commercialDate:string; dueDate?:string; totalMinor:number; allocatedMinor:number; openMinor:number }

const SAFE_CODES = new Set([
  "ACCOUNTING_VALIDATION","ACCOUNTING_PERMISSION_DENIED","ACCOUNTING_INTERNAL","ACCOUNTING_DISABLED",
  "ACCOUNTING_IDEMPOTENCY_CONFLICT","IDEMPOTENCY_CONFLICT","POSTING_RULE_MISSING","POSTING_RULE_AMBIGUOUS",
  "POSTING_RULE_INCOMPLETE","ACCOUNT_ROLE_MISSING","ACCOUNT_INACTIVE","ACCOUNT_NOT_POSTABLE",
  "PAYMENT_METHOD_ACCOUNTING_MISSING","FISCAL_PERIOD_NOT_FOUND","FISCAL_PERIOD_CLOSED",
  "UNBALANCED_GENERATED_ENTRY","POSTED_JOURNAL_IMMUTABLE","JOURNAL_ALREADY_REVERSED",
  "PAYMENT_METHOD_UNAVAILABLE","PAYMENT_REFERENCE_REQUIRED","OVER_ALLOCATION","PAYMENT_HAS_ALLOCATIONS",
  "PAYMENT_ALREADY_REVERSED","ALLOCATION_ALREADY_REVERSED","ALLOCATION_NOT_FOUND","PAYMENT_NOT_FOUND",
  "PAYMENT_NOT_ALLOCATABLE","PAYMENT_NOT_REVERSIBLE","PARTNER_SCOPE_MISMATCH","PARTNER_NOT_ELIGIBLE",
  "DOCUMENT_NOT_FOUND","DOCUMENT_NOT_ALLOCATABLE","ACCOUNT_CONFLICT","JOURNAL_CONFLICT",
  "POSTING_RULE_CONFLICT","POSTING_RULE_IMMUTABLE","POSTING_RULE_LINES_REQUIRED","FISCAL_PERIOD_CONFLICT","FISCAL_PERIOD_LOCKED","CONCURRENCY_CONFLICT",
  "REQUEST_HASH_MISMATCH","NEGATIVE_POSTING_COMPONENT","INJECTED_POSTING_FAILURE","ACCOUNTING_NUMERIC_OVERFLOW",
  "SOURCE_DOCUMENT_NOT_FOUND","ACCOUNT_NOT_FOUND","ACCOUNT_CONFIGURATION_REQUIRED","JOURNAL_NOT_FOUND",
  "JOURNAL_ENTRY_NOT_FOUND","JOURNAL_NOT_DRAFT","JOURNAL_NOT_REVERSIBLE","JOURNAL_UNBALANCED",
  "MANUAL_JOURNAL_LINES_REQUIRED","MANUAL_JOURNAL_LINE_INVALID","MANUAL_JOURNAL_UNBALANCED","MANUAL_JOURNAL_CONFLICT",
]);
export class Phase08GatewayError extends Error { readonly code:string; readonly retryable:boolean; constructor(code:string,retryable=false){super("The local accounting operation could not be completed.");this.name="Phase08GatewayError";this.code=code;this.retryable=retryable;} }
export function normalizePhase08Error(error:unknown):Phase08GatewayError { if(error instanceof Phase08GatewayError)return error; if(typeof error==="object"&&error!==null&&"code" in error){const code=String((error as {code:unknown}).code);const retryable="retryable" in error&&Boolean((error as {retryable:unknown}).retryable);if(SAFE_CODES.has(code))return new Phase08GatewayError(code,retryable);} return new Phase08GatewayError("ACCOUNTING_INTERNAL",true); }
function record(value:unknown):Record<string,unknown>{if(typeof value!=="object"||value===null||Array.isArray(value))throw new Phase08GatewayError("ACCOUNTING_INTERNAL");return value as Record<string,unknown>}
function array(value:unknown):unknown[]{if(!Array.isArray(value))throw new Phase08GatewayError("ACCOUNTING_INTERNAL");return value}
function text(value:unknown):string{if(typeof value!=="string"||!value.trim())throw new Phase08GatewayError("ACCOUNTING_INTERNAL");return value}
function optionalText(value:unknown):string|undefined{return value==null?undefined:text(value)}
function integer(value:unknown):number{if(typeof value!=="number"||!Number.isSafeInteger(value))throw new Phase08GatewayError("ACCOUNTING_INTERNAL");return value}
function bool(value:unknown):boolean{if(typeof value!=="boolean")throw new Phase08GatewayError("ACCOUNTING_INTERNAL");return value}
const entity=(value:unknown):EntityVersion=>{const row=record(value);return{id:text(row.id),rowVersion:integer(row.rowVersion)}};
const account=(value:unknown):AccountView=>{const row=record(value);return{...entity(value),code:text(row.code),nameAr:text(row.nameAr),nameFr:optionalText(row.nameFr),accountType:text(row.accountType),normalSide:text(row.normalSide),allowPosting:bool(row.allowPosting),isActive:bool(row.isActive)}};
const journalLine=(value:unknown):JournalLineView=>{const row=record(value);return{accountId:text(row.accountId),accountCode:text(row.accountCode),description:text(row.description),debitMinor:integer(row.debitMinor),creditMinor:integer(row.creditMinor)}};
const journal=(value:unknown):JournalEntryView=>{const row=record(value);return{id:text(row.id),entryNumber:text(row.entryNumber),entryDate:text(row.entryDate),status:text(row.status),sourceEventType:text(row.sourceEventType),sourceEventId:text(row.sourceEventId),reversalOfEntryId:optionalText(row.reversalOfEntryId),memo:optionalText(row.memo),debitTotalMinor:integer(row.debitTotalMinor),creditTotalMinor:integer(row.creditTotalMinor),lines:array(row.lines).map(journalLine)}};
const payment=(value:unknown):PaymentResult=>{const row=record(value);return{paymentId:text(row.paymentId),journalEntryId:text(row.journalEntryId),amountMinor:integer(row.amountMinor),unallocatedMinor:integer(row.unallocatedMinor),replayed:bool(row.replayed)}};
const allocation=(value:unknown):AllocationResult=>{const row=record(value);return{allocationId:text(row.allocationId),paymentId:text(row.paymentId),documentId:text(row.documentId),amountMinor:integer(row.amountMinor),paymentUnallocatedMinor:integer(row.paymentUnallocatedMinor),documentOpenMinor:integer(row.documentOpenMinor)}};
const statement=(value:unknown):StatementRow=>{const row=record(value);return{eventDate:text(row.eventDate),sourceType:text(row.sourceType),sourceId:text(row.sourceId),debitMinor:integer(row.debitMinor),creditMinor:integer(row.creditMinor),runningBalanceMinor:integer(row.runningBalanceMinor)}};
const openBalance=(value:unknown):OpenBalanceRow=>{const row=record(value);return{documentId:text(row.documentId),documentNumber:text(row.documentNumber),documentType:text(row.documentType),commercialDate:text(row.commercialDate),dueDate:optionalText(row.dueDate),totalMinor:integer(row.totalMinor),allocatedMinor:integer(row.allocatedMinor),openMinor:integer(row.openMinor)}};
function validateResponse(command:Phase08Command,value:unknown):unknown { switch(command){case"list_accounts":return array(value).map(account);case"list_journal_entries":return array(value).map(journal);case"get_journal_entry":return journal(value);case"get_trial_balance":return array(value).map(item=>{const r=record(item);return{accountId:text(r.accountId),accountCode:text(r.accountCode),accountNameAr:text(r.accountNameAr),debitMinor:integer(r.debitMinor),creditMinor:integer(r.creditMinor),balanceMinor:integer(r.balanceMinor)}});case"get_general_ledger":case"get_account_ledger":case"get_cash_bank_register":return array(value).map(item=>{const r=record(item);return{journalEntryId:text(r.journalEntryId),entryNumber:text(r.entryNumber),entryDate:text(r.entryDate),accountId:text(r.accountId),accountCode:text(r.accountCode),description:text(r.description),debitMinor:integer(r.debitMinor),creditMinor:integer(r.creditMinor),runningBalanceMinor:integer(r.runningBalanceMinor)}});case"list_fiscal_periods":return array(value).map(item=>{const r=record(item);return{...entity(item),fiscalYearId:text(r.fiscalYearId),periodNumber:integer(r.periodNumber),name:text(r.name),startsOn:text(r.startsOn),endsOn:text(r.endsOn),status:text(r.status)}});case"post_customer_receipt":case"post_supplier_payment":case"reverse_payment":return payment(value);case"allocate_payment":case"reverse_payment_allocation":return allocation(value);case"list_payments":return array(value).map(payment);case"list_accounting_posting_queue":return array(value).map(item=>{const r=record(item);return{id:text(r.id),sourceEventType:text(r.sourceEventType),sourceEventId:text(r.sourceEventId),attemptNumber:integer(r.attemptNumber),status:text(r.status),errorCode:optionalText(r.errorCode),startedAt:text(r.startedAt),completedAt:optionalText(r.completedAt)}});case"get_partner_statement":return array(value).map(statement);case"get_open_receivables":case"get_open_payables":return array(value).map(openBalance);case"validate_posting_configuration":return array(value).map(text);default:return value;}}
export interface Phase08Gateway { call<T>(command:Phase08Command,request?:unknown,signal?:AbortSignal):Promise<T> }
export function createPhase08Gateway(invoker:InvokeFunction):Phase08Gateway { const call=async <T,>(command:Phase08Command,request?:unknown,signal?:AbortSignal):Promise<T>=>{if(signal?.aborted)throw new DOMException("Aborted","AbortError");try{const raw=request===undefined?await invoker(command):await invoker(command,{request});if(signal?.aborted)throw new DOMException("Aborted","AbortError");return validateResponse(command,raw) as T;}catch(error){if(signal?.aborted)throw new DOMException("Aborted","AbortError");throw normalizePhase08Error(error);}}; return {call}; }
declare global { interface Window { __POSMAN_DEV_PHASE08_INVOKER__?:InvokeFunction } }
export function resolvePhase08Gateway():Phase08Gateway|null { if(import.meta.env.DEV&&typeof window!=="undefined"&&typeof window.__POSMAN_DEV_PHASE08_INVOKER__==="function")return createPhase08Gateway(window.__POSMAN_DEV_PHASE08_INVOKER__);return isTauri()?createPhase08Gateway(invoke):null; }
function stableJson(value: unknown): string {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  const row = value as Record<string, unknown>;
  const keys = Object.keys(row).filter(key => row[key] !== undefined && row[key] !== null).sort();
  return `{${keys.map(key => `${JSON.stringify(key)}:${stableJson(row[key])}`).join(",")}}`;
}
async function sha256(value: unknown): Promise<string> {
  const bytes = new TextEncoder().encode(stableJson(value));
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, "0")).join("");
}
export async function idempotent<T>(prefix: string, payload: T) {
  const nonce = `${Date.now()}-${crypto.randomUUID()}`;
  return {
    idempotencyKey: `${prefix}-${nonce}`,
    requestHashSha256: await sha256(payload),
    payload,
  };
}
