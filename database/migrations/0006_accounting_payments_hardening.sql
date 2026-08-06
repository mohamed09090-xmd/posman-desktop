-- POSMAN Phase 08 - automatic accounting, immutable journals, payments and allocations.
-- Migration 0006 is additive because accepted migrations 0001-0005 are frozen.

CREATE TABLE accounting_setups (
    company_id TEXT NOT NULL PRIMARY KEY REFERENCES companies(id) ON DELETE RESTRICT,
    is_enabled INTEGER NOT NULL DEFAULT 0 CHECK (is_enabled IN (0, 1)),
    retained_earnings_account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    current_fiscal_year_id TEXT REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE TABLE accounting_account_roles (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    role_code TEXT NOT NULL CHECK (role_code IN (
        'CUSTOMER_RECEIVABLE','SUPPLIER_PAYABLE','CASH','BANK','SALES_REVENUE',
        'SALES_RETURNS','COLLECTED_TAX','RECOVERABLE_TAX','INVENTORY','COGS',
        'PURCHASE_RETURNS','GOODS_RECEIVED_NOT_INVOICED','INVENTORY_VARIANCE','ROUNDING_DIFFERENCE'
    )),
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, role_code)
);

CREATE TABLE posting_rule_lines (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    posting_rule_id TEXT NOT NULL REFERENCES posting_rules(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL CHECK (line_number >= 1),
    side TEXT NOT NULL CHECK (side IN ('DEBIT','CREDIT')),
    account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    account_role_code TEXT CHECK (account_role_code IS NULL OR account_role_code IN (
        'CUSTOMER_RECEIVABLE','SUPPLIER_PAYABLE','CASH','BANK','SALES_REVENUE',
        'SALES_RETURNS','COLLECTED_TAX','RECOVERABLE_TAX','INVENTORY','COGS',
        'PURCHASE_RETURNS','GOODS_RECEIVED_NOT_INVOICED','INVENTORY_VARIANCE','ROUNDING_DIFFERENCE'
    )),
    amount_component TEXT NOT NULL CHECK (amount_component IN (
        'DOCUMENT_HT','DOCUMENT_TAX','DOCUMENT_TTC','STOCK_COST','PAYMENT_AMOUNT',
        'UNALLOCATED_AMOUNT','ROUNDING_AMOUNT'
    )),
    description_ar TEXT NOT NULL CHECK (length(trim(description_ar)) > 0),
    description_fr TEXT,
    partner_dimension INTEGER NOT NULL DEFAULT 0 CHECK (partner_dimension IN (0, 1)),
    product_dimension INTEGER NOT NULL DEFAULT 0 CHECK (product_dimension IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (posting_rule_id, line_number),
    CHECK ((account_id IS NOT NULL AND account_role_code IS NULL)
        OR (account_id IS NULL AND account_role_code IS NOT NULL))
);

CREATE TABLE payment_method_accounting (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    payment_method_id TEXT NOT NULL REFERENCES payment_methods(id) ON DELETE RESTRICT,
    account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    account_role_code TEXT CHECK (account_role_code IN ('CASH','BANK')),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, payment_method_id),
    CHECK ((account_id IS NOT NULL AND account_role_code IS NULL)
        OR (account_id IS NULL AND account_role_code IS NOT NULL))
);

CREATE TABLE fiscal_period_events (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    fiscal_period_id TEXT NOT NULL REFERENCES fiscal_periods(id) ON DELETE RESTRICT,
    event_type TEXT NOT NULL CHECK (event_type IN ('CLOSED','REOPENED')),
    reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
    previous_status TEXT NOT NULL CHECK (previous_status IN ('OPEN','CLOSED','LOCKED')),
    new_status TEXT NOT NULL CHECK (new_status IN ('OPEN','CLOSED','LOCKED')),
    occurred_at TEXT NOT NULL,
    occurred_by TEXT
);

ALTER TABLE journal_entries ADD COLUMN request_hash_sha256 TEXT
    CHECK (request_hash_sha256 IS NULL OR (length(request_hash_sha256) = 64
        AND request_hash_sha256 NOT GLOB '*[^0-9A-Fa-f]*'));
ALTER TABLE journal_entries ADD COLUMN posting_rule_id TEXT REFERENCES posting_rules(id) ON DELETE RESTRICT;
ALTER TABLE journal_entries ADD COLUMN reversal_reason TEXT;
ALTER TABLE posting_attempts ADD COLUMN request_hash_sha256 TEXT
    CHECK (request_hash_sha256 IS NULL OR (length(request_hash_sha256) = 64
        AND request_hash_sha256 NOT GLOB '*[^0-9A-Fa-f]*'));
ALTER TABLE posting_attempts ADD COLUMN recorded_at TEXT;
ALTER TABLE payments ADD COLUMN journal_entry_id TEXT REFERENCES journal_entries(id) ON DELETE RESTRICT;
ALTER TABLE payments ADD COLUMN reversal_of_payment_id TEXT REFERENCES payments(id) ON DELETE RESTRICT;
ALTER TABLE payments ADD COLUMN request_hash_sha256 TEXT
    CHECK (request_hash_sha256 IS NULL OR (length(request_hash_sha256) = 64
        AND request_hash_sha256 NOT GLOB '*[^0-9A-Fa-f]*'));
ALTER TABLE payment_allocations ADD COLUMN idempotency_key TEXT;
ALTER TABLE payment_allocations ADD COLUMN request_hash_sha256 TEXT
    CHECK (request_hash_sha256 IS NULL OR (length(request_hash_sha256) = 64
        AND request_hash_sha256 NOT GLOB '*[^0-9A-Fa-f]*'));

CREATE UNIQUE INDEX uq_journal_source_event
    ON journal_entries(company_id, source_event_type, source_event_id)
    WHERE status IN ('POSTED','REVERSED');
CREATE INDEX ix_journal_entries_date_accounting
    ON journal_entries(company_id, entry_date, accounting_journal_id);
CREATE INDEX ix_journal_lines_account
    ON journal_entry_lines(company_id, account_id, journal_entry_id);
CREATE INDEX ix_posting_rules_event_active
    ON posting_rules(company_id, source_event_type, priority DESC, valid_from)
    WHERE is_active = 1;
CREATE UNIQUE INDEX uq_posting_attempt_number
    ON posting_attempts(company_id, source_event_type, source_event_id, attempt_number);
CREATE INDEX ix_posting_attempts_source
    ON posting_attempts(company_id, source_event_type, source_event_id, attempt_number);
CREATE INDEX ix_payments_partner_date
    ON payments(company_id, partner_id, commercial_date);
CREATE UNIQUE INDEX uq_payment_reversal
    ON payments(company_id, reversal_of_payment_id)
    WHERE reversal_of_payment_id IS NOT NULL;
CREATE UNIQUE INDEX uq_allocation_idempotency
    ON payment_allocations(company_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
CREATE INDEX ix_allocations_payment
    ON payment_allocations(company_id, payment_id, allocated_at);
CREATE INDEX ix_allocations_document
    ON payment_allocations(company_id, document_id, allocated_at);
CREATE UNIQUE INDEX uq_allocation_reversal
    ON payment_allocations(company_id, reversal_of_allocation_id)
    WHERE reversal_of_allocation_id IS NOT NULL;
CREATE INDEX ix_period_events_period
    ON fiscal_period_events(company_id, fiscal_period_id, occurred_at);


CREATE TRIGGER trg_accounting_setup_company_insert
BEFORE INSERT ON accounting_setups
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NEW.current_fiscal_year_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM fiscal_years fy WHERE fy.id=NEW.current_fiscal_year_id AND fy.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'ACCOUNTING_SETUP_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.retained_earnings_account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.retained_earnings_account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'ACCOUNTING_SETUP_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_payment_method_accounting_company_insert
BEFORE INSERT ON payment_method_accounting
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM payment_methods pm WHERE pm.id=NEW.payment_method_id AND pm.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_METHOD_ACCOUNTING_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_METHOD_ACCOUNTING_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_journal_entry_company_insert
BEFORE INSERT ON journal_entries
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM fiscal_years fy WHERE fy.id=NEW.fiscal_year_id AND fy.company_id=NEW.company_id
    ) OR NOT EXISTS (
        SELECT 1 FROM fiscal_periods fp WHERE fp.id=NEW.fiscal_period_id AND fp.company_id=NEW.company_id AND fp.fiscal_year_id=NEW.fiscal_year_id
    ) OR NOT EXISTS (
        SELECT 1 FROM accounting_journals j WHERE j.id=NEW.accounting_journal_id AND j.company_id=NEW.company_id
    ) OR (NEW.source_document_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM commercial_documents d WHERE d.id=NEW.source_document_id AND d.company_id=NEW.company_id
    )) OR (NEW.reversal_of_entry_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM journal_entries source WHERE source.id=NEW.reversal_of_entry_id AND source.company_id=NEW.company_id
    )) OR (NEW.posting_rule_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM posting_rules rule WHERE rule.id=NEW.posting_rule_id AND rule.company_id=NEW.company_id
    )) THEN RAISE(ABORT, 'JOURNAL_ENTRY_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_journal_line_company_insert
BEFORE INSERT ON journal_entry_lines
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM journal_entries e WHERE e.id=NEW.journal_entry_id AND e.company_id=NEW.company_id
    ) OR NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.account_id AND a.company_id=NEW.company_id
    ) OR (NEW.partner_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM partners partner WHERE partner.id=NEW.partner_id AND partner.company_id=NEW.company_id
    )) OR (NEW.product_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM products product WHERE product.id=NEW.product_id AND product.company_id=NEW.company_id
    )) THEN RAISE(ABORT, 'JOURNAL_LINE_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_payment_accounting_company_insert
BEFORE INSERT ON payments
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM fiscal_years fy WHERE fy.id=NEW.fiscal_year_id AND fy.company_id=NEW.company_id
    ) OR (NEW.fiscal_period_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM fiscal_periods fp WHERE fp.id=NEW.fiscal_period_id AND fp.company_id=NEW.company_id
          AND fp.fiscal_year_id=NEW.fiscal_year_id
    )) OR NOT EXISTS (
        SELECT 1 FROM partners partner WHERE partner.id=NEW.partner_id AND partner.company_id=NEW.company_id
    ) OR NOT EXISTS (
        SELECT 1 FROM payment_methods method WHERE method.id=NEW.payment_method_id AND method.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.journal_entry_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM journal_entries e WHERE e.id=NEW.journal_entry_id AND e.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_JOURNAL_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.reversal_of_payment_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM payments p WHERE p.id=NEW.reversal_of_payment_id AND p.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_REVERSAL_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.status <> 'DRAFT' AND (
        NEW.journal_entry_id IS NULL OR NEW.posting_date IS NULL OR NEW.request_hash_sha256 IS NULL
    ) THEN RAISE(ABORT, 'POSTED_PAYMENT_ACCOUNTING_REQUIRED') END;
END;

CREATE TRIGGER trg_account_role_company_insert
BEFORE INSERT ON accounting_account_roles
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'ACCOUNT_ROLE_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_rule_line_company_insert
BEFORE INSERT ON posting_rule_lines
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM posting_rules r WHERE r.id=NEW.posting_rule_id AND r.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'POSTING_RULE_LINE_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'POSTING_RULE_ACCOUNT_COMPANY_SCOPE') END;
END;











CREATE TRIGGER trg_posting_attempts_append_only_update
BEFORE UPDATE ON posting_attempts
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'POSTING_ATTEMPT_APPEND_ONLY');
END;

