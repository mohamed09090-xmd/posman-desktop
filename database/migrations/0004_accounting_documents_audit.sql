-- POSMAN Phase 01 - accounting, printing metadata, audit, idempotency, and backups.

CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    parent_account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    account_type TEXT NOT NULL CHECK (account_type IN ('ASSET', 'LIABILITY', 'EQUITY', 'REVENUE', 'EXPENSE', 'OFF_BALANCE')),
    normal_side TEXT NOT NULL CHECK (normal_side IN ('DEBIT', 'CREDIT')),
    allow_posting INTEGER NOT NULL DEFAULT 1 CHECK (allow_posting IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code),
    CHECK (parent_account_id IS NULL OR parent_account_id <> id)
);

CREATE TABLE accounting_journals (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    journal_type TEXT NOT NULL CHECK (journal_type IN ('SALES', 'PURCHASES', 'CASH', 'BANK', 'INVENTORY', 'GENERAL')),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code)
);

CREATE TABLE posting_rules (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    accounting_journal_id TEXT NOT NULL REFERENCES accounting_journals(id) ON DELETE RESTRICT,
    debit_account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    credit_account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    source_event_type TEXT NOT NULL CHECK (length(trim(source_event_type)) > 0),
    condition_expression TEXT,
    priority INTEGER NOT NULL DEFAULT 100 CHECK (priority >= 0),
    valid_from TEXT NOT NULL CHECK (length(valid_from) = 10),
    valid_to TEXT CHECK (valid_to IS NULL OR (length(valid_to) = 10 AND valid_to >= valid_from)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code, valid_from),
    CHECK (debit_account_id <> credit_account_id)
);

CREATE TABLE journal_entries (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    fiscal_period_id TEXT NOT NULL REFERENCES fiscal_periods(id) ON DELETE RESTRICT,
    accounting_journal_id TEXT NOT NULL REFERENCES accounting_journals(id) ON DELETE RESTRICT,
    source_document_id TEXT REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    reversal_of_entry_id TEXT REFERENCES journal_entries(id) ON DELETE RESTRICT,
    entry_number TEXT NOT NULL CHECK (length(trim(entry_number)) > 0),
    entry_date TEXT NOT NULL CHECK (length(entry_date) = 10),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'POSTED', 'REVERSED')),
    source_event_type TEXT NOT NULL CHECK (length(trim(source_event_type)) > 0),
    source_event_id TEXT NOT NULL CHECK (length(trim(source_event_id)) > 0),
    idempotency_key TEXT NOT NULL CHECK (length(trim(idempotency_key)) > 0),
    memo TEXT,
    posted_at TEXT,
    posted_by TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, fiscal_year_id, accounting_journal_id, entry_number),
    UNIQUE (company_id, idempotency_key),
    CHECK (reversal_of_entry_id IS NULL OR reversal_of_entry_id <> id)
);

CREATE TABLE journal_entry_lines (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    journal_entry_id TEXT NOT NULL REFERENCES journal_entries(id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    partner_id TEXT REFERENCES partners(id) ON DELETE RESTRICT,
    product_id TEXT REFERENCES products(id) ON DELETE RESTRICT,
    line_number INTEGER NOT NULL CHECK (line_number >= 1),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    debit_minor INTEGER NOT NULL DEFAULT 0 CHECK (debit_minor >= 0),
    credit_minor INTEGER NOT NULL DEFAULT 0 CHECK (credit_minor >= 0),
    created_at TEXT NOT NULL,
    created_by TEXT,
    UNIQUE (journal_entry_id, line_number),
    CHECK (
        (debit_minor > 0 AND credit_minor = 0)
        OR (credit_minor > 0 AND debit_minor = 0)
    )
);

CREATE TABLE posting_attempts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    result_entry_id TEXT REFERENCES journal_entries(id) ON DELETE RESTRICT,
    retry_of_attempt_id TEXT REFERENCES posting_attempts(id) ON DELETE RESTRICT,
    source_event_type TEXT NOT NULL CHECK (length(trim(source_event_type)) > 0),
    source_event_id TEXT NOT NULL CHECK (length(trim(source_event_id)) > 0),
    idempotency_key TEXT NOT NULL CHECK (length(trim(idempotency_key)) > 0),
    attempt_number INTEGER NOT NULL CHECK (attempt_number >= 1),
    status TEXT NOT NULL CHECK (status IN ('STARTED', 'SUCCEEDED', 'FAILED')),
    error_code TEXT,
    error_message TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    CHECK (retry_of_attempt_id IS NULL OR retry_of_attempt_id <> id),
    CHECK ((status = 'STARTED' AND completed_at IS NULL) OR (status <> 'STARTED' AND completed_at IS NOT NULL))
);

