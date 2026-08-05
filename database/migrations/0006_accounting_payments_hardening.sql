-- POSMAN PHASE 08 - automatic accounting, payments, settlement, and fiscal-period hardening.
-- Accepted migrations 0001-0005 remain frozen. This migration is additive only.

CREATE TABLE accounting_setups (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL UNIQUE REFERENCES companies(id) ON DELETE RESTRICT,
    automatic_posting_enabled INTEGER NOT NULL DEFAULT 0 CHECK (automatic_posting_enabled IN (0, 1)),
    configuration_verified_at TEXT,
    configuration_verified_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    starter_template_installed_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    CHECK (
        automatic_posting_enabled = 0
        OR (configuration_verified_at IS NOT NULL AND configuration_verified_by IS NOT NULL)
    )
);

CREATE TABLE accounting_account_roles (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    role_code TEXT NOT NULL CHECK (role_code IN (
        'CUSTOMER_RECEIVABLE',
        'SUPPLIER_PAYABLE',
        'CASH',
        'BANK',
        'SALES_REVENUE',
        'SALES_RETURNS',
        'COLLECTED_TAX',
        'RECOVERABLE_TAX',
        'INVENTORY',
        'COST_OF_GOODS_SOLD',
        'PURCHASE_RETURNS',
        'INVENTORY_VARIANCE',
        'ROUNDING_DIFFERENCE'
    )),
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, role_code)
);

