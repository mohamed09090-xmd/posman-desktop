import { invoke, isTauri } from "@tauri-apps/api/core";
import type { InvokeFunction } from "./runtime-status";

export interface Phase06ErrorPayload { code: string; message?: string }
export class Phase06GatewayError extends Error {
  readonly code: string;
  constructor(code: string) {
    super("The local inventory or purchasing operation could not be completed.");
    this.name = "Phase06GatewayError";
    this.code = code;
  }
}

export interface IdempotentRequest<T> { idempotencyKey: string; payload: T }
export interface StockQuery { productId?: string; warehouseId?: string; warehouseLocationId?: string; limit?: number }
export interface DocumentQuery { documentType?: string; status?: string; search?: string; limit?: number }
export interface StockLineInput { productId: string; warehouseLocationId?: string; quantityScaled: number; unitCostScaled?: number }
export interface PurchaseLineInput { sourceLineId?: string; productId: string; warehouseId?: string; quantityScaled: number; unitPriceScaled: number; unitCostScaled?: number; discountRateScaled: number; taxRateId?: string }
export interface EntityResult { id: string; documentNumber?: string; status: string; rowVersion: number; replayed: boolean }
export interface StockBalanceView {
  productId: string; productCode: string; productName: string; warehouseId: string; warehouseName: string;
  warehouseLocationId?: string; locationName?: string; onHandScaled: number; reservedScaled: number;
  availableScaled: number; averageCostScaled: number; inventoryValueMinor: number; rowVersion: number;
}
export interface MovementView {
  id: string; productId: string; warehouseId: string; warehouseLocationId?: string; sourceDocumentId?: string;
  movementType: string; businessDate: string; quantityDeltaScaled: number; quantityAfterScaled: number;
  unitCostScaled?: number; averageCostAfterScaled?: number; extendedCostMinor?: number; notes?: string;
}
export interface ReservationView {
  id: string; sourceLineId: string; productId: string; warehouseId: string; warehouseLocationId?: string;
  reservedQuantityScaled: number; status: string; rowVersion: number;
}
export interface DocumentLineView {
  id: string; sourceLineId?: string; productId: string; productCode: string; description: string; warehouseId?: string;
  quantityScaled: number; unitPriceScaled: number; unitCostScaled?: number; taxRateScaled: number;
  lineHtMinor: number; lineTaxMinor: number; lineTtcMinor: number; notes?: string;
}
export interface DocumentView {
  id: string; documentType: string; documentNumber: string; workflowStatus: string; postingStatus: string;
  commercialDate: string; partnerId?: string; warehouseId?: string; sourceDocumentId?: string;
  totalHtMinor: number; totalTaxMinor: number; totalTtcMinor: number; notes?: string; rowVersion: number;
  lines: DocumentLineView[];
}
export interface CountLineView {
  id: string; productId: string; warehouseLocationId?: string; systemQuantityScaled: number;
  countedQuantityScaled: number; varianceQuantityScaled: number; unitCostScaled?: number; rowVersion: number;
}
export interface CountView {
  id: string; warehouseId: string; countNumber: string; commercialDate: string; status: string;
  rowVersion: number; lines: CountLineView[];
}
export interface ReconciliationRow {
  productId: string; warehouseId: string; warehouseLocationId?: string;
  projectionOnHandScaled: number; rebuiltOnHandScaled: number;
  projectionReservedScaled: number; rebuiltReservedScaled: number;
  projectionAverageCostScaled: number; rebuiltAverageCostScaled: number; matches: boolean;
}
export interface ReconciliationView { rows: ReconciliationRow[]; mismatchCount: number; rebuilt: boolean }

