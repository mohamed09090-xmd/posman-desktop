#!/usr/bin/env python3
"""PHASE 06 workflow, accessibility, overflow, clipping, and screenshot evidence."""
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
ARTIFACT_DIR = Path(os.environ.get("POSMAN_ARTIFACT_DIR", Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "phase-06-ui-evidence"))
BASE_URL = "http://127.0.0.1:1420"
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"

MOCK = r"""
window.__POSMAN_PHASE06_CALLS__ = [];
window.__POSMAN_PHASE06_REBUILT__ = false;
window.__POSMAN_DEV_PHASE05_INVOKER__ = async (command, args) => {
  if (command === 'get_setup_status') return {setupRequired:false,hasDraft:false,schemaVersion:'0005',defaultFiscalStartsOn:'2026-01-01',defaultFiscalEndsOn:'2026-12-31'};
  if (command === 'get_current_session') throw {code:'AUTHENTICATION_REQUIRED'};
  if (command === 'login') return {companyId:'company-1',userId:'user-1',username:'admin',displayName:'Administrateur local',preferredLanguage:'ar-DZ',permissions:['*'],locked:false};
  if (command === 'logout') return null;
  if (command === 'get_company_profile') return {id:'company-1',code:'POSMAN',legalName:'SARL Atlas Commerce',nameAr:'مؤسسة الأطلس للتجارة',nameFr:'Atlas Commerce',activityDescription:'Commerce de gros',addressText:'Alger',wilayaCode:'16',phone:'0550000000',email:'contact@example.dz',defaultMarginRateScaled:200000,belowCostPolicy:'ADMIN_OVERRIDE',sessionIdleTimeoutMinutes:15,rowVersion:1};
  return [];
};
const result = (id, status='DRAFT', rowVersion=1) => ({id,documentNumber:`P6-${id}`,status,rowVersion,replayed:false});
const countResult = (status='DRAFT', rowVersion=1) => ({id:'count-1',warehouseId:'warehouse-main',countNumber:'INV-2026-001',commercialDate:'2026-08-05',status,rowVersion,lines:[{id:'count-line-1',productId:'product-oil',systemQuantityScaled:24000000,countedQuantityScaled:2000000,varianceQuantityScaled:-22000000,unitCostScaled:5240000,rowVersion}]});
window.__POSMAN_DEV_PHASE06_INVOKER__ = async (command, args) => {
  window.__POSMAN_PHASE06_CALLS__.push({command,args});
  const request = args?.request;
  switch (command) {
    case 'list_stock_balances': return [
      {productId:'product-oil',productCode:'HUI-001',productName:'زيت مائدة / Huile 1 L',warehouseId:'warehouse-main',warehouseName:'المستودع الرئيسي / Dépôt principal',onHandScaled:24000000,reservedScaled:3000000,availableScaled:21000000,averageCostScaled:5240000,inventoryValueMinor:1257600,rowVersion:3},
      {productId:'product-coffee',productCode:'CAF-250',productName:'قهوة 250غ / Café 250 g',warehouseId:'warehouse-main',warehouseName:'المستودع الرئيسي / Dépôt principal',warehouseLocationId:'location-a',locationName:'A-01',onHandScaled:11500000,reservedScaled:1500000,availableScaled:10000000,averageCostScaled:4812500,inventoryValueMinor:553438,rowVersion:4}
    ];
    case 'list_stock_movements': return [{id:'m1',productId:'product-oil',warehouseId:'warehouse-main',sourceDocumentId:'doc-open',movementType:'OPENING',businessDate:'2026-08-05',quantityDeltaScaled:24000000,quantityAfterScaled:24000000,unitCostScaled:5240000,averageCostAfterScaled:5240000,extendedCostMinor:1257600}];
    case 'list_active_stock_reservations': return [{id:'res-1',sourceLineId:'sales-line-1',productId:'product-oil',warehouseId:'warehouse-main',reservedQuantityScaled:3000000,status:'ACTIVE',rowVersion:1}];
    case 'list_purchasing_documents': return [{id:'po-1',documentType:'PURCHASE_ORDER',documentNumber:'BC-2026-000001',workflowStatus:'CONFIRMED',postingStatus:'DRAFT',commercialDate:'2026-08-05',partnerId:'supplier-1',totalHtMinor:104000,totalTaxMinor:19760,totalTtcMinor:123760,rowVersion:2,lines:[]}];
    case 'reconcile_stock_balances':
      if (window.__POSMAN_PHASE06_REBUILT__) return {rows:[{productId:'product-oil',warehouseId:'warehouse-main',projectionOnHandScaled:24000000,rebuiltOnHandScaled:24000000,projectionReservedScaled:3000000,rebuiltReservedScaled:3000000,projectionAverageCostScaled:5240000,rebuiltAverageCostScaled:5240000,matches:true}],mismatchCount:0,rebuilt:true};
      return {rows:[{productId:'product-oil',warehouseId:'warehouse-main',projectionOnHandScaled:23999999,rebuiltOnHandScaled:24000000,projectionReservedScaled:3000000,rebuiltReservedScaled:3000000,projectionAverageCostScaled:5240000,rebuiltAverageCostScaled:5240000,matches:false}],mismatchCount:1,rebuilt:false};
    case 'rebuild_stock_balances':
      window.__POSMAN_PHASE06_REBUILT__ = true;
      return {rows:[{productId:'product-oil',warehouseId:'warehouse-main',projectionOnHandScaled:24000000,rebuiltOnHandScaled:24000000,projectionReservedScaled:3000000,rebuiltReservedScaled:3000000,projectionAverageCostScaled:5240000,rebuiltAverageCostScaled:5240000,matches:true}],mismatchCount:0,rebuilt:true};
    case 'create_opening_stock': return result('opening-1');
    case 'review_opening_stock': return result('opening-1','REVIEWED',2);
    case 'post_opening_stock': return result('opening-1','POSTED',3);
    case 'create_inventory_count': return countResult();
    case 'review_inventory_count': return countResult('REVIEWED',2);
    case 'post_inventory_count': return result('count-1','POSTED',3);
    case 'create_purchase_order': return result('po-1');
    case 'confirm_purchase_order': return result('po-1','CONFIRMED',2);
    case 'create_purchase_receipt': return result('receipt-1');
    case 'post_purchase_receipt': return result('receipt-1','POSTED',2);
    case 'create_purchase_invoice': return result('invoice-1');
    case 'post_purchase_invoice': return result('invoice-1','POSTED',2);
    case 'direct_receive_and_invoice': return result('invoice-direct-1','POSTED',2);
    case 'post_stock_transfer': return result('transfer-1','POSTED',2);
    case 'post_stock_adjustment': {
      const payload=request?.payload;
      if ((payload?.lines?.[0]?.quantityScaled ?? 0) < 0 && !payload?.allowNegativeOverride) throw {code:'INSUFFICIENT_STOCK'};
      return result('adjustment-1','POSTED',2);
    }
    case 'post_purchase_return': return result('return-1','POSTED',2);
    case 'create_stock_reservation': return result('reservation-2','ACTIVE',1);
    case 'release_stock_reservation': return result('res-1','RELEASED',2);
    case 'cancel_stock_reservation': return result('res-1','CANCELLED',2);
    default: return result(`result-${command}`,'POSTED',2);
  }
};
"""

