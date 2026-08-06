-- POSMAN PHASE 09 - documents, reports, audit presentation, and verified backup/restore.
-- This migration is additive. Accepted migrations 0001-0006 remain immutable.

ALTER TABLE document_templates ADD COLUMN locale TEXT NOT NULL DEFAULT 'ar-DZ'
    CHECK (locale IN ('ar-DZ', 'fr-DZ'));

ALTER TABLE document_template_versions ADD COLUMN locale TEXT NOT NULL DEFAULT 'ar-DZ'
    CHECK (locale IN ('ar-DZ', 'fr-DZ'));
ALTER TABLE document_template_versions ADD COLUMN configuration_json TEXT NOT NULL DEFAULT '{}'
    CHECK (json_valid(configuration_json));
ALTER TABLE document_template_versions ADD COLUMN published_at TEXT;
ALTER TABLE document_template_versions ADD COLUMN published_by TEXT;

CREATE UNIQUE INDEX uq_document_templates_company_type_locale
    ON document_templates(company_id, document_type, locale);
CREATE INDEX idx_template_versions_lookup
    ON document_template_versions(company_id, document_template_id, locale, version_number);

CREATE TABLE document_template_defaults (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    document_type TEXT NOT NULL CHECK (document_type IN (
        'SALES_ORDER', 'DELIVERY_NOTE', 'SALES_INVOICE', 'SALES_CREDIT_NOTE',
        'PURCHASE_ORDER', 'GOODS_RECEIPT', 'SUPPLIER_INVOICE', 'PURCHASE_RETURN',
        'CUSTOMER_RECEIPT', 'SUPPLIER_PAYMENT'
    )),
    locale TEXT NOT NULL CHECK (locale IN ('ar-DZ', 'fr-DZ')),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    configuration_json TEXT NOT NULL CHECK (json_valid(configuration_json)),
    created_at TEXT NOT NULL,
    UNIQUE (document_type, locale)
);

CREATE TABLE document_template_drafts (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_template_id TEXT NOT NULL REFERENCES document_templates(id) ON DELETE RESTRICT,
    base_template_version_id TEXT REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    document_type TEXT NOT NULL CHECK (document_type IN (
        'SALES_ORDER', 'DELIVERY_NOTE', 'SALES_INVOICE', 'SALES_CREDIT_NOTE',
        'PURCHASE_ORDER', 'GOODS_RECEIPT', 'SUPPLIER_INVOICE', 'PURCHASE_RETURN',
        'CUSTOMER_RECEIPT', 'SUPPLIER_PAYMENT'
    )),
    locale TEXT NOT NULL CHECK (locale IN ('ar-DZ', 'fr-DZ')),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    configuration_json TEXT NOT NULL CHECK (json_valid(configuration_json)),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'PUBLISHED', 'ABANDONED')),
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_at TEXT NOT NULL,
    updated_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, id),
    CHECK (base_template_version_id IS NULL OR length(trim(base_template_version_id)) > 0)
);

CREATE UNIQUE INDEX uq_template_draft_active
    ON document_template_drafts(company_id, document_template_id, locale)
    WHERE status = 'DRAFT';
CREATE INDEX idx_template_drafts_lookup
    ON document_template_drafts(company_id, document_type, locale, status, updated_at);

CREATE TABLE document_template_publications (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_template_id TEXT NOT NULL REFERENCES document_templates(id) ON DELETE RESTRICT,
    template_version_id TEXT NOT NULL UNIQUE REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    locale TEXT NOT NULL CHECK (locale IN ('ar-DZ', 'fr-DZ')),
    status TEXT NOT NULL DEFAULT 'PUBLISHED' CHECK (status IN ('PUBLISHED', 'RETIRED')),
    activated_at TEXT NOT NULL,
    activated_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    retired_at TEXT,
    retired_by TEXT REFERENCES users(id) ON DELETE RESTRICT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    CHECK (
        (status = 'PUBLISHED' AND retired_at IS NULL AND retired_by IS NULL)
        OR (status = 'RETIRED' AND retired_at IS NOT NULL AND retired_by IS NOT NULL)
    )
);

CREATE UNIQUE INDEX uq_template_publication_active
    ON document_template_publications(company_id, document_template_id, locale)
    WHERE status = 'PUBLISHED';
CREATE INDEX idx_template_publications_history
    ON document_template_publications(company_id, document_template_id, locale, activated_at DESC);

