import { useCallback, useEffect, useMemo, useState, type ButtonHTMLAttributes, type FormEvent, type InputHTMLAttributes, type ReactNode } from "react";
import { useI18n } from "../../i18n/I18nProvider";
import {
  Phase06GatewayError,
  resolvePhase06Gateway,
  type DocumentView,
  type EntityResult,
  type MovementView,
  type ReconciliationView,
  type ReservationView,
  type Phase06Command,
  type StockBalanceView,
} from "../../platform/tauri/phase06";
import { PHASE06_COPY, type Phase06Screen } from "./copy";
import "./phase06.css";

type Gateway = ReturnType<typeof resolvePhase06Gateway>;
type Flash = { tone: "success" | "error" | "warning"; text: string };
const inventory: Phase06Screen[] = ["overview", "ledger", "opening", "transfer", "adjustment", "count", "reservations", "reconciliation"];
const purchasing: Phase06Screen[] = ["orders", "orderEditor", "receipt", "invoice", "direct", "return"];
const today = "2026-08-05";
const preview: StockBalanceView[] = [
  { productId:"product-oil",productCode:"HUI-001",productName:"زيت مائدة / Huile 1 L",warehouseId:"warehouse-main",warehouseName:"المستودع الرئيسي / Dépôt principal",onHandScaled:24000000,reservedScaled:3000000,availableScaled:21000000,averageCostScaled:5240000,inventoryValueMinor:1257600,rowVersion:3 },
];