CREATE TRIGGER trg_posting_attempts_append_only_delete
BEFORE DELETE ON posting_attempts
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'POSTING_ATTEMPT_APPEND_ONLY');
END;

CREATE TRIGGER trg_payment_history_immutable_update
BEFORE UPDATE ON payments
FOR EACH ROW WHEN OLD.status <> 'DRAFT'
BEGIN
    SELECT RAISE(ABORT, 'POSTED_PAYMENT_IMMUTABLE');
END;

CREATE TRIGGER trg_payment_history_immutable_delete
BEFORE DELETE ON payments
FOR EACH ROW WHEN OLD.status <> 'DRAFT'
BEGIN
    SELECT RAISE(ABORT, 'POSTED_PAYMENT_IMMUTABLE');
END;

CREATE TRIGGER trg_allocation_append_only_update
BEFORE UPDATE ON payment_allocations
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'PAYMENT_ALLOCATION_APPEND_ONLY');
END;

CREATE TRIGGER trg_allocation_append_only_delete
BEFORE DELETE ON payment_allocations
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'PAYMENT_ALLOCATION_APPEND_ONLY');
END;

CREATE TRIGGER trg_allocation_company_scope_insert
BEFORE INSERT ON payment_allocations
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM payments p WHERE p.id=NEW.payment_id AND p.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_ALLOCATION_COMPANY_SCOPE') END;
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM commercial_documents d WHERE d.id=NEW.document_id AND d.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_ALLOCATION_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.reversal_of_allocation_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM payment_allocations original WHERE original.id=NEW.reversal_of_allocation_id
          AND original.company_id=NEW.company_id AND original.payment_id=NEW.payment_id
          AND original.document_id=NEW.document_id
    ) THEN RAISE(ABORT, 'PAYMENT_ALLOCATION_REVERSAL_SCOPE') END;
