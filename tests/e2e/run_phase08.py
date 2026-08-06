#!/usr/bin/env python3
"""PHASE 08 accounting workflows, Axe, directionality, overflow, and screenshot evidence."""
from __future__ import annotations

import json
import os
import signal
import socket
import subprocess
import time
from pathlib import Path
from typing import Callable

from playwright.sync_api import Page, sync_playwright

ROOT = Path(__file__).resolve().parents[2]
ARTIFACT_DIR = Path(
    os.environ.get(
        "POSMAN_ARTIFACT_DIR",
        Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "phase-08-ui-evidence",
    )
)
BASE_URL = "http://127.0.0.1:1420/#phase08"
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"

MOCK = r"""
window.__POSMAN_PHASE08_CALLS__ = [];
window.__POSMAN_PHASE08_RULE_FIXED__ = false;
window.__POSMAN_PHASE08_RETRY_COUNT__ = 0;
window.__POSMAN_PHASE08_ALLOCATION_COUNT__ = 0;
window.__POSMAN_PHASE08_MANUAL_POSTED__ = false;
const entity = (id, rowVersion=1) => ({id,rowVersion});
const accounts = [
 {id:'acc-411',code:'411',nameAr:'العملاء',nameFr:'Clients',accountType:'ASSET',normalSide:'DEBIT',allowPosting:true,isActive:true,rowVersion:1},
 {id:'acc-401',code:'401',nameAr:'الموردون',nameFr:'Fournisseurs',accountType:'LIABILITY',normalSide:'CREDIT',allowPosting:true,isActive:true,rowVersion:1},
 {id:'acc-512',code:'512',nameAr:'البنك',nameFr:'Banque',accountType:'ASSET',normalSide:'DEBIT',allowPosting:true,isActive:true,rowVersion:1},
 {id:'acc-700',code:'700',nameAr:'المبيعات',nameFr:'Ventes',accountType:'REVENUE',normalSide:'CREDIT',allowPosting:true,isActive:true,rowVersion:1}
];
const saleEntry = {
 id:'je-sale-7',entryNumber:'20260806-000007',entryDate:'2026-08-06',status:'POSTED',sourceEventType:'SALES_INVOICE',sourceEventId:'invoice-7',memo:'FAC000007',debitTotalMinor:119000,creditTotalMinor:119000,
 lines:[
  {accountId:'acc-411',accountCode:'411',description:'Client FAC000007',debitMinor:119000,creditMinor:0},
  {accountId:'acc-700',accountCode:'700',description:'Vente FAC000007',debitMinor:0,creditMinor:100000},
  {accountId:'acc-4457',accountCode:'4457',description:'TVA collectée',debitMinor:0,creditMinor:19000}
 ]
};
const manualEntry = {
 id:'je-manual-1',entryNumber:'20260806-000008',entryDate:'2026-08-06',status:'POSTED',sourceEventType:'MANUAL',sourceEventId:'manual-1',memo:'Écriture manuelle contrôlée',debitTotalMinor:100000,creditTotalMinor:100000,
 lines:[
  {accountId:'acc-512',accountCode:'512',description:'Régularisation débit',debitMinor:100000,creditMinor:0},
  {accountId:'acc-700',accountCode:'700',description:'Régularisation crédit',debitMinor:0,creditMinor:100000}
 ]
};
const queue = () => {
 const purchaseSucceeded = window.__POSMAN_PHASE08_RULE_FIXED__ && window.__POSMAN_PHASE08_RETRY_COUNT__ > 1;
 return [
 {id:'attempt-sale',sourceEventType:'SALES_INVOICE',sourceEventId:'invoice-7',attemptNumber:1,status:'SUCCEEDED',startedAt:'2026-08-06T08:00:00Z',completedAt:'2026-08-06T08:00:00Z'},
 {id:'attempt-purchase',sourceEventType:'PURCHASE_INVOICE',sourceEventId:'purchase-4',attemptNumber:1,status:purchaseSucceeded?'SUCCEEDED':'FAILED',errorCode:purchaseSucceeded?undefined:'POSTING_RULE_MISSING',startedAt:'2026-08-06T09:00:00Z',completedAt:'2026-08-06T09:00:00Z'}
 ];
};
window.__POSMAN_DEV_PHASE08_INVOKER__ = async (command,args) => {
 window.__POSMAN_PHASE08_CALLS__.push({command,args});
 switch(command) {
  case 'validate_posting_configuration': return window.__POSMAN_PHASE08_RULE_FIXED__ ? [] : ['PURCHASE_INVOICE: POSTING_RULE_MISSING'];
  case 'install_accounting_template': return entity('setup-company-1',2);
  case 'list_accounts': return accounts;
  case 'create_account': return entity('acc-new',1);
  case 'list_posting_rules': return [entity('rule-sales',1), ...(window.__POSMAN_PHASE08_RULE_FIXED__?[entity('rule-purchase',1)]:[])];
  case 'save_posting_rule': window.__POSMAN_PHASE08_RULE_FIXED__=true; return entity('rule-purchase',1);
  case 'list_journal_entries': return window.__POSMAN_PHASE08_MANUAL_POSTED__ ? [saleEntry,manualEntry] : [saleEntry];
  case 'create_manual_journal_entry': return entity('manual-1',1);
  case 'post_manual_journal_entry': window.__POSMAN_PHASE08_MANUAL_POSTED__=true; return entity('manual-1',2);
  case 'reverse_journal_entry': return entity('je-manual-reversal',1);
  case 'post_customer_receipt': return {paymentId:'pay-customer-1',journalEntryId:'je-pay-customer-1',amountMinor:60000,unallocatedMinor:60000,replayed:false};
  case 'post_supplier_payment': return {paymentId:'pay-supplier-1',journalEntryId:'je-pay-supplier-1',amountMinor:60000,unallocatedMinor:60000,replayed:false};
  case 'allocate_payment': {
    window.__POSMAN_PHASE08_ALLOCATION_COUNT__ += 1;
    const remaining = window.__POSMAN_PHASE08_ALLOCATION_COUNT__ === 1 ? 40000 : 0;
    return {allocationId:`allocation-${window.__POSMAN_PHASE08_ALLOCATION_COUNT__}`,paymentId:'pay-customer-1',documentId:'invoice-7',amountMinor:window.__POSMAN_PHASE08_ALLOCATION_COUNT__===1?20000:40000,paymentUnallocatedMinor:remaining,documentOpenMinor:remaining};
  }
  case 'list_payments': return [
   {paymentId:'pay-customer-1',journalEntryId:'je-pay-customer-1',amountMinor:60000,unallocatedMinor:window.__POSMAN_PHASE08_ALLOCATION_COUNT__===0?60000:(window.__POSMAN_PHASE08_ALLOCATION_COUNT__===1?40000:0),replayed:false},
   {paymentId:'pay-supplier-1',journalEntryId:'je-pay-supplier-1',amountMinor:60000,unallocatedMinor:60000,replayed:false}
  ];
  case 'get_partner_statement': return [{eventDate:'2026-08-06',sourceType:'SALES_INVOICE',sourceId:'invoice-7',debitMinor:119000,creditMinor:60000,runningBalanceMinor:59000}];
  case 'get_open_receivables': return [{documentId:'invoice-7',documentNumber:'FAC000007',documentType:'SALES_INVOICE',commercialDate:'2026-08-06',totalMinor:60000,allocatedMinor:window.__POSMAN_PHASE08_ALLOCATION_COUNT__===0?0:(window.__POSMAN_PHASE08_ALLOCATION_COUNT__===1?20000:60000),openMinor:window.__POSMAN_PHASE08_ALLOCATION_COUNT__===0?60000:(window.__POSMAN_PHASE08_ALLOCATION_COUNT__===1?40000:0)}];
  case 'get_open_payables': return [{documentId:'purchase-4',documentNumber:'FA000004',documentType:'PURCHASE_INVOICE',commercialDate:'2026-08-06',totalMinor:80000,allocatedMinor:0,openMinor:80000}];
  case 'get_trial_balance': return [{accountId:'acc-411',accountCode:'411',accountNameAr:'العملاء',debitMinor:119000,creditMinor:60000,balanceMinor:59000},{accountId:'acc-700',accountCode:'700',accountNameAr:'المبيعات',debitMinor:0,creditMinor:100000,balanceMinor:-100000}];
  case 'get_general_ledger': return [{journalEntryId:'je-sale-7',entryNumber:'20260806-000007',entryDate:'2026-08-06',accountId:'acc-411',accountCode:'411',description:'Client FAC000007',debitMinor:119000,creditMinor:0,runningBalanceMinor:119000}];
  case 'list_fiscal_periods': return [{id:'period-2026-08',fiscalYearId:'fy-2026',periodNumber:8,name:'Août 2026',startsOn:'2026-08-01',endsOn:'2026-08-31',status:'OPEN',rowVersion:1}];
  case 'close_fiscal_period': return entity('period-2026-08',2);
  case 'reopen_fiscal_period': return entity('period-2026-08',3);
  case 'list_accounting_posting_queue': return queue();
  case 'retry_posting_attempt':
   window.__POSMAN_PHASE08_RETRY_COUNT__ += 1;
   if (!window.__POSMAN_PHASE08_RULE_FIXED__) throw {code:'POSTING_RULE_MISSING',retryable:false};
   return {journalEntryId:'je-purchase-4',postingAttemptId:`attempt-purchase-${window.__POSMAN_PHASE08_RETRY_COUNT__}`,replayed:false};
  default: return entity(`result-${command}`,1);
 }
};
"""

