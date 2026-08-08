import { invoke } from "@tauri-apps/api/core";

export type Phase09Locale = "ar-DZ" | "fr-DZ";
export type TemplateState = "DRAFT" | "PUBLISHED" | "RETIRED";
export type SortDirection = "ASC" | "DESC";
export type BackupKind = "MANUAL" | "AUTOMATIC_DAILY" | "AUTOMATIC_WEEKLY" | "PRE_RESTORE";

export interface SafeError {
  code: string;
  message: string;
  retryable: boolean;
}

export class Phase09GatewayError extends Error implements SafeError {
  readonly code: string;
  readonly retryable: boolean;

  constructor(error: SafeError) {
    super(error.message);
    this.name = "Phase09GatewayError";
    this.code = error.code;
    this.retryable = error.retryable;
  }
}

export interface Page<T> {
  items: T[];
  page: number;
  pageSize: number;
  total: number;
}

export interface TemplateConfiguration {
  documentTitleAr: string;
  documentTitleFr: string;
  showLogo: boolean;
  showCompanyIdentity: boolean;
  showTradeRegister: boolean;
  showTaxIdentifier: boolean;
  showPartnerAddress: boolean;
  showPaymentInformation: boolean;
  footerTextAr: string;
  footerTextFr: string;
  spacing: "NORMAL" | "COMPACT";
  orientation: "PORTRAIT" | "LANDSCAPE";
  enabledSections: string[];
}

export interface TemplateSummary {
  templateId: string;
  documentType: string;
  locale: Phase09Locale;
  displayName: string;
  activeVersionId: string | null;
  activeVersionNumber: number | null;
  activeContentSha256: string | null;
  draftId: string | null;
  draftRowVersion: number | null;
  state: TemplateState;
}

export interface TemplateDraftView {
  draftId: string;
  templateId: string;
  documentType: string;
  locale: Phase09Locale;
  displayName: string;
  configuration: TemplateConfiguration;
  baseTemplateVersionId: string | null;
  rowVersion: number;
  updatedAt: string;
}

export interface TemplateVersionView {
  versionId: string;
  versionNumber: number;
  locale: Phase09Locale;
  contentSha256: string;
  status: "PUBLISHED" | "RETIRED";
  publishedAt: string;
  publishedBy: string;
  rowVersion: number;
}

export interface TemplateDetail {
  summary: TemplateSummary;
  draft: TemplateDraftView | null;
  versions: TemplateVersionView[];
}

export interface TemplateKeyRequest {
  documentType: string;
  locale: Phase09Locale;
}

export interface CreateTemplateDraftRequest extends TemplateKeyRequest {
  displayName?: string | null;
}

export interface UpdateTemplateDraftRequest {
  draftId: string;
  displayName: string;
  configuration: TemplateConfiguration;
  expectedRowVersion: number;
}

export interface PublishTemplateRequest {
  draftId: string;
  expectedRowVersion: number;
  confirmed: boolean;
}

export interface RetireTemplateRequest {
  templateVersionId: string;
  expectedRowVersion: number;
  confirmed: boolean;
}

export interface DocumentRequest {
  documentType: string;
  sourceDocumentId: string;
  locale: Phase09Locale;
}

export interface PreviewResult {
  previewId: string;
  documentType: string;
  sourceDocumentId: string;
  locale: Phase09Locale;
  integrityState: string;
}

export interface PreviewContent {
  previewId: string;
  locale: Phase09Locale;
  direction: "rtl" | "ltr";
  html: string;
  css: string;
  contentSha256: string;
  integrityState: string;
}

export interface RenderedDocumentView {
  renderId: string;
  documentType: string;
  sourceDocumentId: string;
  sourceDocumentNumber: string;
  sourceDocumentStatus: string;
  templateId: string;
  templateVersionId: string;
  locale: Phase09Locale;
  contentSha256: string;
  pdfRelativePath: string;
  pdfSha256: string;
  pdfSizeBytes: number;
  renderedAt: string;
  renderedBy: string;
  integrityState: string;
}

export interface RenderedDocumentsRequest {
  documentType?: string | null;
  sourceDocumentId?: string | null;
  page: number;
  pageSize: number;
}

export interface RenderedDocumentKeyRequest {
  renderId: string;
}

export interface ExportResult {
  relativePath: string;
  sha256: string;
  sizeBytes: number;
}

