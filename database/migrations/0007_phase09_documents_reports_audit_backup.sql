-- PHASE 09: versioned structured templates, immutable document snapshots,
-- operational reporting support, audit presentation indexes, and verified backup/restore metadata.

CREATE TABLE phase09_template_drafts (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_template_id TEXT NOT NULL REFERENCES document_templates(id) ON DELETE RESTRICT,
    document_type TEXT NOT NULL CHECK (document_type IN (
        'SALES_ORDER', 'DELIVERY_NOTE', 'SALES_INVOICE', 'SALES_CREDIT_NOTE',
        'PURCHASE_ORDER', 'GOODS_RECEIPT', 'SUPPLIER_INVOICE', 'PURCHASE_RETURN',
        'CUSTOMER_RECEIPT', 'SUPPLIER_PAYMENT'
    )),
    locale TEXT NOT NULL CHECK (locale IN ('ar', 'fr')),
    version_number INTEGER NOT NULL CHECK (version_number >= 1),
    base_template_version_id TEXT REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    state TEXT NOT NULL DEFAULT 'DRAFT' CHECK (state IN ('DRAFT', 'PUBLISHED', 'RETIRED')),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    title_ar TEXT NOT NULL CHECK (length(trim(title_ar)) > 0),
    title_fr TEXT NOT NULL CHECK (length(trim(title_fr)) > 0),
    show_logo INTEGER NOT NULL DEFAULT 1 CHECK (show_logo IN (0, 1)),
    show_company_identity INTEGER NOT NULL DEFAULT 1 CHECK (show_company_identity IN (0, 1)),
    show_trade_register INTEGER NOT NULL DEFAULT 1 CHECK (show_trade_register IN (0, 1)),
    show_tax_identifier INTEGER NOT NULL DEFAULT 1 CHECK (show_tax_identifier IN (0, 1)),
    show_partner_address INTEGER NOT NULL DEFAULT 1 CHECK (show_partner_address IN (0, 1)),
    show_payment_information INTEGER NOT NULL DEFAULT 1 CHECK (show_payment_information IN (0, 1)),
    footer_ar TEXT NOT NULL DEFAULT '',
    footer_fr TEXT NOT NULL DEFAULT '',
    spacing TEXT NOT NULL DEFAULT 'NORMAL' CHECK (spacing IN ('NORMAL', 'COMPACT')),
    orientation TEXT NOT NULL DEFAULT 'PORTRAIT' CHECK (orientation IN ('PORTRAIT', 'LANDSCAPE')),
    optional_sections_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(optional_sections_json)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, document_template_id, locale, version_number)
);

CREATE TABLE phase09_template_version_configs (
    template_version_id TEXT NOT NULL PRIMARY KEY REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_template_id TEXT NOT NULL REFERENCES document_templates(id) ON DELETE RESTRICT,
    source_draft_id TEXT NOT NULL REFERENCES phase09_template_drafts(id) ON DELETE RESTRICT,
    document_type TEXT NOT NULL CHECK (length(trim(document_type)) > 0),
    locale TEXT NOT NULL CHECK (locale IN ('ar', 'fr')),
    config_json TEXT NOT NULL CHECK (json_valid(config_json)),
    published_at TEXT NOT NULL,
    published_by TEXT,
    UNIQUE (company_id, document_template_id, locale, template_version_id)
);

CREATE TABLE phase09_template_retirements (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    template_version_id TEXT NOT NULL REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    retired_at TEXT NOT NULL,
    retired_by TEXT,
    reason TEXT,
    UNIQUE (company_id, template_version_id)
);