ERROR_CAPTURE = r"""
window.__POSMAN_E2E_ERRORS__=[];
addEventListener('error',event=>window.__POSMAN_E2E_ERRORS__.push(String(event.error||event.message)));
addEventListener('unhandledrejection',event=>window.__POSMAN_E2E_ERRORS__.push(String(event.reason)));
const original=console.error;
console.error=(...args)=>{window.__POSMAN_E2E_ERRORS__.push(args.map(String).join(' '));original(...args)};
"""


def wait_server(timeout: float = 45) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", 1420), timeout=.5):
                return
        except OSError:
            time.sleep(.25)
    raise RuntimeError("Vite did not start")


def open_workspace(page: Page, french: bool) -> None:
    page.goto(BASE_URL, wait_until="networkidle")
    page.locator(".p8-workspace").wait_for()
    page.get_by_role("button", name="Français" if french else "العربية", exact=True).click()
    expected = "ltr" if french else "rtl"
    if page.locator("html").get_attribute("dir") != expected:
        raise AssertionError(f"expected {expected} direction")


def no_overflow(page: Page, label: str) -> None:
    dims = page.evaluate("() => ({inner:innerWidth,doc:document.documentElement.scrollWidth,body:document.body.scrollWidth})")
    if max(dims["doc"], dims["body"]) > dims["inner"] + 1:
        raise AssertionError(f"{label}: page overflow {dims}")