export const PHASE06_COMMANDS = [
  "list_stock_balances", "list_stock_movements", "create_opening_stock", "review_opening_stock",
  "post_opening_stock", "post_stock_adjustment", "post_stock_transfer", "create_inventory_count",
  "update_inventory_count", "review_inventory_count", "post_inventory_count", "get_inventory_count",
  "create_stock_reservation", "release_stock_reservation", "consume_stock_reservation",
  "cancel_stock_reservation", "list_active_stock_reservations", "reconcile_stock_balances",
  "rebuild_stock_balances", "create_purchase_order", "update_purchase_order", "confirm_purchase_order",
  "cancel_purchase_order", "hold_purchase_order", "create_purchase_receipt", "post_purchase_receipt",
  "create_purchase_invoice", "post_purchase_invoice", "direct_receive_and_invoice",
  "post_purchase_return", "list_purchasing_documents", "get_purchasing_document",
] as const;
export type Phase06Command = typeof PHASE06_COMMANDS[number];

const SAFE_CODES = new Set([
  "VALIDATION_ERROR", "VALIDATION_FAILED", "OPERATION_FAILED", "AUTHENTICATION_REQUIRED", "PERMISSION_DENIED",
  "CONCURRENCY_CONFLICT", "IDEMPOTENCY_CONFLICT", "INSUFFICIENT_STOCK",
  "RESERVED_STOCK_CONFLICT", "NEGATIVE_STOCK_OVERRIDE_REQUIRED", "STALE_INVENTORY_COUNT",
  "POSTED_DOCUMENT_IMMUTABLE", "POSTED_DOCUMENT_LOCKED", "OVER_TRANSFORMATION", "TRANSFORMATION_LIMIT_EXCEEDED", "NOT_FOUND",
  "NUMERIC_OVERFLOW", "MOVEMENT_CONSUMPTION_EXCEEDED", "REVIEW_REQUIRED", "SUPPLIER_REQUIRED",
]);

export function normalizePhase06Error(error: unknown): Phase06GatewayError {
  if (error instanceof Phase06GatewayError) return error;
  if (typeof error === "object" && error !== null && "code" in error) {
    const code = String((error as Phase06ErrorPayload).code);
    if (SAFE_CODES.has(code)) return new Phase06GatewayError(code);
  }
  return new Phase06GatewayError("OPERATION_FAILED");
}
function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Phase06GatewayError("OPERATION_FAILED");
  return value as Record<string, unknown>;
}
function array(value: unknown): unknown[] { if (!Array.isArray(value)) throw new Phase06GatewayError("OPERATION_FAILED"); return value; }
function integer(value: unknown): number { if (typeof value !== "number" || !Number.isSafeInteger(value)) throw new Phase06GatewayError("OPERATION_FAILED"); return value; }
function nonNegativeInteger(value: unknown): number { const result = integer(value); if (result < 0) throw new Phase06GatewayError("OPERATION_FAILED"); return result; }
function text(value: unknown): string { if (typeof value !== "string" || value.trim() === "") throw new Phase06GatewayError("OPERATION_FAILED"); return value; }
function optionalText(value: unknown): string | undefined {
  if (value === undefined || value === null) return undefined;
  return text(value);
}
function boolean(value: unknown): boolean { if (typeof value !== "boolean") throw new Phase06GatewayError("OPERATION_FAILED"); return value; }