const idempotency = (prefix: string) => `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
const scaled = (form: FormData, name: string, factor: number) => Math.round(Number(form.get(name) ?? 0) * factor);
const string = (form: FormData, name: string) => String(form.get(name) ?? "").trim();
const optional = (form: FormData, name: string) => string(form, name) || undefined;
const safeMessage = (error: unknown, fallback: string) => error instanceof Phase06GatewayError ? `${fallback} (${error.code})` : fallback;

function Button(props: ButtonHTMLAttributes<HTMLButtonElement>) { return <button className="p6-button" {...props} />; }
function Input(props: InputHTMLAttributes<HTMLInputElement>) { return <input className="p6-input" {...props} />; }
function Field({ label, children }: { label: string; children: ReactNode }) { return <label className="p6-field"><span>{label}</span>{children}</label>; }
function Status({ children }: { children: ReactNode }) { return <span className="p6-status"><span aria-hidden="true">●</span>{children}</span>; }

export function Phase06Workspace() {
  const { locale } = useI18n();
  const copy = PHASE06_COPY[locale];
  const gateway = useMemo(resolvePhase06Gateway, []);
  const [screen, setScreen] = useState<Phase06Screen>("overview");
  const [flash, setFlash] = useState<Flash>();
  return <section className="p6-workspace" aria-labelledby="p6-heading">
    <header className="p6-commandbar"><div><p>POSMAN / PHASE 06</p><h1 id="p6-heading">{copy.title}</h1></div><Status>{gateway ? "SQLite · local" : copy.preview}</Status></header>
    <div className="p6-layout">
      <nav className="p6-process" aria-label={copy.title}>
        <h2>{copy.inventory}</h2>
        {inventory.map((id,index)=><button key={id} className={screen===id?"is-active":""} onClick={()=>{setFlash(undefined);setScreen(id);}}><span aria-hidden="true">{String(index+1).padStart(2,"0")}</span>{copy[id]}</button>)}
        <h2>{copy.purchasing}</h2>
        {purchasing.map((id,index)=><button key={id} className={screen===id?"is-active":""} onClick={()=>{setFlash(undefined);setScreen(id);}}><span aria-hidden="true">{String(index+9).padStart(2,"0")}</span>{copy[id]}</button>)}
      </nav>
      <main className="p6-canvas" data-testid={`phase06-${screen}`} aria-labelledby="p6-screen-heading">
        <header className="p6-canvas__header"><div><p>{copy.title}</p><h2 id="p6-screen-heading">{copy[screen]}</h2></div></header>
        {flash?<div className={`p6-flash p6-flash--${flash.tone}`} role={flash.tone==="error"?"alert":"status"}>{flash.text}</div>:null}
        {screen==="overview"?<Overview gateway={gateway}/>:null}
        {screen==="ledger"?<Ledger gateway={gateway}/>:null}
        {screen==="orders"?<Documents gateway={gateway}/>:null}
        {screen==="reservations"?<Reservations gateway={gateway} flash={setFlash}/>:null}
        {screen==="reconciliation"?<Reconciliation gateway={gateway} flash={setFlash}/>:null}
        {!(["overview","ledger","orders","reservations","reconciliation"] as Phase06Screen[]).includes(screen)?<Operation screen={screen} gateway={gateway} flash={setFlash}/>:null}
      </main>
    </div>
  </section>;
}

function useLoad<T>(loader:()=>Promise<T>, initial:T) {
  const [value,setValue]=useState(initial); const [state,setState]=useState<"loading"|"ready"|"empty"|"error">("loading");
  const load=useCallback(async()=>{setState("loading");try{const next=await loader();setValue(next);setState(Array.isArray(next)&&next.length===0?"empty":"ready");}catch{setState("error");}},[loader]);
  useEffect(()=>{let active=true;void loader().then(next=>{if(active){setValue(next);setState(Array.isArray(next)&&next.length===0?"empty":"ready");}}).catch(()=>{if(active)setState("error");});return()=>{active=false;};},[loader]);
  return {value,state,load};
}
function Boundary({state,retry,children}:{state:"loading"|"ready"|"empty"|"error";retry:()=>Promise<void>;children:ReactNode}){const{locale}=useI18n();const c=PHASE06_COPY[locale];if(state==="loading")return <div className="p6-state" aria-busy="true">{c.loading}</div>;if(state==="empty")return <div className="p6-state">{c.empty}</div>;if(state==="error")return <div className="p6-state" role="alert">{c.error}<Button onClick={()=>void retry()}>{c.retry}</Button></div>;return <>{children}</>;}

function Overview({gateway}:{gateway:Gateway}){const{locale}=useI18n();const c=PHASE06_COPY[locale];const loader=useCallback(()=>gateway?gateway.balances({limit:500}):Promise.resolve(preview),[gateway]);const{value,state,load}=useLoad(loader,[] as StockBalanceView[]);const qty=new Intl.NumberFormat(locale,{maximumFractionDigits:6});const money=new Intl.NumberFormat(locale,{style:"currency",currency:"DZD"});return <Boundary state={state} retry={load}><div className="p6-table-wrap"><table className="p6-table"><thead><tr><th>{c.product}</th><th>{c.warehouse}/{c.location}</th><th>{c.onHand}</th><th>{c.reserved}</th><th>{c.available}</th><th>{c.cump}</th><th>{c.value}</th></tr></thead><tbody>{value.map(row=><tr key={`${row.productId}-${row.warehouseLocationId??"all"}`}><td><strong>{row.productCode}</strong><small>{row.productName}</small></td><td>{row.warehouseName}<small>{row.locationName??"—"}</small></td><td>{qty.format(row.onHandScaled/1e6)}</td><td>{qty.format(row.reservedScaled/1e6)}</td><td>{qty.format(row.availableScaled/1e6)}</td><td>{money.format(row.averageCostScaled/1e4)}</td><td>{money.format(row.inventoryValueMinor/100)}</td></tr>)}</tbody></table></div></Boundary>}
function Ledger({gateway}:{gateway:Gateway}){const{locale}=useI18n();const c=PHASE06_COPY[locale];const loader=useCallback(()=>gateway?gateway.movements({limit:500}):Promise.resolve([] as MovementView[]),[gateway]);const{value,state,load}=useLoad(loader,[] as MovementView[]);return <Boundary state={state} retry={load}><div className="p6-table-wrap"><table className="p6-table"><thead><tr><th>{c.date}</th><th>{c.movement}</th><th>{c.product}</th><th>{c.quantity}</th><th>{c.cump}</th><th>{c.document}</th></tr></thead><tbody>{value.map(row=><tr key={row.id}><td>{new Intl.DateTimeFormat(locale).format(new Date(`${row.businessDate}T00:00:00`))}</td><td><Status>{row.movementType}</Status></td><td><code>{row.productId}</code></td><td>{row.quantityDeltaScaled/1e6}</td><td>{(row.averageCostAfterScaled??0)/1e4}</td><td><code>{row.sourceDocumentId??"—"}</code></td></tr>)}</tbody></table></div></Boundary>}
function Documents({gateway}:{gateway:Gateway}){const{locale}=useI18n();const c=PHASE06_COPY[locale];const loader=useCallback(()=>gateway?gateway.call<DocumentView[]>("list_purchasing_documents",{limit:200}):Promise.resolve([] as DocumentView[]),[gateway]);const{value,state,load}=useLoad(loader,[] as DocumentView[]);return <Boundary state={state} retry={load}><div className="p6-table-wrap"><table className="p6-table"><thead><tr><th>{c.document}</th><th>{c.date}</th><th>{c.supplier}</th><th>{c.status}</th><th>HT</th><th>TTC</th></tr></thead><tbody>{value.map(row=><tr key={row.id}><td><strong>{row.documentNumber}</strong><small>{row.documentType}</small></td><td>{row.commercialDate}</td><td>{row.partnerId??"—"}</td><td><Status>{row.workflowStatus}</Status></td><td>{row.totalHtMinor/100}</td><td>{row.totalTtcMinor/100}</td></tr>)}</tbody></table></div></Boundary>}

function Reservations({gateway,flash}:{gateway:Gateway;flash:(f:Flash)=>void}){const{locale}=useI18n();const c=PHASE06_COPY[locale];const loader=useCallback(()=>gateway?gateway.call<ReservationView[]>("list_active_stock_reservations"):Promise.resolve([] as ReservationView[]),[gateway]);const{value,state,load}=useLoad(loader,[] as ReservationView[]);const act=async(row:ReservationView)=>{if(!gateway)return;try{await gateway.call("release_stock_reservation",{idempotencyKey:idempotency("release"),payload:{reservationId:row.id,rowVersion:row.rowVersion}});flash({tone:"success",text:c.success});await load();}catch(error){flash({tone:"error",text:safeMessage(error,c.error)});}};return <Boundary state={state} retry={load}><div className="p6-table-wrap"><table className="p6-table"><thead><tr><th>{c.product}</th><th>{c.warehouse}</th><th>{c.quantity}</th><th>{c.status}</th><th/></tr></thead><tbody>{value.map(row=><tr key={row.id}><td>{row.productId}</td><td>{row.warehouseId}</td><td>{row.reservedQuantityScaled/1e6}</td><td><Status>{row.status}</Status></td><td><Button onClick={()=>void act(row)}>{c.release}</Button></td></tr>)}</tbody></table></div></Boundary>}
function Reconciliation({gateway,flash}:{gateway:Gateway;flash:(f:Flash)=>void}){const{locale}=useI18n();const c=PHASE06_COPY[locale];const loader=useCallback(()=>gateway?gateway.call<ReconciliationView>("reconcile_stock_balances"):Promise.resolve({rows:[],mismatchCount:0,rebuilt:false}),[gateway]);const{value,state,load}=useLoad(loader,{rows:[],mismatchCount:0,rebuilt:false} as ReconciliationView);const rebuild=async()=>{if(!gateway)return;try{const next=await gateway.call<ReconciliationView>("rebuild_stock_balances",{idempotencyKey:idempotency("rebuild"),payload:{}});flash({tone:"success",text:next.mismatchCount===0?"OK":c.mismatch});await load();}catch(error){flash({tone:"error",text:safeMessage(error,c.error)});}};return <Boundary state={state} retry={load}><div className="p6-callout"><strong>{value.mismatchCount?c.mismatch:"OK"}</strong><Button onClick={()=>void rebuild()}>{c.rebuild}</Button></div><div className="p6-table-wrap"><table className="p6-table"><thead><tr><th>{c.product}</th><th>{c.warehouse}</th><th>{c.projection}</th><th>{c.rebuilt}</th><th>{c.status}</th></tr></thead><tbody>{value.rows.map(row=><tr key={`${row.productId}-${row.warehouseId}`}><td>{row.productId}</td><td>{row.warehouseId}</td><td>{row.projectionOnHandScaled/1e6}</td><td>{row.rebuiltOnHandScaled/1e6}</td><td>{row.matches?"OK":c.mismatch}</td></tr>)}</tbody></table></div></Boundary>}

function Operation({screen,gateway,flash}:{screen:Phase06Screen;gateway:Gateway;flash:(f:Flash)=>void}){
  const{locale}=useI18n();const c=PHASE06_COPY[locale];const[result,setResult]=useState<EntityResult>();const[busy,setBusy]=useState(false);const[negative,setNegative]=useState(false);
  useEffect(()=>{setResult(undefined);setNegative(false);},[screen]);
  const invoke=async(command:Phase06Command,request:unknown)=>{if(!gateway){setResult({id:`preview-${command}`,documentNumber:"P6-PREVIEW",status:"DRAFT",rowVersion:1,replayed:false});return;}const next=await gateway.call<EntityResult>(command,request);setResult(next);};
  const submit=async(event:FormEvent<HTMLFormElement>)=>{event.preventDefault();const form=new FormData(event.currentTarget);const quantity=scaled(form,"quantity",1e6);setNegative(quantity<0);if(["transfer","adjustment","direct","return"].includes(screen)&&!window.confirm(c.confirmPosting))return;setBusy(true);try{await invoke(commandFor(screen),requestFor(screen,form));flash({tone:"success",text:c.success});}catch(error){flash({tone:"error",text:safeMessage(error,c.error)});}finally{setBusy(false);}};
  const advance=async(action:"review"|"post"|"confirm")=>{if(!result||!gateway)return;if((action==="post"||action==="confirm")&&!window.confirm(c.confirmPosting))return;setBusy(true);try{const payload={documentId:result.id,rowVersion:result.rowVersion};const command=advanceCommand(screen,action);const request=action==="review"?payload:{idempotencyKey:idempotency(command),payload};const next=await gateway.call<EntityResult>(command,request);setResult(next);flash({tone:"success",text:c.success});}catch(error){flash({tone:"error",text:safeMessage(error,c.error)});}finally{setBusy(false);}};
  const needsWarehouse=["opening","transfer","adjustment","count","receipt","direct","return"].includes(screen);const needsSupplier=["orderEditor","receipt","invoice","direct","return"].includes(screen);const needsLine=["opening","transfer","adjustment","count","orderEditor","receipt","invoice","direct","return"].includes(screen);
  return <form className="p6-form" onSubmit={event=>void submit(event)}>
    {screen==="direct"?<div className="p6-callout">{c.directNote}</div>:null}
    {needsWarehouse?<Field label={c.warehouse}><Input name="warehouse" defaultValue="warehouse-main" required/></Field>:null}
    {screen==="transfer"?<><Field label={c.source}><Input name="sourceWarehouse" defaultValue="warehouse-main" required/></Field><Field label={c.destination}><Input name="destinationWarehouse" defaultValue="warehouse-secondary" required/></Field></>:null}
    {needsSupplier?<Field label={c.supplier}><Input name="supplier" defaultValue="supplier-1" required/></Field>:null}
    {(screen==="receipt"||screen==="return")?<Field label={c.documentId}><Input name="sourceDocument" defaultValue={screen==="receipt"?"purchase-order-1":"purchase-receipt-1"}/></Field>:null}
    {(screen==="invoice"||screen==="return")?<Field label={c.sourceLineId}><Input name="sourceLine" defaultValue="source-line-1" required/></Field>:null}
    {needsLine?<><Field label={c.product}><Input name="product" defaultValue="product-oil" required/></Field><Field label={c.quantity}><Input name="quantity" type="number" step="0.000001" defaultValue="2" required/></Field></>:null}
    {["opening","adjustment","count","receipt","direct"].includes(screen)?<Field label={c.cost}><Input name="cost" type="number" step="0.0001" defaultValue="520"/></Field>:null}
    {["orderEditor","receipt","invoice","direct","return"].includes(screen)?<Field label={c.price}><Input name="price" type="number" step="0.0001" defaultValue="520" required/></Field>:null}
    {screen==="count"?<Field label={c.countNumber}><Input name="countNumber" defaultValue="INV-2026-001" required/></Field>:null}
    <Field label={c.date}><Input name="date" type="date" defaultValue={today} required/></Field>
    {["adjustment","transfer","return"].includes(screen)?<Field label={c.reason}><Input name="reason" defaultValue="Correction opérationnelle" required/></Field>:null}
    <Field label={c.notes}><Input name="notes"/></Field>
    {(negative||screen==="adjustment"||screen==="return")?<fieldset className="p6-warning"><legend>{c.negativeWarning}</legend><label><input name="override" type="checkbox"/> {c.overrideConfirm}</label></fieldset>:null}
    <div className="p6-action-dock"><Button type="submit" disabled={busy}>{busy?c.loading:(["opening","count","orderEditor","receipt","invoice"].includes(screen)?c.save:c.post)}</Button>
      {result&&screen==="opening"&&result.status==="DRAFT"?<Button type="button" onClick={()=>void advance("review")}>{c.review}</Button>:null}
      {result&&screen==="opening"&&result.status==="REVIEWED"?<Button type="button" onClick={()=>void advance("post")}>{c.post}</Button>:null}
      {result&&screen==="count"&&result.status!=="POSTED"?<Button type="button" onClick={()=>void advance(result.status==="REVIEWED"?"post":"review")}>{result.status==="REVIEWED"?c.post:c.review}</Button>:null}
      {result&&screen==="orderEditor"&&result.status==="DRAFT"?<Button type="button" onClick={()=>void advance("confirm")}>{c.confirm}</Button>:null}
      {result&&(screen==="receipt"||screen==="invoice")&&result.status==="DRAFT"?<Button type="button" onClick={()=>void advance("post")}>{c.post}</Button>:null}
      {result?<><Status>{result.status}</Status><code>{result.documentNumber??result.id}</code>{result.status==="POSTED"?<span>{c.postedLocked}</span>:null}</>:null}
    </div>
  </form>;
}

function commandFor(screen:Phase06Screen){const map={opening:"create_opening_stock",transfer:"post_stock_transfer",adjustment:"post_stock_adjustment",count:"create_inventory_count",orderEditor:"create_purchase_order",receipt:"create_purchase_receipt",invoice:"create_purchase_invoice",direct:"direct_receive_and_invoice",return:"post_purchase_return"} as const;if(!(screen in map))throw new Phase06GatewayError("OPERATION_FAILED");return map[screen as keyof typeof map];}
function advanceCommand(screen:Phase06Screen,action:"review"|"post"|"confirm"){if(screen==="opening")return action==="review"?"review_opening_stock":"post_opening_stock";if(screen==="count")return action==="review"?"review_inventory_count":"post_inventory_count";if(screen==="orderEditor")return "confirm_purchase_order";if(screen==="receipt")return "post_purchase_receipt";if(screen==="invoice")return "post_purchase_invoice";throw new Phase06GatewayError("OPERATION_FAILED");}
function requestFor(screen:Phase06Screen,form:FormData){const productId=string(form,"product"),quantityScaled=scaled(form,"quantity",1e6),warehouseId=string(form,"warehouse"),commercialDate=string(form,"date"),unitCostScaled=scaled(form,"cost",1e4)||undefined,unitPriceScaled=scaled(form,"price",1e4),reason=string(form,"reason"),allowNegativeOverride=form.get("override")==="on";const stockLine={productId,quantityScaled,unitCostScaled};const purchaseLine={sourceLineId:optional(form,"sourceLine"),productId,warehouseId:warehouseId||undefined,quantityScaled,unitPriceScaled,unitCostScaled,discountRateScaled:0};switch(screen){case"opening":return{warehouseId,commercialDate,notes:optional(form,"notes"),lines:[stockLine]};case"adjustment":return{idempotencyKey:idempotency("adjustment"),payload:{warehouseId,commercialDate,reason,allowNegativeOverride,lines:[stockLine]}};case"transfer":return{idempotencyKey:idempotency("transfer"),payload:{sourceWarehouseId:string(form,"sourceWarehouse"),destinationWarehouseId:string(form,"destinationWarehouse"),commercialDate,reason,allowNegativeOverride,lines:[{productId,quantityScaled}]}};case"count":return{warehouseId,countNumber:string(form,"countNumber"),commercialDate,notes:optional(form,"notes"),lines:[{productId,countedQuantityScaled:quantityScaled,unitCostScaled}]};case"orderEditor":return{supplierId:string(form,"supplier"),commercialDate,notes:optional(form,"notes"),lines:[purchaseLine]};case"receipt":return{purchaseOrderId:optional(form,"sourceDocument"),supplierId:string(form,"supplier"),warehouseId,commercialDate,notes:optional(form,"notes"),lines:[purchaseLine]};case"invoice":return{supplierId:string(form,"supplier"),commercialDate,notes:optional(form,"notes"),lines:[purchaseLine]};case"direct":return{idempotencyKey:idempotency("direct"),payload:{supplierId:string(form,"supplier"),warehouseId,commercialDate,notes:optional(form,"notes"),lines:[purchaseLine]}};case"return":return{idempotencyKey:idempotency("return"),payload:{sourceDocumentId:string(form,"sourceDocument"),supplierId:string(form,"supplier"),warehouseId,commercialDate,reason,allowNegativeOverride,lines:[purchaseLine]}};default:throw new Phase06GatewayError("OPERATION_FAILED");}}