END;

CREATE TRIGGER trg_accounting_setup_company_update
BEFORE UPDATE OF retained_earnings_account_id,current_fiscal_year_id ON accounting_setups
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NEW.current_fiscal_year_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM fiscal_years fy WHERE fy.id=NEW.current_fiscal_year_id AND fy.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'ACCOUNTING_SETUP_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.retained_earnings_account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.retained_earnings_account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'ACCOUNTING_SETUP_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_payment_method_accounting_company_update
BEFORE UPDATE OF payment_method_id,account_id,account_role_code ON payment_method_accounting
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM payment_methods pm WHERE pm.id=NEW.payment_method_id AND pm.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_METHOD_ACCOUNTING_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'PAYMENT_METHOD_ACCOUNTING_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_account_role_company_update
BEFORE UPDATE OF account_id,role_code ON accounting_account_roles
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'ACCOUNT_ROLE_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_rule_line_company_update
BEFORE UPDATE OF posting_rule_id,account_id,account_role_code ON posting_rule_lines
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM posting_rules r WHERE r.id=NEW.posting_rule_id AND r.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'POSTING_RULE_LINE_COMPANY_SCOPE') END;
    SELECT CASE WHEN NEW.account_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM accounts a WHERE a.id=NEW.account_id AND a.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'POSTING_RULE_ACCOUNT_COMPANY_SCOPE') END;