CREATE TABLE phase09_rendered_documents (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_type TEXT NOT NULL CHECK (length(trim(document_type)) > 0),
    source_document_id TEXT NOT NULL CHECK (length(trim(source_document_id)) > 0),
    source_document_number TEXT NOT NULL CHECK (length(trim(source_document_number)) > 0),
    source_document_status TEXT NOT NULL CHECK (length(trim(source_document_status)) > 0),
    document_template_id TEXT NOT NULL REFERENCES document_templates(id) ON DELETE RESTRICT,
    template_version_id TEXT NOT NULL REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    locale TEXT NOT NULL CHECK (locale IN ('ar', 'fr')),
    canonical_payload_json TEXT NOT NULL CHECK (json_valid(canonical_payload_json)),
    rendered_html TEXT NOT NULL CHECK (length(trim(rendered_html)) > 0),
    rendered_css TEXT NOT NULL,
    content_sha256 TEXT NOT NULL CHECK (length(content_sha256) = 64),
    pdf_relative_path TEXT NOT NULL CHECK (length(trim(pdf_relative_path)) > 0),
    pdf_sha256 TEXT NOT NULL CHECK (length(pdf_sha256) = 64),
    pdf_size_bytes INTEGER NOT NULL CHECK (pdf_size_bytes > 0),
    rendered_at TEXT NOT NULL,
    rendered_by TEXT,
    UNIQUE (company_id, id),
    UNIQUE (company_id, pdf_relative_path)
);

CREATE TABLE phase09_backup_settings (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL UNIQUE REFERENCES companies(id) ON DELETE RESTRICT,
    automatic_enabled INTEGER NOT NULL DEFAULT 1 CHECK (automatic_enabled IN (0, 1)),
    daily_enabled INTEGER NOT NULL DEFAULT 1 CHECK (daily_enabled IN (0, 1)),
    weekly_enabled INTEGER NOT NULL DEFAULT 1 CHECK (weekly_enabled IN (0, 1)),
    weekly_day INTEGER NOT NULL DEFAULT 5 CHECK (weekly_day BETWEEN 1 AND 7),
    last_daily_local_date TEXT CHECK (last_daily_local_date IS NULL OR length(last_daily_local_date) = 10),
    last_weekly_local_date TEXT CHECK (last_weekly_local_date IS NULL OR length(last_weekly_local_date) = 10),
    last_attempt_at TEXT,
    last_warning_code TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE TABLE phase09_backups (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    backup_kind TEXT NOT NULL CHECK (backup_kind IN ('MANUAL', 'AUTOMATIC_DAILY', 'AUTOMATIC_WEEKLY', 'PRE_RESTORE')),
    created_at TEXT NOT NULL,
    created_by TEXT,
    application_version TEXT NOT NULL CHECK (length(trim(application_version)) > 0),
    schema_version TEXT NOT NULL CHECK (length(trim(schema_version)) > 0),
    migration_ledger_digest TEXT NOT NULL CHECK (length(migration_ledger_digest) = 64),
    database_size_bytes INTEGER NOT NULL CHECK (database_size_bytes > 0),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64),
    relative_path TEXT NOT NULL CHECK (length(trim(relative_path)) > 0),
    integrity_status TEXT NOT NULL CHECK (integrity_status IN ('OK', 'FAILED', 'NOT_RUN')),
    foreign_key_status TEXT NOT NULL CHECK (foreign_key_status IN ('OK', 'FAILED', 'NOT_RUN')),
    verification_status TEXT NOT NULL CHECK (verification_status IN ('VERIFIED', 'FAILED', 'PENDING')),
    failure_reason TEXT,
    imported INTEGER NOT NULL DEFAULT 0 CHECK (imported IN (0, 1)),
    protected_for_restore INTEGER NOT NULL DEFAULT 0 CHECK (protected_for_restore IN (0, 1)),
    deletion_failure TEXT,
    verified_at TEXT,
    UNIQUE (company_id, relative_path),
    CHECK (
        (verification_status = 'VERIFIED' AND integrity_status = 'OK' AND foreign_key_status = 'OK' AND verified_at IS NOT NULL)
        OR verification_status <> 'VERIFIED'
    )
);