export type ReportId =
  | "SALES_SUMMARY"
  | "SALES_BY_PRODUCT"
  | "SALES_BY_CUSTOMER"
  | "PURCHASES_SUMMARY"
  | "PURCHASES_BY_SUPPLIER"
  | "STOCK_ON_HAND"
  | "STOCK_VALUATION"
  | "STOCK_MOVEMENTS"
  | "LOW_STOCK"
  | "OPEN_RECEIVABLES"
  | "OPEN_PAYABLES"
  | "CASH_BANK_REGISTER"
  | "TRIAL_BALANCE";

export interface ReportDescriptor {
  reportId: ReportId;
  nameAr: string;
  nameFr: string;
  supportsDateRange: boolean;
  supportsWarehouse: boolean;
  supportsPartner: boolean;
  supportsProduct: boolean;
  supportsStatus: boolean;
}

export interface ReportRequest {
  reportId: ReportId;
  startDate?: string | null;
  endDate?: string | null;
  warehouseId?: string | null;
  partnerId?: string | null;
  productId?: string | null;
  status?: string | null;
  sortField?: string | null;
  sortDirection?: SortDirection | null;
  page: number;
  pageSize: number;
  locale: Phase09Locale;
}

export interface ReportColumn {
  key: string;
  labelAr: string;
  labelFr: string;
  kind: string;
}

export type ReportValue = string | number | boolean | null;

export interface ReportRow {
  values: Record<string, ReportValue>;
}

export interface ReportPage {
  reportId: ReportId;
  columns: ReportColumn[];
  rows: ReportRow[];
  page: number;
  pageSize: number;
  totalRows: number;
  generatedAt: string;
}

export interface AuditRequest {
  startAt?: string | null;
  endAt?: string | null;
  userId?: string | null;
  domain?: string | null;
  action?: string | null;
  entityType?: string | null;
  entityId?: string | null;
  outcome?: "SUCCESS" | "FAILURE" | "DENIED" | null;
  sensitiveOnly?: boolean | null;
  page: number;
  pageSize: number;
}

export interface AuditEventView {
  id: string;
  actorUserId: string | null;
  actorDisplayName: string | null;
  domain: string;
  actionCode: string;
  entityType: string;
  entityId: string;
  occurredAt: string;
  outcome: string;
  sensitive: boolean;
  details: unknown;
}

export interface BackupSettingsView {
  automaticEnabled: boolean;
  weeklyEnabled: boolean;
  timezoneName: string;
  lastAttemptLocalDate: string | null;
  lastSuccessLocalDate: string | null;
  lastWarningCode: string | null;
  rowVersion: number;
  encryptionStatus: "LOCAL_UNENCRYPTED";
}

export interface UpdateBackupSettingsRequest {
  automaticEnabled: boolean;
  weeklyEnabled: boolean;
  expectedRowVersion: number;
}

export interface CreateBackupRequest {
  backupKind: Exclude<BackupKind, "PRE_RESTORE">;
}

export interface BackupListRequest {
  backupKind?: BackupKind | null;
  page: number;
  pageSize: number;
}

export interface BackupKeyRequest {
  backupId: string;
}

export interface BackupView {
  backupId: string;
  backupKind: BackupKind;
  createdAt: string;
  createdBy: string | null;
  applicationVersion: string;
  schemaVersion: string;
  migrationLedgerDigest: string;
  databaseSizeBytes: number;
  sha256: string;
  relativePath: string;
  integrityStatus: string;
  foreignKeyStatus: string;
  verificationStatus: string;
  failureReason: string | null;
  selectedForRestore: boolean;
}

export interface RestoreBackupRequest {
  backupId: string;
  currentPassword: string;
  confirmationText: "RESTORE";
  confirmed: true;
}

type JsonRecord = Record<string, unknown>;

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isSafeError(value: unknown): value is SafeError {
  return (
    isRecord(value) &&
    typeof value.code === "string" &&
    typeof value.message === "string" &&
    typeof value.retryable === "boolean"
  );
}

export function normalizePhase09Error(error: unknown): Phase09GatewayError {
  if (error instanceof Phase09GatewayError) {
    return error;
  }
  if (isSafeError(error)) {
    return new Phase09GatewayError(error);
  }
  if (isRecord(error) && isSafeError(error.error)) {
    return new Phase09GatewayError(error.error);
  }
  return new Phase09GatewayError({
    code: "INTERNAL_ERROR",
    message: "POSMAN could not complete the local operation.",
    retryable: true,
  });
}

export async function invokePhase09<T>(
  command: string,
  payload?: JsonRecord,
): Promise<T> {
  try {
    return await invoke<T>(command, payload);
  } catch (error: unknown) {
    throw normalizePhase09Error(error);
  }
}