def wait_server(timeout: float = 45) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", 1420), timeout=.5): return
        except OSError: time.sleep(.25)
    raise RuntimeError("Vite did not start")

def login(page: Page, french: bool = False) -> None:
    page.goto(BASE_URL, wait_until="networkidle")
    page.locator("input[name='username']").fill("admin")
    page.locator("input[name='password']").fill("Correct-Horse-2026")
    page.locator("button[type='submit']").click()
    page.locator(".p5-shell").wait_for()
    if french:
        page.get_by_role("button", name="Français", exact=True).click()
        assert page.locator("html").get_attribute("dir") == "ltr"
        page.get_by_role("button", name="Stock et achats", exact=True).click()
    else:
        assert page.locator("html").get_attribute("dir") == "rtl"
        page.get_by_role("button", name="المخزون والمشتريات", exact=True).click()
    page.locator(".p6-workspace").wait_for()

def no_overflow(page: Page, label: str) -> None:
    dims = page.evaluate("() => ({inner:innerWidth,doc:document.documentElement.scrollWidth,body:document.body.scrollWidth})")
    if max(dims["doc"], dims["body"]) > dims["inner"] + 1: raise AssertionError(f"{label}: page overflow {dims}")

def no_clipped_primary(page: Page, label: str) -> None:
    failures = page.locator(".p6-commandbar h1,.p6-canvas__header h2,.p6-process button,.p6-action-dock button").evaluate_all("""els => els.filter(el => { const r=el.getBoundingClientRect(); const s=getComputedStyle(el); if(s.display==='none') return false; return r.width<=0 || r.height<=0 || el.scrollHeight>el.clientHeight+3; }).map(el => el.textContent?.trim())""")
    if failures: raise AssertionError(f"{label}: clipped primary labels {failures}")