CREATE TABLE phase09_restore_attempts (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    backup_id TEXT NOT NULL REFERENCES phase09_backups(id) ON DELETE RESTRICT,
    pre_restore_backup_id TEXT REFERENCES phase09_backups(id) ON DELETE RESTRICT,
    requested_at TEXT NOT NULL,
    requested_by TEXT,
    completed_at TEXT,
    outcome TEXT NOT NULL CHECK (outcome IN ('STARTED', 'SUCCESS', 'FAILED', 'ROLLED_BACK')),
    failure_code TEXT,
    details_json TEXT CHECK (details_json IS NULL OR json_valid(details_json))
);

CREATE INDEX idx_phase09_template_drafts_company_type_locale_state
    ON phase09_template_drafts (company_id, document_type, locale, state, version_number DESC);
CREATE INDEX idx_phase09_template_configs_company_type_locale
    ON phase09_template_version_configs (company_id, document_type, locale, published_at DESC);
CREATE INDEX idx_phase09_retirements_company_version
    ON phase09_template_retirements (company_id, template_version_id);
CREATE INDEX idx_phase09_rendered_company_source
    ON phase09_rendered_documents (company_id, document_type, source_document_id, rendered_at DESC);
CREATE INDEX idx_phase09_rendered_company_time
    ON phase09_rendered_documents (company_id, rendered_at DESC);
CREATE INDEX idx_phase09_backups_company_kind_time
    ON phase09_backups (company_id, backup_kind, created_at DESC);
CREATE INDEX idx_phase09_backups_company_verified
    ON phase09_backups (company_id, verification_status, created_at DESC);
CREATE INDEX idx_phase09_restore_company_time
    ON phase09_restore_attempts (company_id, requested_at DESC);
CREATE INDEX idx_phase09_audit_workspace
    ON audit_logs (company_id, occurred_at DESC, action_code, entity_type, outcome);
CREATE INDEX idx_phase09_audit_actor_time
    ON audit_logs (company_id, actor_user_id, occurred_at DESC);
CREATE INDEX idx_phase09_documents_report
    ON commercial_documents (company_id, document_type, commercial_date, posting_status, partner_id);
CREATE INDEX idx_phase09_stock_report
    ON stock_balances (company_id, warehouse_id, product_id);

CREATE TRIGGER trg_phase09_template_draft_company_scope_insert
BEFORE INSERT ON phase09_template_drafts
WHEN NOT EXISTS (
    SELECT 1 FROM document_templates t
    WHERE t.id = NEW.document_template_id
      AND t.company_id = NEW.company_id
      AND t.document_type = NEW.document_type
) OR (
    NEW.base_template_version_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM document_template_versions v
        WHERE v.id = NEW.base_template_version_id
          AND v.company_id = NEW.company_id
          AND v.document_template_id = NEW.document_template_id
    )
)
BEGIN
    SELECT RAISE(ABORT, 'PHASE09_TEMPLATE_DRAFT_COMPANY_SCOPE');
END;

CREATE TRIGGER trg_phase09_template_draft_company_scope_update
BEFORE UPDATE OF company_id, document_template_id, document_type, base_template_version_id
ON phase09_template_drafts
WHEN NOT EXISTS (
    SELECT 1 FROM document_templates t
    WHERE t.id = NEW.document_template_id
      AND t.company_id = NEW.company_id
      AND t.document_type = NEW.document_type
) OR (
    NEW.base_template_version_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM document_template_versions v
        WHERE v.id = NEW.base_template_version_id
          AND v.company_id = NEW.company_id
          AND v.document_template_id = NEW.document_template_id
    )
)
BEGIN
    SELECT RAISE(ABORT, 'PHASE09_TEMPLATE_DRAFT_COMPANY_SCOPE');
END;

CREATE TRIGGER trg_phase09_template_config_company_scope_insert
BEFORE INSERT ON phase09_template_version_configs
WHEN NOT EXISTS (
    SELECT 1
    FROM document_template_versions v
    JOIN document_templates t ON t.id = v.document_template_id
    JOIN phase09_template_drafts d ON d.id = NEW.source_draft_id
    WHERE v.id = NEW.template_version_id
      AND v.company_id = NEW.company_id
      AND v.document_template_id = NEW.document_template_id
      AND t.company_id = NEW.company_id
      AND t.document_type = NEW.document_type
      AND d.company_id = NEW.company_id
      AND d.document_template_id = NEW.document_template_id
      AND d.document_type = NEW.document_type
      AND d.locale = NEW.locale
)
BEGIN
    SELECT RAISE(ABORT, 'PHASE09_TEMPLATE_CONFIG_COMPANY_SCOPE');