export function validateEntityResult(value: unknown): EntityResult { const row=record(value);return{id:text(row.id),documentNumber:optionalText(row.documentNumber),status:text(row.status),rowVersion:nonNegativeInteger(row.rowVersion),replayed:boolean(row.replayed)}; }
export function validateStockBalance(value: unknown): StockBalanceView {
  const row = record(value);
  const warehouseLocationId = optionalText(row.warehouseLocationId);
  const locationName = optionalText(row.locationName);
  return {
    productId: text(row.productId),
    productCode: text(row.productCode),
    productName: text(row.productName),
    warehouseId: text(row.warehouseId),
    warehouseName: text(row.warehouseName),
    ...(warehouseLocationId === undefined ? {} : { warehouseLocationId }),
    ...(locationName === undefined ? {} : { locationName }),
    onHandScaled: integer(row.onHandScaled),
    reservedScaled: nonNegativeInteger(row.reservedScaled),
    availableScaled: integer(row.availableScaled),
    averageCostScaled: nonNegativeInteger(row.averageCostScaled),
    inventoryValueMinor: integer(row.inventoryValueMinor),
    rowVersion: nonNegativeInteger(row.rowVersion),
  };
}
export function validateStockBalances(value: unknown): StockBalanceView[] { return array(value).map(validateStockBalance); }
export function validateMovement(value: unknown): MovementView { const row=record(value);return{id:text(row.id),productId:text(row.productId),warehouseId:text(row.warehouseId),warehouseLocationId:optionalText(row.warehouseLocationId),sourceDocumentId:optionalText(row.sourceDocumentId),movementType:text(row.movementType),businessDate:text(row.businessDate),quantityDeltaScaled:integer(row.quantityDeltaScaled),quantityAfterScaled:integer(row.quantityAfterScaled),unitCostScaled:row.unitCostScaled==null?undefined:nonNegativeInteger(row.unitCostScaled),averageCostAfterScaled:row.averageCostAfterScaled==null?undefined:nonNegativeInteger(row.averageCostAfterScaled),extendedCostMinor:row.extendedCostMinor==null?undefined:integer(row.extendedCostMinor),notes:optionalText(row.notes)}; }
export function validateReservation(value: unknown): ReservationView { const row=record(value);return{id:text(row.id),sourceLineId:text(row.sourceLineId),productId:text(row.productId),warehouseId:text(row.warehouseId),warehouseLocationId:optionalText(row.warehouseLocationId),reservedQuantityScaled:nonNegativeInteger(row.reservedQuantityScaled),status:text(row.status),rowVersion:nonNegativeInteger(row.rowVersion)}; }
function validateDocumentLine(value: unknown): DocumentLineView { const row=record(value);return{id:text(row.id),sourceLineId:optionalText(row.sourceLineId),productId:text(row.productId),productCode:text(row.productCode),description:text(row.description),warehouseId:optionalText(row.warehouseId),quantityScaled:integer(row.quantityScaled),unitPriceScaled:nonNegativeInteger(row.unitPriceScaled),unitCostScaled:row.unitCostScaled==null?undefined:nonNegativeInteger(row.unitCostScaled),taxRateScaled:nonNegativeInteger(row.taxRateScaled),lineHtMinor:integer(row.lineHtMinor),lineTaxMinor:integer(row.lineTaxMinor),lineTtcMinor:integer(row.lineTtcMinor),notes:optionalText(row.notes)}; }
export function validateDocument(value: unknown): DocumentView { const row=record(value);return{id:text(row.id),documentType:text(row.documentType),documentNumber:text(row.documentNumber),workflowStatus:text(row.workflowStatus),postingStatus:text(row.postingStatus),commercialDate:text(row.commercialDate),partnerId:optionalText(row.partnerId),warehouseId:optionalText(row.warehouseId),sourceDocumentId:optionalText(row.sourceDocumentId),totalHtMinor:integer(row.totalHtMinor),totalTaxMinor:integer(row.totalTaxMinor),totalTtcMinor:integer(row.totalTtcMinor),notes:optionalText(row.notes),rowVersion:nonNegativeInteger(row.rowVersion),lines:array(row.lines).map(validateDocumentLine)}; }
function validateCountLine(value: unknown): CountLineView { const row=record(value);return{id:text(row.id),productId:text(row.productId),warehouseLocationId:optionalText(row.warehouseLocationId),systemQuantityScaled:integer(row.systemQuantityScaled),countedQuantityScaled:integer(row.countedQuantityScaled),varianceQuantityScaled:integer(row.varianceQuantityScaled),unitCostScaled:row.unitCostScaled==null?undefined:nonNegativeInteger(row.unitCostScaled),rowVersion:nonNegativeInteger(row.rowVersion)}; }
export function validateCount(value: unknown): CountView { const row=record(value);return{id:text(row.id),warehouseId:text(row.warehouseId),countNumber:text(row.countNumber),commercialDate:text(row.commercialDate),status:text(row.status),rowVersion:nonNegativeInteger(row.rowVersion),lines:array(row.lines).map(validateCountLine)}; }
function validateReconciliationRow(value: unknown): ReconciliationRow { const row=record(value);return{productId:text(row.productId),warehouseId:text(row.warehouseId),warehouseLocationId:optionalText(row.warehouseLocationId),projectionOnHandScaled:integer(row.projectionOnHandScaled),rebuiltOnHandScaled:integer(row.rebuiltOnHandScaled),projectionReservedScaled:nonNegativeInteger(row.projectionReservedScaled),rebuiltReservedScaled:nonNegativeInteger(row.rebuiltReservedScaled),projectionAverageCostScaled:nonNegativeInteger(row.projectionAverageCostScaled),rebuiltAverageCostScaled:nonNegativeInteger(row.rebuiltAverageCostScaled),matches:boolean(row.matches)}; }
export function validateReconciliation(value: unknown): ReconciliationView { const row=record(value);return{rows:array(row.rows).map(validateReconciliationRow),mismatchCount:nonNegativeInteger(row.mismatchCount),rebuilt:boolean(row.rebuilt)}; }