CREATE TABLE document_templates (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    document_type TEXT NOT NULL CHECK (length(trim(document_type)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code)
);

CREATE TABLE document_template_versions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_template_id TEXT NOT NULL REFERENCES document_templates(id) ON DELETE RESTRICT,
    version_number INTEGER NOT NULL CHECK (version_number >= 1),
    html_template TEXT NOT NULL CHECK (length(trim(html_template)) > 0),
    css_template TEXT NOT NULL,
    content_hash_sha256 TEXT NOT NULL CHECK (length(content_hash_sha256) = 64),
    is_published INTEGER NOT NULL DEFAULT 0 CHECK (is_published IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    UNIQUE (document_template_id, version_number),
    UNIQUE (company_id, content_hash_sha256)
);

CREATE TABLE rendered_documents (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    source_document_id TEXT NOT NULL REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    template_version_id TEXT NOT NULL REFERENCES document_template_versions(id) ON DELETE RESTRICT,
    file_format TEXT NOT NULL CHECK (file_format IN ('PDF', 'HTML')),
    relative_file_path TEXT NOT NULL CHECK (length(trim(relative_file_path)) > 0),
    content_hash_sha256 TEXT NOT NULL CHECK (length(content_hash_sha256) = 64),
    rendered_at TEXT NOT NULL,
    rendered_by TEXT,
    UNIQUE (source_document_id, content_hash_sha256)
);

CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    entity_type TEXT NOT NULL CHECK (length(trim(entity_type)) > 0),
    entity_id TEXT NOT NULL CHECK (length(trim(entity_id)) > 0),
    original_file_name TEXT NOT NULL CHECK (length(trim(original_file_name)) > 0),
    relative_file_path TEXT NOT NULL CHECK (length(trim(relative_file_path)) > 0),
    mime_type TEXT NOT NULL CHECK (length(trim(mime_type)) > 0),
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    content_hash_sha256 TEXT NOT NULL CHECK (length(content_hash_sha256) = 64),
    created_at TEXT NOT NULL,
    created_by TEXT,
    UNIQUE (company_id, entity_type, entity_id, content_hash_sha256)
);

CREATE TABLE audit_logs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    actor_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    action_code TEXT NOT NULL CHECK (length(trim(action_code)) > 0),
    entity_type TEXT NOT NULL CHECK (length(trim(entity_type)) > 0),
    entity_id TEXT NOT NULL CHECK (length(trim(entity_id)) > 0),
    occurred_at TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('SUCCESS', 'FAILURE', 'DENIED')),
    correlation_id TEXT,
    details_json TEXT
);

CREATE TABLE idempotency_keys (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    namespace TEXT NOT NULL CHECK (length(trim(namespace)) > 0),
    idempotency_key TEXT NOT NULL CHECK (length(trim(idempotency_key)) > 0),
    request_hash_sha256 TEXT NOT NULL CHECK (length(request_hash_sha256) = 64),
    status TEXT NOT NULL CHECK (status IN ('IN_PROGRESS', 'SUCCEEDED', 'FAILED')),
    result_entity_type TEXT,
    result_entity_id TEXT,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    expires_at TEXT,
    UNIQUE (company_id, namespace, idempotency_key),
    CHECK ((status = 'IN_PROGRESS' AND completed_at IS NULL) OR (status <> 'IN_PROGRESS' AND completed_at IS NOT NULL))
);

CREATE TABLE backup_history (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    backup_kind TEXT NOT NULL CHECK (backup_kind IN ('AUTOMATIC', 'MANUAL', 'PRE_IMPORT', 'PRE_RESTORE', 'PRE_RESET')),
    relative_file_path TEXT NOT NULL CHECK (length(trim(relative_file_path)) > 0),
    schema_version TEXT NOT NULL CHECK (length(trim(schema_version)) > 0),
    content_hash_sha256 TEXT,
    size_bytes INTEGER CHECK (size_bytes IS NULL OR size_bytes >= 0),
    status TEXT NOT NULL CHECK (status IN ('STARTED', 'VERIFIED', 'FAILED')),
    started_at TEXT NOT NULL,
    completed_at TEXT,
    error_message TEXT,
    created_by TEXT,
    CHECK (content_hash_sha256 IS NULL OR length(content_hash_sha256) = 64),
    CHECK ((status = 'STARTED' AND completed_at IS NULL) OR (status <> 'STARTED' AND completed_at IS NOT NULL))
);

CREATE INDEX idx_accounts_company_active ON accounts(company_id, is_active, code);
CREATE INDEX idx_posting_rules_lookup ON posting_rules(company_id, source_event_type, valid_from, valid_to, priority);
CREATE INDEX idx_journal_entries_source ON journal_entries(company_id, source_event_type, source_event_id);
CREATE INDEX idx_posting_attempts_source ON posting_attempts(company_id, source_event_type, source_event_id, attempt_number);
CREATE INDEX idx_audit_logs_entity ON audit_logs(company_id, entity_type, entity_id, occurred_at);