CREATE TABLE document_render_snapshots (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_type TEXT NOT NULL CHECK (document_type IN (
        'SALES_ORDER', 'DELIVERY_NOTE', 'SALES_INVOICE', 'SALES_CREDIT_NOTE',
        'PURCHASE_ORDER', 'GOODS_RECEIPT', 'SUPPLIER_INVOICE', 'PURCHASE_RETURN',
        'CUSTOMER_RECEIPT', 'SUPPLIER_PAYMENT'
    )),
    source_entity_kind TEXT NOT NULL CHECK (source_entity_kind IN ('COMMERCIAL_DOCUMENT', 'PAYMENT')),
    source_document_id TEXT NOT NULL CHECK (length(trim(source_document_id)) > 0),
    source_document_number TEXT NOT NULL CHECK (length(trim(source_document_number)) > 0),
    source_document_status TEXT NOT NULL CHECK (length(trim(source_document_status)) > 0),
    template_id TEXT NOT NULL REFERENCES document_templates(id) ON DELETE RESTRICT,
    template_version_id TEXT NOT NULL REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    locale TEXT NOT NULL CHECK (locale IN ('ar-DZ', 'fr-DZ')),
    canonical_payload_json TEXT NOT NULL CHECK (json_valid(canonical_payload_json)),
    rendered_html TEXT NOT NULL CHECK (length(trim(rendered_html)) > 0),
    rendered_css TEXT NOT NULL,
    content_sha256 TEXT NOT NULL CHECK (
        length(content_sha256) = 64 AND content_sha256 NOT GLOB '*[^0-9A-Fa-f]*'
    ),
    pdf_relative_path TEXT NOT NULL CHECK (
        length(trim(pdf_relative_path)) > 0
        AND substr(pdf_relative_path, 1, 1) <> '/'
        AND instr(pdf_relative_path, '..') = 0
        AND instr(pdf_relative_path, char(92)) = 0
    ),
    pdf_sha256 TEXT NOT NULL CHECK (
        length(pdf_sha256) = 64 AND pdf_sha256 NOT GLOB '*[^0-9A-Fa-f]*'
    ),
    pdf_size_bytes INTEGER NOT NULL CHECK (pdf_size_bytes > 0),
    rendered_at TEXT NOT NULL,
    rendered_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    UNIQUE (company_id, id),
    UNIQUE (company_id, pdf_relative_path)
);

CREATE INDEX idx_render_snapshots_source
    ON document_render_snapshots(company_id, source_entity_kind, source_document_id, rendered_at DESC);
CREATE INDEX idx_render_snapshots_number
    ON document_render_snapshots(company_id, document_type, source_document_number, rendered_at DESC);

CREATE TABLE backup_policies (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL UNIQUE REFERENCES companies(id) ON DELETE RESTRICT,
    automatic_enabled INTEGER NOT NULL DEFAULT 1 CHECK (automatic_enabled IN (0, 1)),
    weekly_enabled INTEGER NOT NULL DEFAULT 1 CHECK (weekly_enabled IN (0, 1)),
    timezone_name TEXT NOT NULL DEFAULT 'Africa/Algiers' CHECK (length(trim(timezone_name)) > 0),
    last_attempt_local_date TEXT CHECK (last_attempt_local_date IS NULL OR length(last_attempt_local_date) = 10),
    last_success_local_date TEXT CHECK (last_success_local_date IS NULL OR length(last_success_local_date) = 10),
    last_warning_code TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE TABLE verified_backups (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    backup_kind TEXT NOT NULL CHECK (backup_kind IN (
        'MANUAL', 'AUTOMATIC_DAILY', 'AUTOMATIC_WEEKLY', 'PRE_RESTORE'
    )),
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    application_version TEXT NOT NULL CHECK (length(trim(application_version)) > 0),
    schema_version TEXT NOT NULL CHECK (length(trim(schema_version)) > 0),
    migration_ledger_digest TEXT NOT NULL CHECK (
        length(migration_ledger_digest) = 64 AND migration_ledger_digest NOT GLOB '*[^0-9A-Fa-f]*'
    ),
    database_size_bytes INTEGER NOT NULL CHECK (database_size_bytes > 0),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9A-Fa-f]*'),
    relative_path TEXT NOT NULL CHECK (
        length(trim(relative_path)) > 0
        AND substr(relative_path, 1, 1) <> '/'
        AND instr(relative_path, '..') = 0
        AND instr(relative_path, char(92)) = 0
    ),
    integrity_status TEXT NOT NULL CHECK (integrity_status IN ('PENDING', 'OK', 'FAILED')),
    foreign_key_status TEXT NOT NULL CHECK (foreign_key_status IN ('PENDING', 'OK', 'FAILED')),
    verification_status TEXT NOT NULL CHECK (verification_status IN ('PENDING', 'VERIFIED', 'FAILED')),
    failure_reason TEXT,
    selected_for_restore INTEGER NOT NULL DEFAULT 0 CHECK (selected_for_restore IN (0, 1)),
    imported_at TEXT,
    imported_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    deletion_failure TEXT,
    UNIQUE (company_id, id),
    UNIQUE (company_id, relative_path),
    CHECK (
        (verification_status = 'VERIFIED' AND integrity_status = 'OK' AND foreign_key_status = 'OK' AND failure_reason IS NULL)
        OR verification_status <> 'VERIFIED'
    )
);

