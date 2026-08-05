#!/usr/bin/env python3
from __future__ import annotations
import json, os, signal, socket, subprocess, time
from pathlib import Path
from typing import Callable
from playwright.sync_api import Page, sync_playwright

ROOT=Path(__file__).resolve().parents[2]
ARTIFACT_DIR=Path(os.environ.get("POSMAN_ARTIFACT_DIR",ROOT/"artifacts/phase07"))
AXE_PATH=ROOT/"node_modules/axe-core/axe.min.js"
BASE_URL="http://127.0.0.1:1420"
MOCK=r"""
window.__POSMAN_PHASE07_CALLS__=[];
window.__POSMAN_DEV_PHASE07_INVOKER__=async(command,args)=>{
 window.__POSMAN_PHASE07_CALLS__.push({command,args});
 const entity={id:'doc-'+command,documentNumber:command.includes('invoice')||command==='direct_sale'?'FAC000008':'BL000009',status:'POSTED',rowVersion:2,replayed:false};
 if(command==='get_sales_summary')return{draftOrders:2,confirmedOrders:3,partialOrders:1,uninvoicedDeliveries:2,postedInvoices:7,belowCostOverrides:1};
 if(command==='list_sales_documents')return[
  {id:'order-20',documentType:'SALES_ORDER',documentNumber:'CMD000020',workflowStatus:'PARTIALLY_DELIVERED',postingStatus:'DRAFT',commercialDate:'2026-08-05',partnerId:'customer-1',warehouseId:'warehouse-main',totalHtMinor:250000,totalTaxMinor:47500,totalTtcMinor:297500,rowVersion:3,lines:[]},
  {id:'delivery-8',documentType:'DELIVERY_NOTE',documentNumber:'BL000008',workflowStatus:'POSTED',postingStatus:'POSTED',commercialDate:'2026-08-05',partnerId:'customer-1',warehouseId:'warehouse-main',sourceDocumentId:'order-20',totalHtMinor:100000,totalTaxMinor:19000,totalTtcMinor:119000,rowVersion:2,lines:[]}
 ];
 if(command==='direct_sale'||command==='post_sales_return')return{primary:entity,relatedDocumentIds:['related-1']};
 return entity;
};
"""

def wait_server()->None:
 for _ in range(100):
  try:
   with socket.create_connection(("127.0.0.1",1420),timeout=.5): return
  except OSError: time.sleep(.2)
 raise RuntimeError("Vite did not start")

def no_overflow(page:Page,label:str)->None:
 dims=page.evaluate("() => ({inner:innerWidth,doc:document.documentElement.scrollWidth,body:document.body.scrollWidth})")
 if max(dims["doc"],dims["body"])>dims["inner"]+1: raise AssertionError(f"{label}: overflow {dims}")

def no_clipping(page:Page,label:str)->None:
 failures=page.locator(".p7-commandbar h1,.p7-rail button,.p7-canvas h2,.p7-action-dock button").evaluate_all("""els=>els.filter(el=>{const s=getComputedStyle(el),r=el.getBoundingClientRect();if(s.display==='none')return false;return r.width<=0||r.height<=0||el.scrollHeight>el.clientHeight+3}).map(el=>el.textContent?.trim())""")
 if failures: raise AssertionError(f"{label}: clipped {failures}")

def axe(page:Page,name:str)->dict:
 page.add_script_tag(path=str(AXE_PATH));result=page.evaluate("async()=>await axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}})")
 unresolved=[item for item in result["incomplete"] if item.get("impact") in {"critical","serious"}]
 (ARTIFACT_DIR/f"axe-{name}.json").write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding="utf-8")
 summary={"violations":len(result["violations"]),"incomplete":len(result["incomplete"]),"unresolvedCriticalSeriousIncomplete":len(unresolved),"passes":len(result["passes"])}
 if result["violations"] or unresolved:
  details=[{"id":v["id"],"impact":v.get("impact"),"targets":[n["target"] for n in v["nodes"]]} for v in result["violations"]]
  raise AssertionError(f"{name}: axe {summary} {details}")
 return summary

def open_sales(page:Page,french:bool)->None:
 page.goto(BASE_URL,wait_until="networkidle")
 if french: page.get_by_role("button",name="Français",exact=True).click()
 page.get_by_role("button",name="Ventes" if french else "المبيعات",exact=True).click()
 page.locator(".p7-workspace").wait_for()
 assert page.locator("html").get_attribute("dir")==("ltr" if french else "rtl")