def axe(page: Page, name: str) -> dict:
    page.add_script_tag(path=str(AXE_PATH))
    result = page.evaluate("async () => await axe.run(document, {runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}})")
    unresolved = [item for item in result["incomplete"] if item.get("impact") in {"critical", "serious"}]
    summary = {"violations":len(result["violations"]),"incomplete":len(result["incomplete"]),"unresolvedCriticalSeriousIncomplete":len(unresolved),"passes":len(result["passes"])}
    (ARTIFACT_DIR / f"axe-{name}.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    if summary["violations"] or unresolved: raise AssertionError(f"{name}: axe {summary}")
    return summary

def evidence(page: Page, name: str) -> dict:
    no_overflow(page, name); no_clipped_primary(page, name)
    page.screenshot(path=str(ARTIFACT_DIR / f"{name}.png"), full_page=True)
    report = axe(page, name)
    errors = page.evaluate("() => window.__POSMAN_E2E_ERRORS__ || []")
    if errors: raise AssertionError(f"{name}: browser errors {errors}")
    return {"name":name,"axe":report,"calls":page.evaluate("() => window.__POSMAN_PHASE06_CALLS__")}

def select(page: Page, ar: str, fr: str, french: bool) -> None: page.get_by_role("button", name=fr if french else ar, exact=True).click()

def run_case(browser, name: str, width: int, height: int, french: bool, action: Callable[[Page], None]) -> dict:
    page = browser.new_page(viewport={"width":width,"height":height})
    page.on("dialog", lambda dialog: dialog.accept())
    page.add_init_script(MOCK)
    page.add_init_script("window.__POSMAN_E2E_ERRORS__=[];addEventListener('error',e=>window.__POSMAN_E2E_ERRORS__.push(String(e.error||e.message)));addEventListener('unhandledrejection',e=>window.__POSMAN_E2E_ERRORS__.push(String(e.reason)));const old=console.error;console.error=(...a)=>{window.__POSMAN_E2E_ERRORS__.push(a.map(String).join(' '));old(...a)};")
    try:
        login(page, french); action(page); return evidence(page, name)
    finally: page.close()

def main() -> int:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    server = subprocess.Popen(["npm","run","dev","--","--host","127.0.0.1"],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,start_new_session=True)
    try:
        wait_server(); reports=[]
        with sync_playwright() as p:
            browser=p.chromium.launch()
            reports.append(run_case(browser,"ar-stock-overview-1280x800",1280,800,False,lambda page: page.locator("[data-testid='phase06-overview']").wait_for()))
            reports.append(run_case(browser,"fr-stock-overview-1280x800",1280,800,True,lambda page: page.locator("[data-testid='phase06-overview']").wait_for()))
            def opening(page):
                select(page,"المخزون الافتتاحي","Stock initial",False); page.locator("[data-testid='phase06-opening'] button[type='submit']").click(); page.get_by_role("button",name="إرسال للمراجعة").click(); page.get_by_role("button",name="ترحيل",exact=True).click(); page.get_by_text("المستند المرحّل مقفل").wait_for()
            reports.append(run_case(browser,"ar-opening-post",1280,800,False,opening))
            def partial_receipt(page):
                select(page,"محرر أمر الشراء","Éditeur de commande",True); page.locator("[data-testid='phase06-orderEditor'] button[type='submit']").click(); page.get_by_role("button",name="Confirmer",exact=True).click(); select(page,"استلام المشتريات","Réception achat",True); page.locator("[data-testid='phase06-receipt'] button[type='submit']").click(); page.get_by_role("button",name="Valider",exact=True).click()
            reports.append(run_case(browser,"fr-purchase-order-partial-receipt",1280,800,True,partial_receipt))
            reports.append(run_case(browser,"ar-direct-receive-invoice",1280,800,False,lambda page:(select(page,"استلام وفوترة مباشرة","Réception et facture directe",False),page.locator("[data-testid='phase06-direct'] button[type='submit']").click(),page.get_by_text("المستند المرحّل مقفل").wait_for())))
            reports.append(run_case(browser,"ar-transfer",1280,800,False,lambda page:(select(page,"تحويل المخزون","Transfert de stock",False),page.locator("[data-testid='phase06-transfer'] button[type='submit']").click())))
            def count(page):
                select(page,"الجرد الفعلي","Inventaire physique",True); page.locator("[data-testid='phase06-count'] button[type='submit']").click(); page.get_by_role("button",name="Soumettre à la revue",exact=True).click(); page.get_by_role("button",name="Valider",exact=True).click()
            reports.append(run_case(browser,"fr-stock-count",1280,800,True,count))
            def negative_block(page):
                select(page,"تسوية المخزون","Ajustement",False); page.locator("input[name='quantity']").fill("-100"); page.locator("[data-testid='phase06-adjustment'] button[type='submit']").click(); page.get_by_text("INSUFFICIENT_STOCK").wait_for()
            reports.append(run_case(browser,"ar-negative-stock-block",1280,800,False,negative_block))
            def override(page):
                select(page,"تسوية المخزون","Ajustement",False); page.locator("input[name='quantity']").fill("-100"); page.locator("input[name='override']").check(); page.get_by_text("سيؤدي هذا الإجراء إلى مخزون سالب").wait_for(); page.locator("[data-testid='phase06-adjustment'] button[type='submit']").click()
            reports.append(run_case(browser,"ar-negative-override-warning",1280,800,False,override))
            reports.append(run_case(browser,"fr-supplier-return",1280,800,True,lambda page:(select(page,"مرتجع المشتريات","Retour fournisseur",True),page.locator("[data-testid='phase06-return'] button[type='submit']").click())))
            def reconcile(page):
                select(page,"مطابقة الأرصدة","Rapprochement des soldes",False); page.locator(".p6-callout").get_by_text("عدم تطابق",exact=True).wait_for(); page.get_by_role("button",name="إعادة بناء projection",exact=True).click(); page.locator(".p6-callout").get_by_text("OK",exact=True).wait_for()
            reports.append(run_case(browser,"ar-reconciliation-rebuild",1280,800,False,reconcile))
            reports.append(run_case(browser,"ar-critical-1024x640",1024,640,False,lambda page:(select(page,"تحويل المخزون","Transfert de stock",False),page.locator("[data-testid='phase06-transfer']").wait_for())))
            reports.append(run_case(browser,"fr-critical-1024x640",1024,640,True,lambda page:(select(page,"استلام المشتريات","Réception achat",True),page.locator("[data-testid='phase06-receipt']").wait_for())))
            browser.close()
        (ARTIFACT_DIR / "phase06-e2e-summary.json").write_text(json.dumps(reports,ensure_ascii=False,indent=2),encoding="utf-8")
        print(json.dumps(reports,ensure_ascii=False,indent=2)); return 0
    finally:
        if server.poll() is None:
            os.killpg(server.pid,signal.SIGTERM)
            try: server.wait(timeout=10)
            except subprocess.TimeoutExpired: os.killpg(server.pid,signal.SIGKILL)
        if server.stdout: (ARTIFACT_DIR / "phase06-vite.log").write_text(server.stdout.read(),encoding="utf-8")
if __name__ == "__main__": raise SystemExit(main())
