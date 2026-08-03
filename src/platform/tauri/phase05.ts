import { invoke, isTauri } from "@tauri-apps/api/core";
import type { InvokeFunction } from "./runtime-status";

export type BelowCostPolicy = "BLOCK" | "ADMIN_OVERRIDE" | "WARNING_ONLY";
export type LocaleCode = "ar-DZ" | "fr-DZ";

export interface Phase05ErrorPayload { code: string; message?: string }
export class Phase05GatewayError extends Error {
  readonly code: string;
  constructor(code: string) {
    super("The local operation could not be completed.");
    this.name = "Phase05GatewayError";
    this.code = code;
  }
}

export interface SetupStatus {
  setupRequired: boolean;
  hasDraft: boolean;
  schemaVersion: string;
  defaultFiscalStartsOn: string;
  defaultFiscalEndsOn: string;
}
export interface SetupDraft { draftSchemaVersion: number; data: Record<string, unknown>; rowVersion: number }
export interface CompleteSetupResult { companyId: string; administratorUserId: string; recoveryCode?: string; alreadyCompleted: boolean }
export interface SessionView {
  companyId: string; userId: string; username: string; displayName: string;
  preferredLanguage: string; permissions: string[]; locked: boolean;
}
export interface CompanyProfile {
  id: string; code: string; legalName: string; nameAr: string; nameFr?: string;
  activityDescription?: string; legalForm?: string; tradeRegisterNumber?: string;
  taxIdentifier?: string; statisticalIdentifier?: string; taxArticleNumber?: string;
  bankRib?: string; socialCapitalMinor?: number; addressText?: string; wilayaCode?: string;
  city?: string; postalCode?: string; phone?: string; email?: string;
  defaultMarginRateScaled: number; belowCostPolicy: BelowCostPolicy;
  sessionIdleTimeoutMinutes: number; rowVersion: number;
}
export interface FiscalPeriodView { periodNumber: number; name: string; startsOn: string; endsOn: string; status: string }
export interface FiscalSetup { fiscalYearId: string; code: string; startsOn: string; endsOn: string; periods: FiscalPeriodView[]; rowVersion: number; inUse: boolean }
export interface DocumentSequenceView { id: string; documentType: string; prefix: string; nextNumber: number; paddingWidth: number; preview: string; rowVersion: number }
export interface ReferenceRecord { id: string; code: string; nameAr: string; nameFr?: string; isActive: boolean; rowVersion: number; details: Record<string, unknown> }
export interface ProductView { id: string; code: string; nameAr: string; nameFr?: string; unitId: string; productFamilyId?: string; taxRateId?: string; purchasePriceScaled: number; salePriceScaled: number; suggestedSalePriceScaled: number; pricingWarning?: "BELOW_COST" | "ZERO_MARGIN"; belowCostPolicy: BelowCostPolicy; isActive: boolean; rowVersion: number }
export interface PartnerView { id: string; code: string; legalName: string; displayNameAr: string; displayNameFr?: string; isCustomer: boolean; isSupplier: boolean; isActive: boolean; rowVersion: number }
export interface UserView { id: string; username: string; displayName: string; preferredLanguage: string; isActive: boolean; roleIds: string[]; rowVersion: number }
export interface RoleView { id: string; code: string; nameAr: string; nameFr?: string; isSystem: boolean; isActive: boolean; permissionCodes: string[]; rowVersion: number }
export interface Page<T> { items: T[]; page: number; pageSize: number; total: number }
export interface PageRequest { search?: string; page?: number; pageSize?: number; includeInactive?: boolean }

export const PHASE05_COMMANDS = [
  "get_setup_status", "load_setup_draft", "save_setup_draft", "discard_setup_draft", "complete_initial_setup",
  "login", "recover_admin_password", "get_current_session", "logout", "lock_session", "unlock_session",
  "change_own_password", "rotate_recovery_code", "get_company_profile", "update_company_profile",
  "get_fiscal_setup", "update_fiscal_setup", "list_document_sequences", "update_document_sequence",
  "list_users", "create_user", "update_user", "set_user_roles", "reset_user_password",
  "list_roles", "create_role", "update_role", "set_role_permissions",
  "list_products", "create_product", "update_product", "set_product_active", "set_product_price",
  "list_partners", "create_partner", "update_partner", "set_partner_active", "create_partner_address", "create_partner_contact",
  "list_units", "create_unit", "update_unit", "set_unit_active",
  "list_tax_rates", "create_tax_rate", "update_tax_rate", "set_tax_rate_active",
  "list_warehouses", "create_warehouse", "update_warehouse", "set_warehouse_active",
  "list_warehouse_locations", "create_warehouse_location", "update_warehouse_location", "set_warehouse_location_active",
  "list_product_families", "create_product_family", "update_product_family", "set_product_family_active",
] as const;
export type Phase05Command = typeof PHASE05_COMMANDS[number];

const SAFE_CODES = new Set([
  "VALIDATION_FAILED", "OPERATION_FAILED", "AUTHENTICATION_REQUIRED", "AUTHENTICATION_FAILED",
  "PERMISSION_DENIED", "SESSION_LOCKED", "CONCURRENCY_CONFLICT", "ACCOUNT_LOCKED",
  "SETUP_ALREADY_COMPLETED", "BELOW_COST_BLOCKED", "BELOW_COST_OVERRIDE_REQUIRED",
]);
function normalize(error: unknown): Phase05GatewayError {
  if (error instanceof Phase05GatewayError) return error;
  if (typeof error === "object" && error !== null && "code" in error) {
    const code = String((error as Phase05ErrorPayload).code);
    if (SAFE_CODES.has(code)) return new Phase05GatewayError(code);
  }
  return new Phase05GatewayError("OPERATION_FAILED");
}

export interface Phase05Gateway {
  call<T>(command: Phase05Command, request?: unknown): Promise<T>;
  getSetupStatus(): Promise<SetupStatus>;
  getCurrentSession(): Promise<SessionView>;
}
export function createPhase05Gateway(invoker: InvokeFunction): Phase05Gateway {
  const call = async <T,>(command: Phase05Command, request?: unknown): Promise<T> => {
    try {
      return request === undefined
        ? await invoker(command) as T
        : await invoker(command, { request }) as T;
    } catch (error) { throw normalize(error); }
  };
  return {
    call,
    getSetupStatus: () => call<SetupStatus>("get_setup_status"),
    getCurrentSession: () => call<SessionView>("get_current_session"),
  };
}

declare global { interface Window { __POSMAN_DEV_PHASE05_INVOKER__?: InvokeFunction } }
export function resolvePhase05Gateway(): Phase05Gateway | null {
  if (import.meta.env.DEV && typeof window !== "undefined" && typeof window.__POSMAN_DEV_PHASE05_INVOKER__ === "function") {
    return createPhase05Gateway(window.__POSMAN_DEV_PHASE05_INVOKER__);
  }
  return isTauri() ? createPhase05Gateway(invoke) : null;
}