CREATE TABLE payment_method_accounting (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    payment_method_id TEXT NOT NULL REFERENCES payment_methods(id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    accounting_journal_id TEXT NOT NULL REFERENCES accounting_journals(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, payment_method_id)
);

CREATE TABLE posting_rule_lines (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    posting_rule_id TEXT NOT NULL REFERENCES posting_rules(id) ON DELETE RESTRICT,
    line_number INTEGER NOT NULL CHECK (line_number >= 1),
    side TEXT NOT NULL CHECK (side IN ('DEBIT', 'CREDIT')),
    account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    account_role_code TEXT CHECK (account_role_code IS NULL OR account_role_code IN (
        'CUSTOMER_RECEIVABLE',
        'SUPPLIER_PAYABLE',
        'CASH',
        'BANK',
        'SALES_REVENUE',
        'SALES_RETURNS',
        'COLLECTED_TAX',
        'RECOVERABLE_TAX',
        'INVENTORY',
        'COST_OF_GOODS_SOLD',
        'PURCHASE_RETURNS',
        'INVENTORY_VARIANCE',
        'ROUNDING_DIFFERENCE'
    )),
    amount_component TEXT NOT NULL CHECK (amount_component IN (
        'DOCUMENT_HT',
        'DOCUMENT_TAX',
        'DOCUMENT_TTC',
        'INVENTORY_COST',
        'PAYMENT_AMOUNT',
        'ALLOCATED_AMOUNT',
        'ROUNDING_DIFFERENCE'
    )),
    partner_dimension TEXT NOT NULL DEFAULT 'NONE'
        CHECK (partner_dimension IN ('NONE', 'SOURCE_PARTNER', 'REQUIRED')),
    product_dimension TEXT NOT NULL DEFAULT 'NONE'
        CHECK (product_dimension IN ('NONE', 'SOURCE_PRODUCT', 'REQUIRED')),
    description_ar TEXT NOT NULL CHECK (length(trim(description_ar)) > 0),
    description_fr TEXT NOT NULL CHECK (length(trim(description_fr)) > 0),
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (posting_rule_id, line_number),
    CHECK (
        (account_id IS NOT NULL AND account_role_code IS NULL)
        OR (account_id IS NULL AND account_role_code IS NOT NULL)
    )
);

CREATE TABLE fiscal_period_events (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    fiscal_period_id TEXT NOT NULL REFERENCES fiscal_periods(id) ON DELETE RESTRICT,
    event_type TEXT NOT NULL CHECK (event_type IN ('CLOSED', 'REOPENED')),
    reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
    previous_status TEXT NOT NULL CHECK (previous_status IN ('OPEN', 'CLOSED')),
    new_status TEXT NOT NULL CHECK (new_status IN ('OPEN', 'CLOSED')),
    previous_row_version INTEGER NOT NULL CHECK (previous_row_version >= 1),
    new_row_version INTEGER NOT NULL CHECK (new_row_version = previous_row_version + 1),
    occurred_at TEXT NOT NULL,
    actor_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    CHECK (
        (event_type = 'CLOSED' AND previous_status = 'OPEN' AND new_status = 'CLOSED')
        OR (event_type = 'REOPENED' AND previous_status = 'CLOSED' AND new_status = 'OPEN')
    )
);

ALTER TABLE journal_entries ADD COLUMN request_hash_sha256 TEXT
    CHECK (
        request_hash_sha256 IS NULL
        OR (
            length(request_hash_sha256) = 64
            AND request_hash_sha256 NOT GLOB '*[^0-9a-f]*'
        )
    );
ALTER TABLE journal_entries ADD COLUMN reversal_reason TEXT
    CHECK (reversal_reason IS NULL OR length(trim(reversal_reason)) > 0);

ALTER TABLE posting_attempts ADD COLUMN request_hash_sha256 TEXT
    CHECK (
        request_hash_sha256 IS NULL
        OR (
            length(request_hash_sha256) = 64
            AND request_hash_sha256 NOT GLOB '*[^0-9a-f]*'
        )
    );

ALTER TABLE payments ADD COLUMN journal_entry_id TEXT REFERENCES journal_entries(id) ON DELETE RESTRICT;
ALTER TABLE payments ADD COLUMN reversal_of_payment_id TEXT REFERENCES payments(id) ON DELETE RESTRICT;
ALTER TABLE payments ADD COLUMN reversal_reason TEXT
    CHECK (reversal_reason IS NULL OR length(trim(reversal_reason)) > 0);
ALTER TABLE payments ADD COLUMN posted_at TEXT;
ALTER TABLE payments ADD COLUMN posted_by TEXT REFERENCES users(id) ON DELETE SET NULL;

CREATE UNIQUE INDEX uq_journal_entries_source_event
    ON journal_entries(company_id, source_event_type, source_event_id)
    WHERE reversal_of_entry_id IS NULL;

CREATE UNIQUE INDEX uq_payment_reversal
    ON payments(company_id, reversal_of_payment_id)
    WHERE reversal_of_payment_id IS NOT NULL;

CREATE UNIQUE INDEX uq_payment_allocation_active_document
    ON payment_allocations(payment_id, document_id)
    WHERE allocation_status = 'ACTIVE';

CREATE UNIQUE INDEX uq_payment_allocation_reversal
    ON payment_allocations(company_id, reversal_of_allocation_id)
    WHERE reversal_of_allocation_id IS NOT NULL;

CREATE INDEX idx_posting_rule_lines_lookup
    ON posting_rule_lines(company_id, posting_rule_id, line_number);

CREATE INDEX idx_accounting_account_roles_lookup
    ON accounting_account_roles(company_id, role_code, account_id);

CREATE INDEX idx_payment_method_accounting_lookup
    ON payment_method_accounting(company_id, payment_method_id);

CREATE INDEX idx_payments_partner_open
    ON payments(company_id, partner_id, payment_kind, status, commercial_date);

CREATE INDEX idx_payment_allocations_document
    ON payment_allocations(company_id, document_id, allocation_status);

CREATE INDEX idx_fiscal_period_events_period
    ON fiscal_period_events(company_id, fiscal_period_id, occurred_at);

CREATE TRIGGER trg_accounting_account_roles_company_scope_insert
BEFORE INSERT ON accounting_account_roles
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounts
        WHERE id = NEW.account_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'account role account must belong to the same company') END;
END;

CREATE TRIGGER trg_accounting_account_roles_company_scope_update
BEFORE UPDATE ON accounting_account_roles
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounts
        WHERE id = NEW.account_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'account role account must belong to the same company') END;
END;

CREATE TRIGGER trg_payment_method_accounting_company_scope_insert
BEFORE INSERT ON payment_method_accounting
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM payment_methods
        WHERE id = NEW.payment_method_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'payment method must belong to the same company') END;
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounts
        WHERE id = NEW.account_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'payment account must belong to the same company') END;
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounting_journals
        WHERE id = NEW.accounting_journal_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'payment journal must belong to the same company') END;
END;

