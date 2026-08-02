#!/usr/bin/env python3
"""PHASE 05 Arabic/French browser, accessibility, clipping, and evidence gate."""
from __future__ import annotations

import json
import os
import signal
import socket
import subprocess
import time
from pathlib import Path
from playwright.sync_api import Page, sync_playwright

ROOT = Path(__file__).resolve().parents[2]
ARTIFACT_DIR = Path(os.environ.get("POSMAN_ARTIFACT_DIR", Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "posman-phase05-evidence"))
BASE_URL = "http://127.0.0.1:1420"
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"

MOCK = r"""
window.__POSMAN_PHASE05_CALLS__ = [];
window.__POSMAN_DEV_PHASE05_INVOKER__ = async (command, args) => {
  window.__POSMAN_PHASE05_CALLS__.push({command, args});
  const page = (items) => ({items, page: 1, pageSize: 100, total: items.length});
  switch (command) {
    case 'get_setup_status': return {setupRequired:false,hasDraft:false,schemaVersion:'0005',defaultFiscalStartsOn:'2026-01-01',defaultFiscalEndsOn:'2026-12-31'};
    case 'get_current_session': throw {code:'AUTHENTICATION_REQUIRED'};
    case 'login': return {companyId:'company-1',userId:'user-1',username:'admin',displayName:'Administrateur local',preferredLanguage:'ar-DZ',permissions:['*'],locked:false};
    case 'logout': return null;
    case 'get_company_profile': return {id:'company-1',code:'POSMAN',legalName:'SARL Atlas Commerce',nameAr:'مؤسسة الأطلس للتجارة',nameFr:'Atlas Commerce',activityDescription:'Commerce de gros',addressText:'Alger',wilayaCode:'16',phone:'0550000000',email:'contact@example.dz',defaultMarginRateScaled:200000,belowCostPolicy:'ADMIN_OVERRIDE',sessionIdleTimeoutMinutes:15,rowVersion:1};
    case 'get_fiscal_setup': return {fiscalYearId:'fy-1',code:'2026',startsOn:'2026-01-01',endsOn:'2026-12-31',periods:Array.from({length:12},(_,i)=>({periodNumber:i+1,name:String(i+1).padStart(2,'0'),startsOn:`2026-${String(i+1).padStart(2,'0')}-01`,endsOn:`2026-${String(i+1).padStart(2,'0')}-28`,status:'OPEN'})),rowVersion:1,inUse:false};
    case 'list_document_sequences': return [{id:'seq-1',documentType:'SALES_INVOICE',prefix:'FAC',nextNumber:1,paddingWidth:6,preview:'FAC-2026-000001',rowVersion:1},{id:'seq-2',documentType:'DELIVERY_NOTE',prefix:'BL',nextNumber:1,paddingWidth:6,preview:'BL-2026-000001',rowVersion:1},{id:'seq-3',documentType:'PURCHASE_ORDER',prefix:'BC',nextNumber:1,paddingWidth:6,preview:'BC-2026-000001',rowVersion:1}];
    case 'list_users': return page([{id:'u1',username:'admin',displayName:'Administrateur local',preferredLanguage:'ar-DZ',isActive:true,roleIds:['r1'],rowVersion:1}]);
    case 'list_roles': return [{id:'r1',code:'SYSTEM_ADMINISTRATOR',nameAr:'مسؤول النظام',nameFr:'Administrateur système',isSystem:true,isActive:true,permissionCodes:['*'],rowVersion:1}];
    case 'list_products': return page([{id:'p1',code:'CAF-250',nameAr:'قهوة 250غ',nameFr:'Café 250 g',unitId:'u1',purchasePriceScaled:480000,salePriceScaled:470000,suggestedSalePriceScaled:540000,pricingWarning:'BELOW_COST',belowCostPolicy:'ADMIN_OVERRIDE',isActive:true,rowVersion:1},{id:'p2',code:'HUI-001',nameAr:'زيت 1 لتر',nameFr:'Huile 1 L',unitId:'u1',purchasePriceScaled:520000,salePriceScaled:520000,suggestedSalePriceScaled:620000,pricingWarning:'ZERO_MARGIN',belowCostPolicy:'ADMIN_OVERRIDE',isActive:true,rowVersion:1}]);
    case 'list_partners': return page([{id:'c1',code:'CLI-001',legalName:'SARL Client Atlas',displayNameAr:'عميل الأطلس',displayNameFr:'Client Atlas',isCustomer:true,isSupplier:false,isActive:true,rowVersion:1},{id:'s1',code:'FRN-001',legalName:'SARL Fournisseur Atlas',displayNameAr:'مورد الأطلس',displayNameFr:'Fournisseur Atlas',isCustomer:false,isSupplier:true,isActive:true,rowVersion:1}]);
    case 'list_product_families': case 'list_units': case 'list_warehouses': case 'list_warehouse_locations': case 'list_tax_rates': return page([{id:'ref1',code:'STD',nameAr:'قياسي',nameFr:'Standard',isActive:true,rowVersion:1,details:{}}]);
    default: return null;
  }
};
"""

def wait_server(timeout: float = 45) -> None:
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        try:
            with socket.create_connection(("127.0.0.1", 1420), timeout=.5): return
        except OSError: time.sleep(.25)
    raise RuntimeError("Vite did not start")

def no_overflow(page: Page, label: str) -> None:
    dims = page.evaluate("() => ({inner:innerWidth,doc:document.documentElement.scrollWidth,body:document.body.scrollWidth})")
    if max(dims["doc"], dims["body"]) > dims["inner"] + 1: raise AssertionError(f"{label}: page overflow {dims}")

