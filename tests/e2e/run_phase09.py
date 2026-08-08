#!/usr/bin/env python3
"""PHASE 09 browser workflows, accessibility, overflow, and screenshot evidence."""

from __future__ import annotations

import json
import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

from playwright.sync_api import Page, sync_playwright

ROOT = Path(__file__).resolve().parents[2]
ARTIFACT_DIR = Path(
    os.environ.get(
        "POSMAN_PHASE09_ARTIFACT_DIR",
        Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "phase-09-ui-evidence",
    )
)
BASE_URL = "http://127.0.0.1:1420/#phase09"
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"

MOCK = r"""
window.__POSMAN_PHASE09_CALLS__=[];
window.__POSMAN_PHASE09_LOGGED_IN__=true;
window.__POSMAN_PHASE09_RENDERED__=[{
 renderId:'render-original',documentType:'SALES_INVOICE',sourceDocumentId:'invoice-7',sourceDocumentNumber:'FAC-2026-000007',sourceDocumentStatus:'POSTED',templateId:'template-sales',templateVersionId:'template-version-1',locale:'ar-DZ',contentSha256:'a'.repeat(64),pdfRelativePath:'documents/original.pdf',pdfSha256:'b'.repeat(64),pdfSizeBytes:4096,renderedAt:'2026-08-08T08:00:00Z',renderedBy:'user-1',integrityState:'VERIFIED'
}];
window.__POSMAN_PHASE09_BACKUPS__=[
 {backupId:'backup-good',backupKind:'MANUAL',createdAt:'2026-08-08T08:00:00Z',createdBy:'user-1',applicationVersion:'0.1.0',schemaVersion:'0007',migrationLedgerDigest:'c'.repeat(64),databaseSizeBytes:8192,sha256:'d'.repeat(64),relativePath:'backups/good.sqlite3',integrityStatus:'OK',foreignKeyStatus:'OK',verificationStatus:'VERIFIED',failureReason:null,selectedForRestore:false},
 {backupId:'backup-corrupt',backupKind:'MANUAL',createdAt:'2026-08-08T07:00:00Z',createdBy:'user-1',applicationVersion:'0.1.0',schemaVersion:'0007',migrationLedgerDigest:'c'.repeat(64),databaseSizeBytes:8192,sha256:'e'.repeat(64),relativePath:'backups/corrupt.sqlite3',integrityStatus:'FAILED',foreignKeyStatus:'UNKNOWN',verificationStatus:'FAILED',failureReason:'BACKUP_INTEGRITY_FAILED',selectedForRestore:false}
];
window.__POSMAN_PHASE09_DRAFT__={
 draftId:'draft-sales',templateId:'template-sales',documentType:'SALES_INVOICE',locale:'ar-DZ',displayName:'فاتورة مبيعات A4',baseTemplateVersionId:'template-version-1',rowVersion:1,updatedAt:'2026-08-08T08:00:00Z',
 configuration:{documentTitleAr:'فاتورة مبيعات',documentTitleFr:'Facture de vente',showLogo:true,showCompanyIdentity:true,showTradeRegister:true,showTaxIdentifier:true,showPartnerAddress:true,showPaymentInformation:true,footerTextAr:'شكرًا لتعاملكم',footerTextFr:'Merci pour votre confiance',spacing:'NORMAL',orientation:'PORTRAIT',enabledSections:['LINES','TOTALS']}
};
window.__POSMAN_PHASE09_VERSIONS__=[{versionId:'template-version-1',versionNumber:1,locale:'ar-DZ',contentSha256:'f'.repeat(64),status:'PUBLISHED',publishedAt:'2026-08-08T07:00:00Z',publishedBy:'user-1',rowVersion:1}];
const session={companyId:'company-1',userId:'user-1',username:'admin',displayName:'Administrateur local',preferredLanguage:'ar-DZ',permissions:['*'],locked:false};
window.__POSMAN_DEV_PHASE05_INVOKER__=async(command,args)=>{
 if(command==='get_setup_status') return {setupRequired:false,hasDraft:false,schemaVersion:'0007',defaultFiscalStartsOn:'2026-01-01',defaultFiscalEndsOn:'2026-12-31'};
 if(command==='get_current_session') { if(!window.__POSMAN_PHASE09_LOGGED_IN__) throw {code:'AUTHENTICATION_REQUIRED'}; return session; }
 if(command==='login') { window.__POSMAN_PHASE09_LOGGED_IN__=true; return session; }
 return null;
};
const page=(items)=>({items,page:1,pageSize:50,total:items.length});
const exportResult=(name)=>({relativePath:name,sha256:'9'.repeat(64),sizeBytes:2048});
const summary=()=>({templateId:'template-sales',documentType:'SALES_INVOICE',locale:'ar-DZ',displayName:'فاتورة مبيعات A4',activeVersionId:'template-version-1',activeVersionNumber:1,activeContentSha256:'f'.repeat(64),draftId:window.__POSMAN_PHASE09_DRAFT__?.draftId||null,draftRowVersion:window.__POSMAN_PHASE09_DRAFT__?.rowVersion||null,state:window.__POSMAN_PHASE09_DRAFT__?'DRAFT':'PUBLISHED'});
window.__POSMAN_DEV_PHASE09_INVOKER__=async(command,args)=>{
 window.__POSMAN_PHASE09_CALLS__.push({command,args});
 switch(command){
  case 'phase09_list_templates': return [summary()];
  case 'phase09_get_template': return {summary:summary(),draft:window.__POSMAN_PHASE09_DRAFT__,versions:window.__POSMAN_PHASE09_VERSIONS__};
  case 'phase09_create_template_draft': return window.__POSMAN_PHASE09_DRAFT__;
  case 'phase09_update_template_draft': window.__POSMAN_PHASE09_DRAFT__={...window.__POSMAN_PHASE09_DRAFT__,...args.request,rowVersion:2}; return window.__POSMAN_PHASE09_DRAFT__;
  case 'phase09_publish_template': { const version={versionId:'template-version-2',versionNumber:2,locale:'ar-DZ',contentSha256:'8'.repeat(64),status:'PUBLISHED',publishedAt:'2026-08-08T09:00:00Z',publishedBy:'user-1',rowVersion:1}; window.__POSMAN_PHASE09_VERSIONS__.unshift(version); window.__POSMAN_PHASE09_DRAFT__=null; return version; }
  case 'phase09_retire_template': return {...window.__POSMAN_PHASE09_VERSIONS__[0],status:'RETIRED',rowVersion:2};
  case 'phase09_list_rendered_documents': return page(window.__POSMAN_PHASE09_RENDERED__);
  case 'phase09_preview_document': return {previewId:'preview-1',documentType:args.request.documentType,sourceDocumentId:args.request.sourceDocumentId,locale:args.request.locale,integrityState:'VERIFIED'};
  case 'phase09_get_preview_content': return {previewId:'preview-1',locale:'ar-DZ',direction:'rtl',html:'<main>FAC-2026-000007</main>',css:'@page{size:A4}',contentSha256:'7'.repeat(64),integrityState:'VERIFIED'};
  case 'phase09_render_document': { const item={...window.__POSMAN_PHASE09_RENDERED__[0],renderId:'render-new',sourceDocumentId:args.request.sourceDocumentId,sourceDocumentNumber:'FAC-2026-000008',locale:args.request.locale,pdfSha256:'6'.repeat(64)}; window.__POSMAN_PHASE09_RENDERED__.unshift(item); return item; }
  case 'phase09_get_rendered_document': return window.__POSMAN_PHASE09_RENDERED__[0];
  case 'phase09_verify_rendered_document': return {...window.__POSMAN_PHASE09_RENDERED__.find(item=>item.renderId===args.request.renderId),integrityState:'VERIFIED'};
  case 'phase09_export_rendered_pdf': return exportResult('document.pdf');
  case 'phase09_print_rendered_document': return null;
  case 'phase09_list_reports': return [{reportId:'SALES_SUMMARY',nameAr:'ملخص المبيعات',nameFr:'Résumé des ventes',supportsDateRange:true,supportsWarehouse:false,supportsPartner:false,supportsProduct:false,supportsStatus:false}];
  case 'phase09_run_report': return {reportId:'SALES_SUMMARY',columns:[{key:'total',labelAr:'الإجمالي',labelFr:'Total',kind:'MONEY'}],rows:[{values:{total:119000}}],page:1,pageSize:50,totalRows:1,generatedAt:'2026-08-08T09:00:00Z'};
  case 'phase09_export_report_csv': return exportResult('report.csv');
  case 'phase09_export_report_pdf': return exportResult('report.pdf');
  case 'phase09_list_audit_events': return page([{id:'audit-1',actorUserId:'user-1',actorDisplayName:'Administrateur local',domain:'BACKUP',actionCode:'BACKUP_CREATED',entityType:'PHASE09_BACKUP',entityId:'backup-good',occurredAt:'2026-08-08T08:00:00Z',outcome:'SUCCESS',sensitive:true,details:{credential:'[REDACTED]',path:'[REDACTED]'}}]);
  case 'phase09_export_audit_csv': return exportResult('audit.csv');
  case 'phase09_get_backup_settings': return {automaticEnabled:true,weeklyEnabled:true,timezoneName:'Africa/Algiers',lastAttemptLocalDate:'2026-08-08',lastSuccessLocalDate:'2026-08-08',lastWarningCode:null,rowVersion:1,encryptionStatus:'LOCAL_UNENCRYPTED'};
  case 'phase09_update_backup_settings': return {automaticEnabled:args.request.automaticEnabled,weeklyEnabled:args.request.weeklyEnabled,timezoneName:'Africa/Algiers',lastAttemptLocalDate:'2026-08-08',lastSuccessLocalDate:'2026-08-08',lastWarningCode:null,rowVersion:2,encryptionStatus:'LOCAL_UNENCRYPTED'};
  case 'phase09_create_backup': { const item={...window.__POSMAN_PHASE09_BACKUPS__[0],backupId:'backup-new',createdAt:'2026-08-08T09:00:00Z',sha256:'5'.repeat(64)}; window.__POSMAN_PHASE09_BACKUPS__.unshift(item); return item; }
  case 'phase09_list_backups': return page(window.__POSMAN_PHASE09_BACKUPS__);
  case 'phase09_verify_backup': { const item=window.__POSMAN_PHASE09_BACKUPS__.find(value=>value.backupId===args.request.backupId); if(item.backupId==='backup-corrupt') throw {code:'BACKUP_INTEGRITY_FAILED',message:'The backup failed integrity verification.',retryable:false}; return {...item,verificationStatus:'VERIFIED'}; }
  case 'phase09_export_backup': return exportResult('backup.sqlite3');
  case 'phase09_import_backup': return window.__POSMAN_PHASE09_BACKUPS__[0];
  case 'phase09_restore_backup': window.__POSMAN_PHASE09_LOGGED_IN__=false; return null;
  case 'phase09_delete_backup': window.__POSMAN_PHASE09_BACKUPS__=window.__POSMAN_PHASE09_BACKUPS__.filter(value=>value.backupId!==args.request.backupId); return null;
  default: throw {code:'UNKNOWN_COMMAND',message:'Unknown command',retryable:false};
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
            with socket.create_connection(("127.0.0.1", 1420), timeout=0.5):
                return
        except OSError:
            time.sleep(0.25)
    raise RuntimeError("Vite did not start")


def calls(page: Page) -> list[dict[str, object]]:
    return page.evaluate("() => window.__POSMAN_PHASE09_CALLS__")


def assert_called(page: Page, command: str) -> None:
    page.wait_for_function(
        "command => (window.__POSMAN_PHASE09_CALLS__ || []).some(item => item.command === command)",
        arg=command,
    )
    if command not in [item["command"] for item in calls(page)]:
        raise AssertionError(f"expected {command} invocation")


def open_workspace(page: Page, locale: str) -> None:
    page.add_init_script(MOCK)
    page.add_init_script(ERROR_CAPTURE)
    page.goto(BASE_URL, wait_until="networkidle")
    page.locator("main.phase09-workspace").wait_for()
    page.get_by_role("button", name="Français" if locale == "fr-DZ" else "العربية", exact=True).click()
    direction = "ltr" if locale == "fr-DZ" else "rtl"
    page.locator(f'main.phase09-workspace[dir="{direction}"]').wait_for()


def click_tab(page: Page, ar: str, fr: str, locale: str) -> None:
    page.get_by_role("button", name=fr if locale == "fr-DZ" else ar, exact=True).click()


def run_named_workflow(page: Page, name: str, locale: str) -> None:
    open_workspace(page, locale)
    french = locale == "fr-DZ"
    if name == "phase09_ar_template_publish_and_historical_reprint":
        click_tab(page, "القوالب", "Modèles", locale)
        page.on("dialog", lambda dialog: dialog.accept())
        page.get_by_role("button", name="نشر النسخة", exact=True).click()
        assert_called(page, "phase09_publish_template")
        page.get_by_text("v2", exact=True).wait_for()
        click_tab(page, "الوثائق", "Documents", locale)
        page.get_by_role("button", name="طباعة", exact=True).first.click()
        assert_called(page, "phase09_print_rendered_document")
    elif name == "phase09_fr_sales_invoice_preview_and_pdf":
        field = page.get_by_label("Identifiant source")
        field.fill("invoice-8")
        page.get_by_role("button", name="Aperçu", exact=True).click()
        page.locator(".phase09-integrity").filter(has_text="VERIFIED").first.wait_for()
        page.get_by_role("button", name="Créer le PDF historique", exact=True).click()
        page.locator(f'code[title="{"6" * 64}"]').wait_for()
        assert_called(page, "phase09_preview_document")
        assert_called(page, "phase09_render_document")
    elif name == "phase09_ar_reports_csv_and_pdf":
        click_tab(page, "التقارير", "Rapports", locale)
        page.get_by_role("button", name="تشغيل التقرير", exact=True).click()
        page.get_by_text("119000", exact=True).wait_for()
        page.get_by_role("button", name="تصدير CSV", exact=True).click()
        page.get_by_text("9" * 12, exact=False).wait_for()
        page.get_by_role("button", name="تصدير PDF", exact=True).click()
        assert_called(page, "phase09_export_report_csv")
        assert_called(page, "phase09_export_report_pdf")
    elif name == "phase09_fr_audit_filter_and_redacted_export":
        click_tab(page, "سجل التدقيق", "Audit", locale)
        page.get_by_label("Événements sensibles uniquement").check()
        page.get_by_role("button", name="Appliquer les filtres", exact=True).click()
        page.get_by_text("[REDACTED]", exact=False).first.wait_for()
        if "secret" in page.locator("main").inner_text().lower():
            raise AssertionError("audit UI leaked secret text")
        page.get_by_role("button", name="Exporter CSV", exact=True).click()
        assert_called(page, "phase09_export_audit_csv")
    elif name == "phase09_ar_manual_backup_and_verification":
        click_tab(page, "النسخ والاستعادة", "Sauvegarde et restauration", locale)
        page.get_by_role("button", name="إنشاء نسخة يدوية", exact=True).click()
        assert_called(page, "phase09_create_backup")
        page.locator(f'code[title="{"5" * 64}"]').wait_for()
        page.get_by_role("button", name="التحقق من السلامة", exact=True).first.click()
        assert_called(page, "phase09_verify_backup")
    elif name == "phase09_fr_corrupted_backup_rejected":
        click_tab(page, "النسخ والاستعادة", "Sauvegarde et restauration", locale)
        page.get_by_role("button", name="Vérifier l’intégrité", exact=True).nth(1).click()
        page.get_by_text("BACKUP_INTEGRITY_FAILED", exact=True).wait_for()
    elif name == "phase09_ar_restore_requires_verified_safety_backup":
        click_tab(page, "النسخ والاستعادة", "Sauvegarde et restauration", locale)
        page.get_by_label("كلمة المرور الحالية").fill("Local-Admin-Password")
        page.get_by_label("اكتب RESTORE").fill("RESTORE")
        page.on("dialog", lambda dialog: dialog.accept())
        page.get_by_role("button", name="استعادة البيانات", exact=True).first.click()
        page.locator(".p5-auth").wait_for()
        restore = next(item for item in calls(page) if item["command"] == "phase09_restore_backup")
        request = restore["args"]["request"]
        if request["confirmationText"] != "RESTORE" or request["confirmed"] is not True:
            raise AssertionError("restore confirmation envelope is incomplete")
    elif name == "phase09_fr_restore_success_returns_to_login":
        click_tab(page, "النسخ والاستعادة", "Sauvegarde et restauration", locale)
        page.get_by_label("Mot de passe actuel").fill("Local-Admin-Password")
        page.get_by_label("Saisissez RESTORE").fill("RESTORE")
        page.on("dialog", lambda dialog: dialog.accept())
        page.get_by_role("button", name="Restaurer les données", exact=True).first.click()
        page.locator(".p5-auth").wait_for()
        if page.locator("main.phase09-workspace").count():
            raise AssertionError("restore did not leave the authenticated workspace")
    else:
        raise AssertionError(f"unknown scenario {name}")


def collect_evidence(
    page: Page, name: str, locale: str, page_errors: list[str]
) -> dict[str, object]:
    if page.locator("main.phase09-workspace").count():
        expected = "ltr" if locale == "fr-DZ" else "rtl"
        actual = page.locator("main.phase09-workspace").get_attribute("dir")
        if actual != expected:
            raise AssertionError(f"{name}: direction {actual}, expected {expected}")
    dimensions = page.evaluate("() => ({inner:innerWidth,html:document.documentElement.scrollWidth,body:document.body.scrollWidth})")
    if max(dimensions["html"], dimensions["body"]) > dimensions["inner"] + 1:
        raise AssertionError(f"{name}: horizontal overflow {dimensions}")
    clipped = page.locator(".workspace-switcher button,.phase09-hero h1,.phase09-tabs button,.phase09-panel h2,.phase09-actions button").evaluate_all(
        """elements => elements.filter(element => { const style=getComputedStyle(element); if(style.display==='none') return false; return element.scrollHeight>element.clientHeight+3; }).map(element=>element.textContent?.trim())"""
    )
    if clipped:
        raise AssertionError(f"{name}: clipped primary labels {clipped}")
    if not AXE_PATH.is_file():
        raise RuntimeError(f"axe-core is unavailable at {AXE_PATH}")
    page.add_script_tag(path=str(AXE_PATH))
    axe = page.evaluate("async () => await window.axe.run(document, {runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}})")
    unresolved = [item for item in axe["incomplete"] if item.get("impact") in {"critical", "serious"}]
    (ARTIFACT_DIR / f"axe-{name}.json").write_text(json.dumps(axe, ensure_ascii=False, indent=2), encoding="utf-8")
    if axe["violations"] or unresolved:
        raise AssertionError(f"{name}: axe violations={len(axe['violations'])}, unresolved={len(unresolved)}")
    errors = page.evaluate("() => window.__POSMAN_E2E_ERRORS__ || []")
    if errors or page_errors:
        raise AssertionError(
            f"{name}: browser errors {errors}; pageerror events {page_errors}"
        )
    screenshot = ARTIFACT_DIR / f"{name}.png"
    page.screenshot(path=str(screenshot), full_page=True)
    return {
        "name": name,
        "locale": locale,
        "viewport": page.viewport_size,
        "screenshot": screenshot.name,
        "axe": {"violations": 0, "incomplete": len(axe["incomplete"]), "unresolvedCriticalSerious": 0},
        "overflow": dimensions,
        "calls": calls(page),
        "outcome": "PASS",
    }


SCENARIOS = (
    ("phase09_ar_template_publish_and_historical_reprint", "ar-DZ", (1280, 800)),
    ("phase09_fr_sales_invoice_preview_and_pdf", "fr-DZ", (1024, 640)),
    ("phase09_ar_reports_csv_and_pdf", "ar-DZ", (1280, 800)),
    ("phase09_fr_audit_filter_and_redacted_export", "fr-DZ", (1024, 640)),
    ("phase09_ar_manual_backup_and_verification", "ar-DZ", (1280, 800)),
    ("phase09_fr_corrupted_backup_rejected", "fr-DZ", (1024, 640)),
    ("phase09_ar_restore_requires_verified_safety_backup", "ar-DZ", (1280, 800)),
    ("phase09_fr_restore_success_returns_to_login", "fr-DZ", (1024, 640)),
)


def main() -> int:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    vite_log = (ARTIFACT_DIR / "vite.log").open("w", encoding="utf-8")
    server = subprocess.Popen(
        ["npm", "run", "dev", "--", "--host", "127.0.0.1", "--port", "1420"],
        cwd=ROOT,
        stdout=vite_log,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    results: list[dict[str, object]] = []
    failed = False
    try:
        wait_server()
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(headless=True)
            try:
                for name, locale, viewport in SCENARIOS:
                    page = browser.new_page(viewport={"width": viewport[0], "height": viewport[1]})
                    page_errors: list[str] = []
                    page.on("pageerror", lambda error: page_errors.append(str(error)))
                    try:
                        run_named_workflow(page, name, locale)
                        results.append(collect_evidence(page, name, locale, page_errors))
                    except Exception as error:
                        failed = True
                        failure = ARTIFACT_DIR / f"{name}-failure.png"
                        try:
                            page.screenshot(path=str(failure), full_page=True)
                        except Exception:
                            pass
                        results.append({"name": name, "locale": locale, "outcome": "FAIL", "error": str(error), "screenshot": failure.name})
                    finally:
                        page.close()
            finally:
                browser.close()
    finally:
        if server.poll() is None:
            os.killpg(server.pid, signal.SIGTERM)
            try:
                server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(server.pid, signal.SIGKILL)
        vite_log.close()
    manifest = {"syntheticDataOnly": True, "baseUrl": BASE_URL, "scenarios": results}
    (ARTIFACT_DIR / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