def run_case(browser,name:str,width:int,height:int,french:bool,action:Callable[[Page],None])->dict:
 page=browser.new_page(viewport={"width":width,"height":height});page.on("dialog",lambda dialog:dialog.accept());page.add_init_script(MOCK);page.add_init_script("window.__POSMAN_E2E_ERRORS__=[];addEventListener('error',e=>window.__POSMAN_E2E_ERRORS__.push(String(e.error||e.message)));addEventListener('unhandledrejection',e=>window.__POSMAN_E2E_ERRORS__.push(String(e.reason)));const old=console.error;console.error=(...a)=>{window.__POSMAN_E2E_ERRORS__.push(a.map(String).join(' '));old(...a)}")
 try:
  open_sales(page,french);action(page);no_overflow(page,name);no_clipping(page,name);page.screenshot(path=str(ARTIFACT_DIR/f"{name}.png"),full_page=True);report=axe(page,name);errors=page.evaluate("()=>window.__POSMAN_E2E_ERRORS__");
  if errors:raise AssertionError(f"{name}: browser errors {errors}")
  return{"name":name,"axe":report,"calls":page.evaluate("()=>window.__POSMAN_PHASE07_CALLS__")}
 finally: page.close()

def main()->int:
 ARTIFACT_DIR.mkdir(parents=True,exist_ok=True);server=subprocess.Popen(["npm","run","dev","--","--host","127.0.0.1"],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,start_new_session=True)
 try:
  wait_server();reports=[]
  with sync_playwright() as p:
   browser=p.chromium.launch()
   reports.append(run_case(browser,"phase07-ar-today-1280x800",1280,800,False,lambda page:page.locator("[data-testid='phase07-today']").wait_for()))
   reports.append(run_case(browser,"phase07-fr-today-1280x800",1280,800,True,lambda page:page.locator("[data-testid='phase07-today']").wait_for()))
   def delivery(page:Page)->None:
    page.get_by_role("button",name="التسليم الجزئي والكامل",exact=True).click();page.locator("input[name='quantity']").fill("8");page.get_by_role("button",name="ترحيل التسليم",exact=True).click();page.get_by_text("المستند المرحّل مقفل ولا يُعدّل.").wait_for()
   reports.append(run_case(browser,"phase07-ar-partial-delivery-1024x640",1024,640,False,delivery))
   def direct(page:Page)->None:
    page.get_by_role("button",name="Vente directe",exact=True).click();page.locator("input[name='quantity']").fill("2");page.locator("input[name='reason']").fill("Dérogation marge approuvée");page.get_by_role("button",name="Valider la vente directe",exact=True).click();page.get_by_text("Le document validé est verrouillé.").wait_for()
   reports.append(run_case(browser,"phase07-fr-direct-sale-1024x640",1024,640,True,direct))
   def return_flow(page:Page)->None:
    page.get_by_role("button",name="المرتجعات والإشعارات الدائنة",exact=True).click();page.get_by_role("button",name="ترحيل المرتجع والإشعار",exact=True).click();page.get_by_text("المستند المرحّل مقفل ولا يُعدّل.").wait_for()
   reports.append(run_case(browser,"phase07-ar-return-credit-1280x800",1280,800,False,return_flow))
   reports.append(run_case(browser,"phase07-fr-lineage-1280x800",1280,800,True,lambda page:page.get_by_role("button",name="Traçabilité documentaire",exact=True).click()))
   browser.close()
  (ARTIFACT_DIR/"phase07-e2e-summary.json").write_text(json.dumps(reports,ensure_ascii=False,indent=2),encoding="utf-8");print(json.dumps(reports,ensure_ascii=False,indent=2));return 0
 finally:
  if server.poll() is None:
   os.killpg(server.pid,signal.SIGTERM)
   try:server.wait(timeout=10)
   except subprocess.TimeoutExpired:os.killpg(server.pid,signal.SIGKILL)
  if server.stdout:(ARTIFACT_DIR/"phase07-vite.log").write_text(server.stdout.read(),encoding="utf-8")

if __name__=="__main__":raise SystemExit(main())