CREATE INDEX idx_verified_backups_retention
    ON verified_backups(company_id, backup_kind, verification_status, created_at DESC);
CREATE INDEX idx_audit_logs_workspace
    ON audit_logs(company_id, occurred_at DESC, actor_user_id, action_code, entity_type, outcome);

CREATE TRIGGER trg_template_drafts_published_no_update
BEFORE UPDATE ON document_template_drafts
WHEN OLD.status <> 'DRAFT'
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_DRAFT_IMMUTABLE');
END;

CREATE TRIGGER trg_template_drafts_published_no_delete
BEFORE DELETE ON document_template_drafts
WHEN OLD.status <> 'DRAFT'
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_DRAFT_IMMUTABLE');
END;

CREATE TRIGGER trg_template_publications_identity_immutable
BEFORE UPDATE ON document_template_publications
WHEN NEW.id <> OLD.id
  OR NEW.company_id <> OLD.company_id
  OR NEW.document_template_id <> OLD.document_template_id
  OR NEW.template_version_id <> OLD.template_version_id
  OR NEW.locale <> OLD.locale
  OR NEW.activated_at <> OLD.activated_at
  OR NEW.activated_by <> OLD.activated_by
  OR OLD.status = 'RETIRED'
  OR (OLD.status = 'PUBLISHED' AND NEW.status NOT IN ('PUBLISHED', 'RETIRED'))
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_VERSION_IMMUTABLE');
END;

CREATE TRIGGER trg_template_publications_no_delete
BEFORE DELETE ON document_template_publications
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_VERSION_IMMUTABLE');
END;

CREATE TRIGGER trg_render_snapshots_no_update
BEFORE UPDATE ON document_render_snapshots
BEGIN
    SELECT RAISE(ABORT, 'RENDERED_DOCUMENT_IMMUTABLE');
END;

CREATE TRIGGER trg_render_snapshots_no_delete
BEFORE DELETE ON document_render_snapshots
BEGIN
    SELECT RAISE(ABORT, 'RENDERED_DOCUMENT_IMMUTABLE');
END;

-- PHASE 09 action-level permissions. Descriptions are intentionally bilingual and safe.
INSERT OR IGNORE INTO permissions(id, code, domain, description_ar, description_fr, is_sensitive, created_at) VALUES
('perm-p09-doc-tpl-view', 'documents.templates.view', 'documents', 'عرض قوالب المستندات', 'Consulter les modèles de documents', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-doc-tpl-manage', 'documents.templates.manage', 'documents', 'إدارة ونشر قوالب المستندات', 'Gérer et publier les modèles de documents', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-doc-render', 'documents.render', 'documents', 'إنشاء نسخة تاريخية للمستند', 'Créer un rendu historique du document', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-doc-print', 'documents.print', 'documents', 'طباعة المستندات', 'Imprimer les documents', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-doc-export', 'documents.export', 'documents', 'تصدير المستندات', 'Exporter les documents', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-reports-view', 'reports.view', 'reports', 'عرض التقارير التشغيلية', 'Consulter les rapports opérationnels', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-reports-export', 'reports.export', 'reports', 'تصدير التقارير التشغيلية', 'Exporter les rapports opérationnels', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-audit-view', 'audit.view', 'audit', 'عرض سجل العمليات', 'Consulter le journal d’audit', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-audit-export', 'audit.export', 'audit', 'تصدير سجل العمليات المنقح', 'Exporter le journal d’audit expurgé', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-backup-view', 'backup.view', 'backup', 'عرض النسخ الاحتياطية', 'Consulter les sauvegardes', 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('perm-p09-backup-manage', 'backup.manage', 'backup', 'إدارة سياسة واحتفاظ النسخ الاحتياطية', 'Gérer la politique et la rétention des sauvegardes', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT OR IGNORE INTO role_permissions(id, company_id, role_id, permission_id, granted_at, granted_by)
SELECT 'rp-p09-' || r.id || '-' || p.id, r.company_id, r.id, p.id,
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), NULL
FROM roles r CROSS JOIN permissions p
WHERE r.code IN ('OWNER', 'SYSTEM_ADMINISTRATOR')
  AND r.is_active = 1
  AND p.code IN (
      'documents.templates.view', 'documents.templates.manage', 'documents.render',
      'documents.print', 'documents.export', 'reports.view', 'reports.export',
      'audit.view', 'audit.export', 'backup.view', 'backup.create',
      'backup.restore', 'backup.manage'
  );