def no_clipped_primary(page: Page, label: str) -> None:
    failures = page.locator(".p8-commandbar h1,.p8-section-title h2,.p8-rail button,.p8-actions button,.p8-period button,.p8-queue button").evaluate_all(
        """els => els.filter(el => { const r=el.getBoundingClientRect(); const s=getComputedStyle(el); if(s.display==='none') return false; return r.width<=0 || r.height<=0 || el.scrollHeight>el.clientHeight+3; }).map(el => el.textContent?.trim())"""
    )
    if failures:
        raise AssertionError(f"{label}: clipped primary labels {failures}")


def axe(page: Page, name: str) -> dict:
    page.add_script_tag(path=str(AXE_PATH))
    result = page.evaluate("async () => await axe.run(document, {runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}})")
    unresolved = [item for item in result["incomplete"] if item.get("impact") in {"critical", "serious"}]
    summary = {
        "violations": len(result["violations"]),
        "incomplete": len(result["incomplete"]),
        "unresolvedCriticalSeriousIncomplete": len(unresolved),
        "passes": len(result["passes"]),
    }
    (ARTIFACT_DIR / f"axe-{name}.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    if summary["violations"] or unresolved:
        raise AssertionError(f"{name}: axe {summary}")
    return summary


def evidence(page: Page, name: str, direction: str) -> dict:
    no_overflow(page, name)
    no_clipped_primary(page, name)
    actual_direction = page.locator("html").get_attribute("dir")
    if actual_direction != direction:
        raise AssertionError(f"{name}: direction {actual_direction}, expected {direction}")
    page.screenshot(path=str(ARTIFACT_DIR / f"{name}.png"), full_page=True)
    report = axe(page, name)
    errors = page.evaluate("() => window.__POSMAN_E2E_ERRORS__ || []")
    if errors:
        raise AssertionError(f"{name}: browser errors {errors}")
    return {
        "name": name,
        "viewport": page.viewport_size,
        "direction": actual_direction,
        "axe": report,
        "calls": page.evaluate("() => window.__POSMAN_PHASE08_CALLS__"),
    }


def section(page: Page, index: int) -> None:
    page.locator(".p8-rail button").nth(index).click()
    page.locator(".p8-canvas").wait_for()


def run_case(browser, name: str, width: int, height: int, french: bool, action: Callable[[Page], None]) -> dict:
    page = browser.new_page(viewport={"width": width, "height": height})
    page.on("dialog", lambda dialog: dialog.accept())
    page.add_init_script(MOCK)
    page.add_init_script(ERROR_CAPTURE)
    try:
        open_workspace(page, french)
        action(page)
        return evidence(page, name, "ltr" if french else "rtl")
    finally:
        page.close()


def main() -> int:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    server = subprocess.Popen(
        ["npm", "run", "dev", "--", "--host", "127.0.0.1"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    try:
        wait_server()
        reports: list[dict] = []
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch()

            def setup_and_rule(page: Page) -> None:
                page.locator("[data-testid='phase08-overview'] form").first.locator("button[type='submit']").click()
                page.get_by_role("status").filter(has_text="اكتملت").wait_for()
                section(page, 2)
                page.locator("[data-testid='phase08-rules'] form button[type='submit']").click()
                page.get_by_role("status").filter(has_text="اكتملت").wait_for()

            reports.append(run_case(browser, "ar-accounting-setup-rules-1280x800", 1280, 800, False, setup_and_rule))

            def sales_trace(page: Page) -> None:
                section(page, 3)
                page.get_by_text("SALES_INVOICE", exact=True).wait_for()
                page.get_by_text("invoice-7", exact=True).wait_for()
                entry_text = page.locator(".p8-entry").first.inner_text().replace("\u202f", "").replace(" ", "")
                if "1190,00" not in entry_text:
                    raise AssertionError(f"localized sales amount missing: {entry_text}")

            reports.append(run_case(browser, "fr-sales-source-journal-trace-1280x800", 1280, 800, True, sales_trace))

            def purchase_supplier_payment(page: Page) -> None:
                section(page, 8)
                page.get_by_text("PURCHASE_INVOICE", exact=True).wait_for()
                section(page, 4)
                supplier = page.locator("[data-testid='phase08-payments'] form").nth(1)
                supplier.locator("button[type='submit']").click()
                page.get_by_role("status").filter(has_text="اكتملت").wait_for()

            reports.append(run_case(browser, "ar-purchase-posting-supplier-payment-1280x800", 1280, 800, False, purchase_supplier_payment))

            def manual_post_reversal(page: Page) -> None:
                section(page, 3)
                form = page.locator("[data-testid='phase08-journals'] form").first
                form.locator("button[type='submit']").click()
                page.get_by_role("button", name="Comptabiliser", exact=True).click()
                page.get_by_text("MANUAL", exact=True).wait_for()
                page.locator(".p8-entry").filter(has_text="MANUAL").get_by_role("button", name="Contrepasser", exact=True).click()
                page.get_by_role("status").filter(has_text="terminée").wait_for()

            reports.append(run_case(browser, "fr-manual-journal-post-reversal-1024x640", 1024, 640, True, manual_post_reversal))

            def customer_partial_full(page: Page) -> None:
                section(page, 4)
                customer = page.locator("[data-testid='phase08-payments'] form").first
                customer.locator("button[type='submit']").click()
                page.get_by_role("status").filter(has_text="اكتملت").wait_for()
                allocation = page.locator("[data-testid='phase08-payments'] form").nth(2)
                allocation.locator("input[name='payment']").fill("pay-customer-1")
                allocation.locator("input[name='amount']").fill("200")
                allocation.locator("button[type='submit']").click()
                page.wait_for_function("() => window.__POSMAN_PHASE08_ALLOCATION_COUNT__ === 1")
                page.get_by_role("status").filter(has_text="اكتملت").wait_for()
                allocation.locator("input[name='amount']").fill("400")
                allocation.locator("button[type='submit']").click()
                page.wait_for_function("() => window.__POSMAN_PHASE08_ALLOCATION_COUNT__ === 2")
                page.get_by_role("status").filter(has_text="اكتملت").wait_for()

            reports.append(run_case(browser, "ar-customer-payment-partial-full-allocation-1024x640", 1024, 640, False, customer_partial_full))

            def repair_and_retry(page: Page) -> None:
                section(page, 8)
                page.get_by_role("button", name="Réessayer", exact=True).click()
                page.get_by_role("alert").filter(has_text="POSTING_RULE_MISSING").wait_for()
                section(page, 2)
                page.locator("[data-testid='phase08-rules'] select[name='event']").select_option("PURCHASE_INVOICE")
                page.locator("[data-testid='phase08-rules'] form button[type='submit']").click()
                page.get_by_role("status").filter(has_text="terminée").wait_for()
                section(page, 8)
                page.get_by_role("button", name="Réessayer", exact=True).click()
                page.get_by_role("status").filter(has_text="terminée").wait_for()
                page.get_by_text("SUCCEEDED", exact=True).last.wait_for()

            reports.append(run_case(browser, "fr-missing-rule-correction-retry-1280x800", 1280, 800, True, repair_and_retry))
            browser.close()

        (ARTIFACT_DIR / "phase08-e2e-summary.json").write_text(
            json.dumps(reports, ensure_ascii=False, indent=2), encoding="utf-8"
        )
        print(json.dumps(reports, ensure_ascii=False, indent=2))
        return 0
    finally:
        if server.poll() is None:
            os.killpg(server.pid, signal.SIGTERM)
            try:
                server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(server.pid, signal.SIGKILL)
        if server.stdout:
            (ARTIFACT_DIR / "phase08-vite.log").write_text(server.stdout.read(), encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