CREATE TRIGGER trg_payment_method_accounting_company_scope_update
BEFORE UPDATE ON payment_method_accounting
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM payment_methods
        WHERE id = NEW.payment_method_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'payment method must belong to the same company') END;
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounts
        WHERE id = NEW.account_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'payment account must belong to the same company') END;
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounting_journals
        WHERE id = NEW.accounting_journal_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'payment journal must belong to the same company') END;
END;

CREATE TRIGGER trg_posting_rule_lines_company_scope_insert
BEFORE INSERT ON posting_rule_lines
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM posting_rules
        WHERE id = NEW.posting_rule_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'posting rule line must belong to the same company') END;
    SELECT CASE WHEN NEW.account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts
        WHERE id = NEW.account_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'posting rule line account must belong to the same company') END;
END;

CREATE TRIGGER trg_posting_rule_lines_company_scope_update
BEFORE UPDATE ON posting_rule_lines
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM posting_rules
        WHERE id = NEW.posting_rule_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'posting rule line must belong to the same company') END;
    SELECT CASE WHEN NEW.account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts
        WHERE id = NEW.account_id AND company_id = NEW.company_id
    ) THEN RAISE(ABORT, 'posting rule line account must belong to the same company') END;
END;

CREATE TRIGGER trg_posting_rules_condition_insert
BEFORE INSERT ON posting_rules
WHEN NEW.condition_expression IS NOT NULL
BEGIN
    SELECT CASE WHEN NOT (
        json_valid(NEW.condition_expression)
        AND json_type(NEW.condition_expression) = 'object'
    ) THEN RAISE(ABORT, 'posting rule condition must be a JSON object') END;
    SELECT CASE WHEN EXISTS (
        SELECT 1 FROM json_each(NEW.condition_expression)
        WHERE key NOT IN ('document_type', 'payment_kind', 'payment_method_kind', 'has_stock_effect')
    ) THEN RAISE(ABORT, 'posting rule condition contains an unsupported key') END;
END;

CREATE TRIGGER trg_posting_rules_condition_update
BEFORE UPDATE OF condition_expression ON posting_rules
WHEN NEW.condition_expression IS NOT NULL
BEGIN
    SELECT CASE WHEN NOT (
        json_valid(NEW.condition_expression)
        AND json_type(NEW.condition_expression) = 'object'
    ) THEN RAISE(ABORT, 'posting rule condition must be a JSON object') END;
    SELECT CASE WHEN EXISTS (
        SELECT 1 FROM json_each(NEW.condition_expression)
        WHERE key NOT IN ('document_type', 'payment_kind', 'payment_method_kind', 'has_stock_effect')
    ) THEN RAISE(ABORT, 'posting rule condition contains an unsupported key') END;
END;

CREATE TRIGGER trg_payments_posted_financial_fields_locked
BEFORE UPDATE OF
    company_id,
    fiscal_year_id,
    fiscal_period_id,
    partner_id,
    payment_method_id,
    payment_number,
    payment_kind,
    commercial_date,
    posting_date,
    amount_minor,
    currency_code,
    external_reference,
    idempotency_key,
    journal_entry_id,
    reversal_of_payment_id
ON payments
WHEN OLD.status IN ('POSTED', 'PARTIALLY_ALLOCATED', 'ALLOCATED', 'REVERSED')
BEGIN
    SELECT RAISE(ABORT, 'posted payment financial fields are immutable');
END;

CREATE TRIGGER trg_payments_posted_no_delete
BEFORE DELETE ON payments
WHEN OLD.status IN ('POSTED', 'PARTIALLY_ALLOCATED', 'ALLOCATED', 'REVERSED')
BEGIN
    SELECT RAISE(ABORT, 'posted payment cannot be deleted');
END;

CREATE TRIGGER trg_payment_allocations_no_update
BEFORE UPDATE ON payment_allocations
BEGIN
    SELECT RAISE(ABORT, 'payment allocation history is immutable');
END;

CREATE TRIGGER trg_payment_allocations_no_delete
BEFORE DELETE ON payment_allocations
BEGIN
    SELECT RAISE(ABORT, 'payment allocation history cannot be deleted');
END;

