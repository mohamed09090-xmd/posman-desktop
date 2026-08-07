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