END;

CREATE TRIGGER trg_phase09_template_retirement_company_scope_insert
BEFORE INSERT ON phase09_template_retirements
WHEN NOT EXISTS (
    SELECT 1 FROM document_template_versions v
    WHERE v.id = NEW.template_version_id
      AND v.company_id = NEW.company_id
)
BEGIN
    SELECT RAISE(ABORT, 'PHASE09_TEMPLATE_RETIREMENT_COMPANY_SCOPE');
END;

CREATE TRIGGER trg_phase09_rendered_document_company_scope_insert
BEFORE INSERT ON phase09_rendered_documents
WHEN NOT EXISTS (
    SELECT 1
    FROM document_templates t
    JOIN document_template_versions v ON v.document_template_id = t.id
    WHERE t.id = NEW.document_template_id
      AND t.company_id = NEW.company_id
      AND t.document_type = NEW.document_type
      AND v.id = NEW.template_version_id
      AND v.company_id = NEW.company_id
)
BEGIN
    SELECT RAISE(ABORT, 'PHASE09_RENDERED_DOCUMENT_COMPANY_SCOPE');
END;

CREATE TRIGGER trg_phase09_restore_attempt_company_scope_insert
BEFORE INSERT ON phase09_restore_attempts
WHEN NOT EXISTS (
    SELECT 1 FROM phase09_backups b
    WHERE b.id = NEW.backup_id
      AND b.company_id = NEW.company_id
) OR (
    NEW.pre_restore_backup_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM phase09_backups p
        WHERE p.id = NEW.pre_restore_backup_id
          AND p.company_id = NEW.company_id
    )
)
BEGIN
    SELECT RAISE(ABORT, 'PHASE09_RESTORE_ATTEMPT_COMPANY_SCOPE');
END;

CREATE TRIGGER trg_phase09_template_draft_published_immutable_update
BEFORE UPDATE ON phase09_template_drafts
WHEN OLD.state <> 'DRAFT'
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_DRAFT_IMMUTABLE');
END;

CREATE TRIGGER trg_phase09_template_draft_published_immutable_delete
BEFORE DELETE ON phase09_template_drafts
WHEN OLD.state <> 'DRAFT'
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_DRAFT_IMMUTABLE');
END;

CREATE TRIGGER trg_phase09_template_config_immutable_update
BEFORE UPDATE ON phase09_template_version_configs
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_CONFIG_IMMUTABLE');
END;

CREATE TRIGGER trg_phase09_template_config_immutable_delete
BEFORE DELETE ON phase09_template_version_configs
BEGIN
    SELECT RAISE(ABORT, 'PUBLISHED_TEMPLATE_CONFIG_IMMUTABLE');
END;

CREATE TRIGGER trg_phase09_template_retirement_append_only_update
BEFORE UPDATE ON phase09_template_retirements
BEGIN
    SELECT RAISE(ABORT, 'TEMPLATE_RETIREMENT_APPEND_ONLY');
END;

CREATE TRIGGER trg_phase09_template_retirement_append_only_delete
BEFORE DELETE ON phase09_template_retirements
BEGIN
    SELECT RAISE(ABORT, 'TEMPLATE_RETIREMENT_APPEND_ONLY');
END;

CREATE TRIGGER trg_phase09_rendered_document_immutable_update
BEFORE UPDATE ON phase09_rendered_documents
BEGIN
    SELECT RAISE(ABORT, 'HISTORICAL_RENDER_IMMUTABLE');
END;

CREATE TRIGGER trg_phase09_rendered_document_immutable_delete
BEFORE DELETE ON phase09_rendered_documents
BEGIN
    SELECT RAISE(ABORT, 'HISTORICAL_RENDER_IMMUTABLE');