export class RequestGate {
  private generation = 0;

  begin(): number {
    this.generation += 1;
    return this.generation;
  }

  isCurrent(generation: number): boolean {
    return generation === this.generation;
  }

  invalidate(): void {
    this.generation += 1;
  }
}

export function requireString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Phase09GatewayError({
      code: "INVALID_RESPONSE",
      message: `POSMAN returned an invalid ${field}.`,
      retryable: true,
    });
  }
  return value;
}

export function requireArray<T>(value: unknown, field: string): T[] {
  if (!Array.isArray(value)) {
    throw new Phase09GatewayError({
      code: "INVALID_RESPONSE",
      message: `POSMAN returned an invalid ${field}.`,
      retryable: true,
    });
  }
  return value as T[];
}

export function requireObject<T extends JsonRecord>(
  value: unknown,
  field: string,
): T {
  if (!isRecord(value)) {
    throw new Phase09GatewayError({
      code: "INVALID_RESPONSE",
      message: `POSMAN returned an invalid ${field}.`,
      retryable: true,
    });
  }
  return value as T;
}

export function createRequestGate(): RequestGate {
  return new RequestGate();
}

export function requireSafeInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    throw new Phase09GatewayError({
      code: "INVALID_RESPONSE",
      message: `POSMAN returned an invalid ${field}.`,
      retryable: true,
    });
  }
  return value;
}

export const templatesGateway = {
  async list(): Promise<TemplateSummary[]> {
    const result = await invokePhase09<unknown>("phase09_list_templates");
    return requireArray<TemplateSummary>(result, "template list");
  },

  async get(request: TemplateKeyRequest): Promise<TemplateDetail> {
    const result = await invokePhase09<unknown>("phase09_get_template", {
      request,
    });
    return requireObject<TemplateDetail & Record<string, unknown>>(
      result,
      "template detail",
    );
  },

  async createDraft(
    request: CreateTemplateDraftRequest,
  ): Promise<TemplateDraftView> {
    const result = await invokePhase09<unknown>(
      "phase09_create_template_draft",
      { request },
    );
    return requireObject<TemplateDraftView & Record<string, unknown>>(
      result,
      "template draft",
    );
  },

  async updateDraft(
    request: UpdateTemplateDraftRequest,
  ): Promise<TemplateDraftView> {
    const result = await invokePhase09<unknown>(
      "phase09_update_template_draft",
      { request },
    );
    return requireObject<TemplateDraftView & Record<string, unknown>>(
      result,
      "updated template draft",
    );
  },

  async publish(
    request: PublishTemplateRequest,
  ): Promise<TemplateVersionView> {
    const result = await invokePhase09<unknown>("phase09_publish_template", {
      request,
    });
    return requireObject<TemplateVersionView & Record<string, unknown>>(
      result,
      "published template version",
    );
  },

  async retire(
    request: RetireTemplateRequest,
  ): Promise<TemplateVersionView> {
    const result = await invokePhase09<unknown>("phase09_retire_template", {
      request,
    });
    return requireObject<TemplateVersionView & Record<string, unknown>>(
      result,
      "retired template version",
    );
  },
};

export const documentsGateway = {
  async preview(request: DocumentRequest): Promise<PreviewResult> {
    const result = await invokePhase09<unknown>("phase09_preview_document", {
      request,
    });
    return requireObject<PreviewResult & Record<string, unknown>>(
      result,
      "document preview",
    );
  },

  async getPreviewContent(previewId: string): Promise<PreviewContent> {
    const result = await invokePhase09<unknown>(
      "phase09_get_preview_content",
      { request: previewId },
    );
    return requireObject<PreviewContent & Record<string, unknown>>(
      result,
      "preview content",
    );
  },

  async render(request: DocumentRequest): Promise<RenderedDocumentView> {
    const result = await invokePhase09<unknown>("phase09_render_document", {
      request,
    });
    return requireObject<RenderedDocumentView & Record<string, unknown>>(
      result,
      "rendered document",
    );
  },

  async list(
    request: RenderedDocumentsRequest,
  ): Promise<Page<RenderedDocumentView>> {
    const result = await invokePhase09<unknown>(
      "phase09_list_rendered_documents",
      { request },
    );
    return requireObject<Page<RenderedDocumentView> & Record<string, unknown>>(
      result,
      "rendered document page",
    );
  },

  async get(
    request: RenderedDocumentKeyRequest,
  ): Promise<RenderedDocumentView> {
    const result = await invokePhase09<unknown>(
      "phase09_get_rendered_document",
      { request },
    );
    return requireObject<RenderedDocumentView & Record<string, unknown>>(
      result,
      "rendered document",
    );
  },

  async verify(
    request: RenderedDocumentKeyRequest,
  ): Promise<RenderedDocumentView> {
    const result = await invokePhase09<unknown>(
      "phase09_verify_rendered_document",
      { request },
    );
    return requireObject<RenderedDocumentView & Record<string, unknown>>(
      result,
      "verified rendered document",
    );
  },

  async exportPdf(request: RenderedDocumentKeyRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>(
      "phase09_export_rendered_pdf",
      { request },
    );
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "document export",
    );
  },

  async print(request: RenderedDocumentKeyRequest): Promise<void> {
    await invokePhase09<void>("phase09_print_rendered_document", { request });
  },
};