END;


CREATE TRIGGER trg_posting_attempt_company_insert
BEFORE INSERT ON posting_attempts
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NEW.result_entry_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM journal_entries entry WHERE entry.id=NEW.result_entry_id AND entry.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'POSTING_ATTEMPT_RESULT_SCOPE') END;
    SELECT CASE WHEN NEW.retry_of_attempt_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM posting_attempts attempt WHERE attempt.id=NEW.retry_of_attempt_id
          AND attempt.company_id=NEW.company_id
          AND attempt.source_event_type=NEW.source_event_type
          AND attempt.source_event_id=NEW.source_event_id
    ) THEN RAISE(ABORT, 'POSTING_ATTEMPT_RETRY_SCOPE') END;
END;

CREATE TRIGGER trg_fiscal_period_event_company_insert
BEFORE INSERT ON fiscal_period_events
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM fiscal_periods period WHERE period.id=NEW.fiscal_period_id
          AND period.company_id=NEW.company_id
    ) THEN RAISE(ABORT, 'FISCAL_PERIOD_EVENT_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_journal_entry_company_update
BEFORE UPDATE OF fiscal_year_id,fiscal_period_id,accounting_journal_id,source_document_id,reversal_of_entry_id,posting_rule_id
ON journal_entries
FOR EACH ROW WHEN OLD.status='DRAFT'
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM fiscal_years fy WHERE fy.id=NEW.fiscal_year_id AND fy.company_id=NEW.company_id
    ) OR NOT EXISTS (
        SELECT 1 FROM fiscal_periods fp WHERE fp.id=NEW.fiscal_period_id AND fp.company_id=NEW.company_id
          AND fp.fiscal_year_id=NEW.fiscal_year_id
    ) OR NOT EXISTS (
        SELECT 1 FROM accounting_journals journal WHERE journal.id=NEW.accounting_journal_id
          AND journal.company_id=NEW.company_id
    ) OR (NEW.source_document_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM commercial_documents document WHERE document.id=NEW.source_document_id
          AND document.company_id=NEW.company_id
    )) OR (NEW.reversal_of_entry_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM journal_entries source WHERE source.id=NEW.reversal_of_entry_id
          AND source.company_id=NEW.company_id
    )) OR (NEW.posting_rule_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM posting_rules rule WHERE rule.id=NEW.posting_rule_id
          AND rule.company_id=NEW.company_id
    )) THEN RAISE(ABORT, 'JOURNAL_ENTRY_COMPANY_SCOPE') END;
END;

CREATE TRIGGER trg_journal_line_company_update
BEFORE UPDATE OF journal_entry_id,account_id,partner_id,product_id ON journal_entry_lines
FOR EACH ROW
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM journal_entries entry WHERE entry.id=NEW.journal_entry_id
          AND entry.company_id=NEW.company_id
    ) OR NOT EXISTS (
        SELECT 1 FROM accounts account WHERE account.id=NEW.account_id
          AND account.company_id=NEW.company_id
    ) OR (NEW.partner_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM partners partner WHERE partner.id=NEW.partner_id
          AND partner.company_id=NEW.company_id
    )) OR (NEW.product_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM products product WHERE product.id=NEW.product_id
          AND product.company_id=NEW.company_id
    )) THEN RAISE(ABORT, 'JOURNAL_LINE_COMPANY_SCOPE') END;
END;