END;

CREATE TRIGGER trg_phase09_restore_attempt_append_only_update
BEFORE UPDATE ON phase09_restore_attempts
WHEN OLD.outcome <> 'STARTED'
BEGIN
    SELECT RAISE(ABORT, 'RESTORE_ATTEMPT_APPEND_ONLY');
END;

CREATE TRIGGER trg_phase09_restore_attempt_append_only_delete
BEFORE DELETE ON phase09_restore_attempts
BEGIN
    SELECT RAISE(ABORT, 'RESTORE_ATTEMPT_APPEND_ONLY');
END;

INSERT OR IGNORE INTO permissions (id, code, domain, description_ar, description_fr, is_sensitive, created_at) VALUES
    ('perm-p09-doc-tpl-view', 'documents.templates.view', 'documents', 'عرض قوالب المستندات', 'Consulter les modèles de documents', 0, '2026-08-06T00:00:00Z'),
    ('perm-p09-doc-tpl-manage', 'documents.templates.manage', 'documents', 'إدارة ونشر قوالب المستندات', 'Gérer et publier les modèles de documents', 1, '2026-08-06T00:00:00Z'),
    ('perm-p09-doc-render', 'documents.render', 'documents', 'إنشاء مستند تاريخي', 'Générer un document historique', 1, '2026-08-06T00:00:00Z'),
    ('perm-p09-doc-print', 'documents.print', 'documents', 'طباعة المستندات', 'Imprimer les documents', 0, '2026-08-06T00:00:00Z'),
    ('perm-p09-doc-export', 'documents.export', 'documents', 'تصدير المستندات', 'Exporter les documents', 0, '2026-08-06T00:00:00Z'),
    ('perm-p09-reports-view', 'reports.view', 'reports', 'عرض التقارير التشغيلية', 'Consulter les rapports opérationnels', 0, '2026-08-06T00:00:00Z'),
    ('perm-p09-reports-export', 'reports.export', 'reports', 'تصدير التقارير التشغيلية', 'Exporter les rapports opérationnels', 0, '2026-08-06T00:00:00Z'),
    ('perm-p09-audit-view', 'audit.view', 'audit', 'عرض سجل التدقيق', 'Consulter le journal d’audit', 1, '2026-08-06T00:00:00Z'),
    ('perm-p09-audit-export', 'audit.export', 'audit', 'تصدير سجل التدقيق', 'Exporter le journal d’audit', 1, '2026-08-06T00:00:00Z'),
    ('perm-p09-backup-view', 'backup.view', 'backup', 'عرض النسخ الاحتياطية', 'Consulter les sauvegardes', 0, '2026-08-06T00:00:00Z'),
    ('perm-backup-create', 'backup.create', 'backup', 'إنشاء نسخة احتياطية موثقة', 'Créer une sauvegarde vérifiée', 1, '2026-08-06T00:00:00Z'),
    ('perm-backup-restore', 'backup.restore', 'backup', 'استعادة نسخة احتياطية موثقة', 'Restaurer une sauvegarde vérifiée', 1, '2026-08-06T00:00:00Z'),
    ('perm-p09-backup-manage', 'backup.manage', 'backup', 'إدارة سياسة النسخ الاحتياطي', 'Gérer la politique de sauvegarde', 1, '2026-08-06T00:00:00Z');

INSERT OR IGNORE INTO role_permissions (id, company_id, role_id, permission_id, granted_at, granted_by)
SELECT 'rp-p09-' || r.id || '-' || p.id, r.company_id, r.id, p.id, '2026-08-06T00:00:00Z', NULL
FROM roles r
JOIN permissions p ON p.code IN (
    'documents.templates.view', 'documents.templates.manage', 'documents.render', 'documents.print',
    'documents.export', 'reports.view', 'reports.export', 'audit.view', 'audit.export',
    'backup.view', 'backup.create', 'backup.restore', 'backup.manage'
)
WHERE r.code IN ('OWNER', 'SYSTEM_ADMINISTRATOR') AND r.is_active = 1;