def visible_labels(page: Page, label: str) -> None:
    failures = page.locator("button, h1, .p5-topbar strong").evaluate_all("""els => els.filter(el => { const r=el.getBoundingClientRect(); return r.width<=0 || r.height<=0 || el.scrollWidth>el.clientWidth+2 || el.scrollHeight>el.clientHeight+2; }).map(el => el.textContent?.trim())""")
    if failures: raise AssertionError(f"{label}: clipped labels {failures}")

def _axe_issue(rule: dict) -> dict:
    return {
        "id": rule.get("id"),
        "impact": rule.get("impact"),
        "help": rule.get("help"),
        "helpUrl": rule.get("helpUrl"),
        "nodes": [
            {
                "target": node.get("target"),
                "html": (node.get("html") or "")[:500],
                "failureSummary": node.get("failureSummary"),
            }
            for node in rule.get("nodes", [])
        ],
    }

def axe(page: Page, name: str) -> dict:
    page.add_script_tag(path=str(AXE_PATH))
    result = page.evaluate("async () => await axe.run(document, {runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}})")
    unresolved = [item for item in result["incomplete"] if item.get("impact") in {"critical", "serious"}]
    summary = {"violations": len(result["violations"]), "incomplete": len(result["incomplete"]), "unresolvedCriticalSeriousIncomplete": len(unresolved), "passes": len(result["passes"])}
    report_path = ARTIFACT_DIR / f"axe-{name}.json"
    report_path.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    if summary["violations"] or unresolved:
        diagnostics = {
            "viewport": name,
            "summary": summary,
            "violations": [_axe_issue(item) for item in result["violations"]],
            "criticalSeriousIncomplete": [_axe_issue(item) for item in unresolved],
        }
        diagnostic_path = ARTIFACT_DIR / f"axe-{name}-diagnostics.json"
        diagnostic_path.write_text(json.dumps(diagnostics, ensure_ascii=False, indent=2), encoding="utf-8")
        print(json.dumps(diagnostics, ensure_ascii=False, indent=2), flush=True)
        raise AssertionError(f"{name}: axe {summary}; details={diagnostic_path}")
    return summary

def run_view(page: Page, width: int, height: int, locale_name: str) -> dict:
    console_errors: list[str] = []
    page.on("console", lambda message: console_errors.append(message.text) if message.type == "error" else None)
    page.on("pageerror", lambda error: console_errors.append(str(error)))
    page.set_viewport_size({"width": width, "height": height})
    page.goto(BASE_URL, wait_until="networkidle")
    page.locator(".p5-auth__card").wait_for()
    assert page.locator("html").get_attribute("dir") == "rtl"
    page.locator("input[name='username']").fill("admin")
    page.locator("input[name='password']").fill("Correct-Horse-2026")
    page.locator("button[type='submit']").click()
    page.locator(".p5-shell").wait_for()
    page.get_by_role("button", name="المواد", exact=True).click()
    page.locator("text=سعر البيع أقل من تكلفة الشراء").wait_for()
    no_overflow(page, f"ar-{width}x{height}")
    visible_labels(page, f"ar-{width}x{height}")
    page.screenshot(path=str(ARTIFACT_DIR / f"phase05-ar-{width}x{height}.png"), full_page=True)
    ar_axe = axe(page, f"ar-{width}x{height}")
    if console_errors: raise AssertionError(f"ar-{width}x{height}: console errors {console_errors}")
    page.get_by_role("button", name="Français", exact=True).click()
    assert page.locator("html").get_attribute("dir") == "ltr"
    page.get_by_role("button", name="Articles", exact=True).click()
    page.locator("text=Prix de vente inférieur au coût d’achat").wait_for()
    no_overflow(page, f"fr-{width}x{height}")
    visible_labels(page, f"fr-{width}x{height}")
    page.screenshot(path=str(ARTIFACT_DIR / f"phase05-fr-{width}x{height}.png"), full_page=True)
    fr_axe = axe(page, f"fr-{width}x{height}")
    if console_errors: raise AssertionError(f"fr-{width}x{height}: console errors {console_errors}")
    return {"viewport": f"{width}x{height}", "ar": ar_axe, "fr": fr_axe, "calls": page.evaluate("() => window.__POSMAN_PHASE05_CALLS__")}

def main() -> int:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    server = subprocess.Popen(["npm", "run", "dev", "--", "--host", "127.0.0.1"], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        wait_server()
        reports = []
        with sync_playwright() as p:
            browser = p.chromium.launch()
            for width, height in ((1280, 800), (1024, 640)):
                page = browser.new_page(viewport={"width": width, "height": height})
                page.add_init_script(MOCK)
                reports.append(run_view(page, width, height, "ar-DZ"))
                page.close()
            browser.close()
        (ARTIFACT_DIR / "phase05-e2e-summary.json").write_text(json.dumps(reports, ensure_ascii=False, indent=2), encoding="utf-8")
        print(json.dumps(reports, ensure_ascii=False, indent=2))
        return 0
    finally:
        if server.poll() is None:
            os.killpg(server.pid, signal.SIGTERM)
            try: server.wait(timeout=10)
            except subprocess.TimeoutExpired: os.killpg(server.pid, signal.SIGKILL)
        if server.stdout:
            (ARTIFACT_DIR / "vite.log").write_text(server.stdout.read(), encoding="utf-8")
if __name__ == "__main__": raise SystemExit(main())