CREATE TRIGGER trg_journal_entries_no_direct_posted_insert
BEFORE INSERT ON journal_entries
WHEN NEW.status = 'POSTED'
BEGIN
    SELECT RAISE(ABORT, 'journal entry must be inserted as DRAFT and posted through the documented transition');
END;

CREATE TRIGGER trg_journal_entries_validate_posting
BEFORE UPDATE OF status ON journal_entries
WHEN NEW.status = 'POSTED' AND OLD.status <> 'POSTED'
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1
        FROM fiscal_periods period
        WHERE period.id = NEW.fiscal_period_id
          AND period.company_id = NEW.company_id
          AND period.fiscal_year_id = NEW.fiscal_year_id
          AND period.status = 'OPEN'
          AND NEW.entry_date BETWEEN period.starts_on AND period.ends_on
    ) THEN RAISE(ABORT, 'journal entry date must belong to an open fiscal period') END;

    SELECT CASE WHEN (
        SELECT COUNT(*) FROM journal_entry_lines line
        WHERE line.journal_entry_id = NEW.id
    ) < 2 THEN RAISE(ABORT, 'journal entry requires at least two lines') END;

    SELECT CASE WHEN (
        SELECT COALESCE(SUM(line.debit_minor), 0)
        FROM journal_entry_lines line
        WHERE line.journal_entry_id = NEW.id
    ) <= 0 THEN RAISE(ABORT, 'journal entry total must be positive') END;

    SELECT CASE WHEN (
        SELECT COALESCE(SUM(line.debit_minor), 0)
        FROM journal_entry_lines line
        WHERE line.journal_entry_id = NEW.id
    ) <> (
        SELECT COALESCE(SUM(line.credit_minor), 0)
        FROM journal_entry_lines line
        WHERE line.journal_entry_id = NEW.id
    ) THEN RAISE(ABORT, 'journal entry is not balanced') END;
END;

CREATE TRIGGER trg_journal_entries_posted_no_update
BEFORE UPDATE ON journal_entries
WHEN OLD.status = 'POSTED'
BEGIN
    SELECT RAISE(ABORT, 'posted journal entry is immutable');
END;

CREATE TRIGGER trg_journal_entries_posted_no_delete
BEFORE DELETE ON journal_entries
WHEN OLD.status = 'POSTED'
BEGIN
    SELECT RAISE(ABORT, 'posted journal entry cannot be deleted');
END;

CREATE TRIGGER trg_journal_lines_posted_no_insert
BEFORE INSERT ON journal_entry_lines
WHEN EXISTS (
    SELECT 1 FROM journal_entries
    WHERE id = NEW.journal_entry_id AND status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'cannot add a line to a posted journal entry');
END;

CREATE TRIGGER trg_journal_lines_posted_no_update
BEFORE UPDATE ON journal_entry_lines
WHEN EXISTS (
    SELECT 1 FROM journal_entries
    WHERE id = OLD.journal_entry_id AND status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'posted journal entry line is immutable');
END;

CREATE TRIGGER trg_journal_lines_posted_no_delete
BEFORE DELETE ON journal_entry_lines
WHEN EXISTS (
    SELECT 1 FROM journal_entries
    WHERE id = OLD.journal_entry_id AND status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'posted journal entry line cannot be deleted');
END;

CREATE TRIGGER trg_document_template_versions_no_update
BEFORE UPDATE ON document_template_versions
BEGIN
    SELECT RAISE(ABORT, 'document template versions are immutable');
END;

CREATE TRIGGER trg_document_template_versions_no_delete
BEFORE DELETE ON document_template_versions
BEGIN
    SELECT RAISE(ABORT, 'document template versions are immutable');
END;

CREATE TRIGGER trg_rendered_documents_no_update
BEFORE UPDATE ON rendered_documents
BEGIN
    SELECT RAISE(ABORT, 'rendered document history is immutable');
END;

CREATE TRIGGER trg_rendered_documents_no_delete
BEFORE DELETE ON rendered_documents
BEGIN
    SELECT RAISE(ABORT, 'rendered document history is immutable');
END;

CREATE TRIGGER trg_audit_logs_no_update
BEFORE UPDATE ON audit_logs
BEGIN
    SELECT RAISE(ABORT, 'audit log is append-only');
END;

CREATE TRIGGER trg_audit_logs_no_delete
BEFORE DELETE ON audit_logs
BEGIN
    SELECT RAISE(ABORT, 'audit log is append-only');
END;