const ENTITY_COMMANDS = new Set<Phase06Command>(["create_opening_stock","review_opening_stock","post_opening_stock","post_stock_adjustment","post_stock_transfer","post_inventory_count","create_stock_reservation","release_stock_reservation","consume_stock_reservation","cancel_stock_reservation","create_purchase_order","update_purchase_order","confirm_purchase_order","cancel_purchase_order","hold_purchase_order","create_purchase_receipt","post_purchase_receipt","create_purchase_invoice","post_purchase_invoice","direct_receive_and_invoice","post_purchase_return"]);
const COUNT_COMMANDS = new Set<Phase06Command>(["create_inventory_count","update_inventory_count","review_inventory_count","get_inventory_count"]);
export function validatePhase06Response(command: Phase06Command, value: unknown): unknown {
  if (ENTITY_COMMANDS.has(command)) return validateEntityResult(value);
  if (COUNT_COMMANDS.has(command)) return validateCount(value);
  switch(command){case"list_stock_balances":return validateStockBalances(value);case"list_stock_movements":return array(value).map(validateMovement);case"list_active_stock_reservations":return array(value).map(validateReservation);case"reconcile_stock_balances":case"rebuild_stock_balances":return validateReconciliation(value);case"list_purchasing_documents":return array(value).map(validateDocument);case"get_purchasing_document":return validateDocument(value);}
}

export interface Phase06Gateway { call<T>(command:Phase06Command,request?:unknown):Promise<T>;balances(query?:StockQuery):Promise<StockBalanceView[]>;movements(query?:StockQuery):Promise<MovementView[]>; }
export function createPhase06Gateway(invoker:InvokeFunction):Phase06Gateway{const call=async<T,>(command:Phase06Command,request?:unknown):Promise<T>=>{try{const value=request===undefined?await invoker(command):await invoker(command,{request});return validatePhase06Response(command,value) as T;}catch(error){throw normalizePhase06Error(error);}};return{call,balances:async(query={})=>validateStockBalances(await call("list_stock_balances",query)),movements:(query={})=>call<MovementView[]>("list_stock_movements",query)};}

declare global { interface Window { __POSMAN_DEV_PHASE06_INVOKER__?: InvokeFunction } }
export function resolvePhase06Gateway():Phase06Gateway|null{if(import.meta.env.DEV&&typeof window!=="undefined"&&typeof window.__POSMAN_DEV_PHASE06_INVOKER__==="function")return createPhase06Gateway(window.__POSMAN_DEV_PHASE06_INVOKER__);return isTauri()?createPhase06Gateway(invoke):null;}