CREATE TRIGGER trg_payment_allocations_company_scope_insert
BEFORE INSERT ON payment_allocations
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1
        FROM payments payment
        JOIN commercial_documents document
          ON document.id = NEW.document_id
         AND document.company_id = NEW.company_id
        WHERE payment.id = NEW.payment_id
          AND payment.company_id = NEW.company_id
          AND payment.partner_id = document.partner_id
          AND (
              (payment.payment_kind = 'RECEIPT' AND document.document_type IN ('SALES_INVOICE', 'SALES_CREDIT_NOTE'))
              OR
              (payment.payment_kind = 'DISBURSEMENT' AND document.document_type IN ('PURCHASE_INVOICE', 'PURCHASE_CREDIT_NOTE'))
          )
    ) THEN RAISE(ABORT, 'payment allocation scope or partner is invalid') END;
END;

CREATE TRIGGER trg_fiscal_period_events_no_update
BEFORE UPDATE ON fiscal_period_events
BEGIN
    SELECT RAISE(ABORT, 'fiscal period event history is immutable');
END;

CREATE TRIGGER trg_fiscal_period_events_no_delete
BEFORE DELETE ON fiscal_period_events
BEGIN
    SELECT RAISE(ABORT, 'fiscal period event history cannot be deleted');
END;

CREATE TRIGGER trg_journal_entries_reversal_requires_reason
BEFORE INSERT ON journal_entries
WHEN NEW.reversal_of_entry_id IS NOT NULL
BEGIN
    SELECT CASE WHEN NEW.reversal_reason IS NULL OR length(trim(NEW.reversal_reason)) = 0
        THEN RAISE(ABORT, 'journal reversal requires a reason') END;
END;

CREATE TRIGGER trg_payments_reversal_requires_reason
BEFORE INSERT ON payments
WHEN NEW.reversal_of_payment_id IS NOT NULL
BEGIN
    SELECT CASE WHEN NEW.reversal_reason IS NULL OR length(trim(NEW.reversal_reason)) = 0
        THEN RAISE(ABORT, 'payment reversal requires a reason') END;
END;

INSERT OR IGNORE INTO permissions
    (id, code, domain, description_ar, description_fr, is_sensitive, created_at)
VALUES
    ('permission-phase08-accounting-read', 'accounting.read', 'ACCOUNTING', 'قراءة المحاسبة', 'Consulter la comptabilité', 0, '1970-01-01T00:00:00Z'),
    ('permission-phase08-accounting-configure', 'accounting.configure', 'ACCOUNTING', 'تهيئة المحاسبة', 'Configurer la comptabilité', 1, '1970-01-01T00:00:00Z'),
    ('permission-phase08-journal-post', 'journal_entry.post', 'ACCOUNTING', 'ترحيل القيود', 'Comptabiliser les écritures', 1, '1970-01-01T00:00:00Z'),
    ('permission-phase08-journal-reverse', 'journal_entry.reverse', 'ACCOUNTING', 'عكس القيود', 'Contrepasser les écritures', 1, '1970-01-01T00:00:00Z'),
    ('permission-phase08-payment-read', 'payment.read', 'PAYMENT', 'قراءة المدفوعات', 'Consulter les paiements', 0, '1970-01-01T00:00:00Z'),
    ('permission-phase08-payment-post', 'payment.post', 'PAYMENT', 'ترحيل المدفوعات', 'Comptabiliser les paiements', 1, '1970-01-01T00:00:00Z'),
    ('permission-phase08-payment-allocate', 'payment.allocate', 'PAYMENT', 'توزيع المدفوعات', 'Affecter les paiements', 1, '1970-01-01T00:00:00Z'),
    ('permission-phase08-payment-reverse', 'payment.reverse', 'PAYMENT', 'عكس المدفوعات', 'Contrepasser les paiements', 1, '1970-01-01T00:00:00Z'),
    ('permission-phase08-period-close', 'fiscal_period.close', 'ACCOUNTING', 'إغلاق الفترة المالية', 'Clôturer la période comptable', 1, '1970-01-01T00:00:00Z'),
    ('permission-phase08-period-reopen', 'fiscal_period.reopen', 'ACCOUNTING', 'إعادة فتح الفترة المالية', 'Rouvrir la période comptable', 1, '1970-01-01T00:00:00Z');