INSERT OR IGNORE INTO role_permissions(id, company_id, role_id, permission_id, granted_at, granted_by)
SELECT 'rp-p09-' || r.id || '-' || p.id, r.company_id, r.id, p.id,
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), NULL
FROM roles r CROSS JOIN permissions p
WHERE r.code IN ('SALES', 'PURCHASING')
  AND r.is_active = 1
  AND p.code IN ('documents.templates.view', 'documents.render', 'documents.print', 'documents.export', 'reports.view', 'reports.export');

INSERT OR IGNORE INTO role_permissions(id, company_id, role_id, permission_id, granted_at, granted_by)
SELECT 'rp-p09-' || r.id || '-' || p.id, r.company_id, r.id, p.id,
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), NULL
FROM roles r CROSS JOIN permissions p
WHERE r.code IN ('ACCOUNTANT', 'AUDITOR')
  AND r.is_active = 1
  AND p.code IN ('documents.templates.view', 'reports.view', 'reports.export', 'audit.view');

-- Safe structured defaults. Rust owns the HTML/CSS renderer; these rows contain no executable markup.
INSERT OR IGNORE INTO document_template_defaults(id, document_type, locale, display_name, configuration_json, created_at)
WITH document_types(document_type, ar_name, fr_name) AS (
    VALUES
    ('SALES_ORDER', 'طلب بيع', 'Bon de commande client'),
    ('DELIVERY_NOTE', 'سند تسليم', 'Bon de livraison'),
    ('SALES_INVOICE', 'فاتورة بيع', 'Facture de vente'),
    ('SALES_CREDIT_NOTE', 'إشعار دائن للعميل', 'Avoir client'),
    ('PURCHASE_ORDER', 'أمر شراء', 'Bon de commande fournisseur'),
    ('GOODS_RECEIPT', 'سند استلام', 'Bon de réception'),
    ('SUPPLIER_INVOICE', 'فاتورة مورد', 'Facture fournisseur'),
    ('PURCHASE_RETURN', 'مرتجع شراء', 'Retour fournisseur'),
    ('CUSTOMER_RECEIPT', 'وصل قبض', 'Reçu client'),
    ('SUPPLIER_PAYMENT', 'سند دفع مورد', 'Paiement fournisseur')
), locales(locale, suffix) AS (VALUES ('ar-DZ', 'ar'), ('fr-DZ', 'fr'))
SELECT 'tpl-default-' || lower(document_type) || '-' || suffix,
       document_type,
       locale,
       CASE WHEN locale = 'ar-DZ' THEN ar_name ELSE fr_name END,
       json_object(
           'documentTitleAr', ar_name,
           'documentTitleFr', fr_name,
           'showLogo', 1,
           'showCompanyIdentity', 1,
           'showTradeRegister', 1,
           'showTaxIdentifier', 1,
           'showPartnerAddress', 1,
           'showPaymentInformation', 1,
           'footerTextAr', 'شكراً لتعاملكم معنا',
           'footerTextFr', 'Merci pour votre confiance',
           'spacing', 'NORMAL',
           'orientation', 'PORTRAIT',
           'enabledSections', json_array('REFERENCES', 'NOTES', 'TOTALS')
       ),
       '2026-08-06T00:00:00Z'
FROM document_types CROSS JOIN locales;
