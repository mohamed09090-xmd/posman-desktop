import { invoke, isTauri } from "@tauri-apps/api/core";
import type { InvokeFunction } from "./runtime-status";
import type { DocumentQuery, DocumentView, EntityResult } from "./phase06";

export interface SalesLineInput { productId:string; warehouseId?:string; quantityScaled:number; unitPriceScaled:number; discountRateScaled:number; taxRateId?:string }
export interface TransformLineInput { sourceLineId:string; quantityScaled:number }
export interface IdempotentRequest<T> { idempotencyKey:string; payload:T }
export interface SalesFlowResult { primary:EntityResult; relatedDocumentIds:string[] }
export interface SalesSummary { draftOrders:number; confirmedOrders:number; partialOrders:number; uninvoicedDeliveries:number; postedInvoices:number; belowCostOverrides:number }
export interface SalesLineAvailability { sourceLineId:string; productId:string; originalQuantityScaled:number; deliveredQuantityScaled:number; invoicedQuantityScaled:number; returnedQuantityScaled:number; remainingQuantityScaled:number }

export const PHASE07_COMMANDS = [
  "create_sales_order","update_sales_order","confirm_sales_order","hold_sales_order","resume_sales_order","cancel_sales_order",
  "deliver_sales_order","invoice_sales_delivery","direct_sale","post_sales_return",
  "list_sales_documents","get_sales_document","get_sales_line_availability","get_sales_summary",
] as const;
export type Phase07Command = typeof PHASE07_COMMANDS[number];