export const reportsGateway = {
  async list(): Promise<ReportDescriptor[]> {
    const result = await invokePhase09<unknown>("phase09_list_reports");
    return requireArray<ReportDescriptor>(result, "report list");
  },

  async run(request: ReportRequest): Promise<ReportPage> {
    const result = await invokePhase09<unknown>("phase09_run_report", {
      request,
    });
    return requireObject<ReportPage & Record<string, unknown>>(
      result,
      "report page",
    );
  },

  async exportCsv(request: ReportRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>(
      "phase09_export_report_csv",
      { request },
    );
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "report CSV export",
    );
  },

  async exportPdf(request: ReportRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>(
      "phase09_export_report_pdf",
      { request },
    );
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "report PDF export",
    );
  },
};

export const auditGateway = {
  async list(request: AuditRequest): Promise<Page<AuditEventView>> {
    const result = await invokePhase09<unknown>("phase09_list_audit_events", {
      request,
    });
    return requireObject<Page<AuditEventView> & Record<string, unknown>>(
      result,
      "audit page",
    );
  },

  async exportCsv(request: AuditRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>("phase09_export_audit_csv", {
      request,
    });
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "audit export",
    );
  },
};

export const backupGateway = {
  async getSettings(): Promise<BackupSettingsView> {
    const result = await invokePhase09<unknown>("phase09_get_backup_settings");
    return requireObject<BackupSettingsView & Record<string, unknown>>(
      result,
      "backup settings",
    );
  },

  async updateSettings(
    request: UpdateBackupSettingsRequest,
  ): Promise<BackupSettingsView> {
    const result = await invokePhase09<unknown>(
      "phase09_update_backup_settings",
      { request },
    );
    return requireObject<BackupSettingsView & Record<string, unknown>>(
      result,
      "updated backup settings",
    );
  },

  async create(request: CreateBackupRequest): Promise<BackupView> {
    const result = await invokePhase09<unknown>("phase09_create_backup", {
      request,
    });
    return requireObject<BackupView & Record<string, unknown>>(
      result,
      "created backup",
    );
  },

  async list(request: BackupListRequest): Promise<Page<BackupView>> {
    const result = await invokePhase09<unknown>("phase09_list_backups", {
      request,
    });
    return requireObject<Page<BackupView> & Record<string, unknown>>(
      result,
      "backup page",
    );
  },

  async verify(request: BackupKeyRequest): Promise<BackupView> {
    const result = await invokePhase09<unknown>("phase09_verify_backup", {
      request,
    });
    return requireObject<BackupView & Record<string, unknown>>(
      result,
      "verified backup",
    );
  },

  async export(request: BackupKeyRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>("phase09_export_backup", {
      request,
    });
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "backup export",
    );
  },

  async import(request?: { sourcePath?: string }): Promise<BackupView> {
    const result = await invokePhase09<unknown>("phase09_import_backup", {
      request,
    });
    return requireObject<BackupView & Record<string, unknown>>(
      result,
      "imported backup",
    );
  },

  async restore(request: RestoreBackupRequest): Promise<void> {
    if (
      !request ||
      request.confirmationText !== "RESTORE" ||
      !request.currentPassword ||
      request.currentPassword.length === 0 ||
      request.confirmed !== true
    ) {
      throw new Phase09GatewayError({
        code: "INVALID_RESTORE_REQUEST",
        message: "Restore requires password and exact RESTORE confirmation.",
        retryable: false,
      });
    }
    await invokePhase09<void>("phase09_restore_backup", { request });
  },

  async delete(request: BackupKeyRequest): Promise<void> {
    await invokePhase09<void>("phase09_delete_backup", { request });
  },
};