const SAFE_CODES=new Set([
  "VALIDATION_FAILED","OPERATION_FAILED","AUTHENTICATION_REQUIRED","PERMISSION_DENIED","CONCURRENCY_CONFLICT",
  "IDEMPOTENCY_CONFLICT","INSUFFICIENT_STOCK","RESERVED_STOCK_CONFLICT","TRANSFORMATION_LIMIT_EXCEEDED",
  "POSTED_DOCUMENT_LOCKED","NOT_FOUND","NUMERIC_OVERFLOW","CUSTOMER_REQUIRED","RESERVATION_REQUIRED",
  "BELOW_COST_BLOCKED","BELOW_COST_OVERRIDE_REQUIRED","DELIVERED_ORDER_CANNOT_CANCEL","REQUEST_IN_PROGRESS",
]);
export class Phase07GatewayError extends Error { readonly code:string; constructor(code:string){super("The local sales operation could not be completed.");this.name="Phase07GatewayError";this.code=code;} }
export function normalizePhase07Error(error:unknown):Phase07GatewayError{if(error instanceof Phase07GatewayError)return error;if(typeof error==="object"&&error!==null&&"code"in error){const code=String((error as {code:unknown}).code);if(SAFE_CODES.has(code))return new Phase07GatewayError(code);}return new Phase07GatewayError("OPERATION_FAILED");}
function record(value:unknown):Record<string,unknown>{if(typeof value!=="object"||value===null||Array.isArray(value))throw new Phase07GatewayError("OPERATION_FAILED");return value as Record<string,unknown>}
function array(value:unknown):unknown[]{if(!Array.isArray(value))throw new Phase07GatewayError("OPERATION_FAILED");return value}
function text(value:unknown):string{if(typeof value!=="string"||!value.trim())throw new Phase07GatewayError("OPERATION_FAILED");return value}
function optionalText(value:unknown):string|undefined{return value==null?undefined:text(value)}
function integer(value:unknown):number{if(typeof value!=="number"||!Number.isSafeInteger(value))throw new Phase07GatewayError("OPERATION_FAILED");return value}
function nonNegative(value:unknown):number{const result=integer(value);if(result<0)throw new Phase07GatewayError("OPERATION_FAILED");return result}
function bool(value:unknown):boolean{if(typeof value!=="boolean")throw new Phase07GatewayError("OPERATION_FAILED");return value}
export function validateEntity(value:unknown):EntityResult{const row=record(value);return{id:text(row.id),documentNumber:optionalText(row.documentNumber),status:text(row.status),rowVersion:nonNegative(row.rowVersion),replayed:bool(row.replayed)}}
function validateLine(value:unknown){const row=record(value);return{id:text(row.id),sourceLineId:optionalText(row.sourceLineId),productId:text(row.productId),productCode:text(row.productCode),description:text(row.description),warehouseId:optionalText(row.warehouseId),quantityScaled:integer(row.quantityScaled),unitPriceScaled:nonNegative(row.unitPriceScaled),unitCostScaled:row.unitCostScaled==null?undefined:nonNegative(row.unitCostScaled),taxRateScaled:nonNegative(row.taxRateScaled),lineHtMinor:integer(row.lineHtMinor),lineTaxMinor:integer(row.lineTaxMinor),lineTtcMinor:integer(row.lineTtcMinor),notes:optionalText(row.notes)}}
export function validateDocument(value:unknown):DocumentView{const row=record(value);return{id:text(row.id),documentType:text(row.documentType),documentNumber:text(row.documentNumber),workflowStatus:text(row.workflowStatus),postingStatus:text(row.postingStatus),commercialDate:text(row.commercialDate),partnerId:optionalText(row.partnerId),warehouseId:optionalText(row.warehouseId),sourceDocumentId:optionalText(row.sourceDocumentId),totalHtMinor:integer(row.totalHtMinor),totalTaxMinor:integer(row.totalTaxMinor),totalTtcMinor:integer(row.totalTtcMinor),notes:optionalText(row.notes),rowVersion:nonNegative(row.rowVersion),lines:array(row.lines).map(validateLine)}}
export function validateFlow(value:unknown):SalesFlowResult{const row=record(value);return{primary:validateEntity(row.primary),relatedDocumentIds:array(row.relatedDocumentIds).map(text)}}
export function validateSummary(value:unknown):SalesSummary{const row=record(value);return{draftOrders:nonNegative(row.draftOrders),confirmedOrders:nonNegative(row.confirmedOrders),partialOrders:nonNegative(row.partialOrders),uninvoicedDeliveries:nonNegative(row.uninvoicedDeliveries),postedInvoices:nonNegative(row.postedInvoices),belowCostOverrides:nonNegative(row.belowCostOverrides)}}
export function validateAvailability(value:unknown):SalesLineAvailability[]{return array(value).map(item=>{const row=record(item);return{sourceLineId:text(row.sourceLineId),productId:text(row.productId),originalQuantityScaled:nonNegative(row.originalQuantityScaled),deliveredQuantityScaled:nonNegative(row.deliveredQuantityScaled),invoicedQuantityScaled:nonNegative(row.invoicedQuantityScaled),returnedQuantityScaled:nonNegative(row.returnedQuantityScaled),remainingQuantityScaled:nonNegative(row.remainingQuantityScaled)}})}
function response(command:Phase07Command,value:unknown):unknown{switch(command){case"direct_sale":case"post_sales_return":return validateFlow(value);case"list_sales_documents":return array(value).map(validateDocument);case"get_sales_document":return validateDocument(value);case"get_sales_line_availability":return validateAvailability(value);case"get_sales_summary":return validateSummary(value);default:return validateEntity(value)}}

export interface Phase07Gateway { call<T>(command:Phase07Command,request?:unknown):Promise<T>; documents(query?:DocumentQuery):Promise<DocumentView[]>; summary():Promise<SalesSummary> }
export function createPhase07Gateway(invoker:InvokeFunction):Phase07Gateway{const call=async<T,>(command:Phase07Command,request?:unknown):Promise<T>=>{try{const value=request===undefined?await invoker(command):await invoker(command,{request});return response(command,value) as T}catch(error){throw normalizePhase07Error(error)}};return{call,documents:(query={})=>call("list_sales_documents",query),summary:()=>call("get_sales_summary")}}
declare global { interface Window { __POSMAN_DEV_PHASE07_INVOKER__?:InvokeFunction } }
export function resolvePhase07Gateway():Phase07Gateway|null{if(import.meta.env.DEV&&typeof window!=="undefined"&&typeof window.__POSMAN_DEV_PHASE07_INVOKER__==="function")return createPhase07Gateway(window.__POSMAN_DEV_PHASE07_INVOKER__);return isTauri()?createPhase07Gateway(invoke):null}
