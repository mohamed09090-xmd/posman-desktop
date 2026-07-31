-- POSMAN PHASE 05 - setup, local security, and reference-data extensions.
-- This migration is additive to the accepted 0001-0004 schema and remains unmerged.

ALTER TABLE companies ADD COLUMN activity_description TEXT
    CHECK (activity_description IS NULL OR length(trim(activity_description)) > 0);
ALTER TABLE companies ADD COLUMN legal_form TEXT
    CHECK (legal_form IS NULL OR length(trim(legal_form)) > 0);
ALTER TABLE companies ADD COLUMN social_capital_minor INTEGER
    CHECK (social_capital_minor IS NULL OR social_capital_minor >= 0);
ALTER TABLE companies ADD COLUMN statistical_identifier TEXT
    CHECK (statistical_identifier IS NULL OR length(trim(statistical_identifier)) > 0);
ALTER TABLE companies ADD COLUMN tax_article_number TEXT
    CHECK (tax_article_number IS NULL OR length(trim(tax_article_number)) > 0);
ALTER TABLE companies ADD COLUMN bank_rib TEXT
    CHECK (bank_rib IS NULL OR length(trim(bank_rib)) > 0);
ALTER TABLE companies ADD COLUMN wilaya_code TEXT
    CHECK (
        wilaya_code IS NULL
        OR (
            length(wilaya_code) = 2
            AND wilaya_code GLOB '[0-9][0-9]'
            AND CAST(wilaya_code AS INTEGER) BETWEEN 1 AND 58
        )
    );
ALTER TABLE companies ADD COLUMN city TEXT
    CHECK (city IS NULL OR length(trim(city)) > 0);
ALTER TABLE companies ADD COLUMN postal_code TEXT
    CHECK (
        postal_code IS NULL
        OR (
            length(postal_code) BETWEEN 4 AND 10
            AND postal_code NOT GLOB '*[^0-9]*'
        )
    );

ALTER TABLE partners ADD COLUMN legal_form TEXT
    CHECK (legal_form IS NULL OR length(trim(legal_form)) > 0);
ALTER TABLE partners ADD COLUMN activity_description TEXT
    CHECK (activity_description IS NULL OR length(trim(activity_description)) > 0);
ALTER TABLE partners ADD COLUMN statistical_identifier TEXT
    CHECK (statistical_identifier IS NULL OR length(trim(statistical_identifier)) > 0);
ALTER TABLE partners ADD COLUMN tax_article_number TEXT
    CHECK (tax_article_number IS NULL OR length(trim(tax_article_number)) > 0);

ALTER TABLE company_settings ADD COLUMN default_margin_rate_scaled INTEGER NOT NULL DEFAULT 0
    CHECK (default_margin_rate_scaled BETWEEN 0 AND 1000000);
ALTER TABLE company_settings ADD COLUMN session_idle_timeout_minutes INTEGER NOT NULL DEFAULT 15
    CHECK (session_idle_timeout_minutes BETWEEN 5 AND 120);
ALTER TABLE company_settings ADD COLUMN default_tax_rate_id TEXT
    REFERENCES tax_rates(id) ON DELETE RESTRICT;

CREATE TABLE setup_drafts (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    singleton_key INTEGER NOT NULL DEFAULT 1 CHECK (singleton_key = 1),
    draft_schema_version INTEGER NOT NULL CHECK (draft_schema_version >= 1),
    validated_json TEXT NOT NULL CHECK (
        json_valid(validated_json)
        AND json_type(validated_json) = 'object'
    ),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE UNIQUE INDEX uq_setup_drafts_singleton_active
    ON setup_drafts(singleton_key)
    WHERE is_active = 1;

CREATE TABLE initial_setup_requests (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    idempotency_key TEXT NOT NULL UNIQUE
        CHECK (length(trim(idempotency_key)) BETWEEN 8 AND 200),
    request_hash_sha256 TEXT NOT NULL CHECK (
        length(request_hash_sha256) = 64
        AND request_hash_sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    status TEXT NOT NULL CHECK (status IN ('IN_PROGRESS', 'SUCCEEDED')),
    result_company_id TEXT REFERENCES companies(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    CHECK (
        (status = 'IN_PROGRESS' AND completed_at IS NULL AND result_company_id IS NULL)
        OR (status = 'SUCCEEDED' AND completed_at IS NOT NULL AND result_company_id IS NOT NULL)
    )
);

CREATE TABLE user_recovery_codes (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL CHECK (
        length(code_hash) = 64
        AND code_hash NOT GLOB '*[^0-9a-f]*'
    ),
    created_at TEXT NOT NULL,
    expires_at TEXT CHECK (expires_at IS NULL OR expires_at > created_at),
    used_at TEXT,
    revoked_at TEXT,
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    CHECK (used_at IS NULL OR used_at >= created_at),
    CHECK (revoked_at IS NULL OR revoked_at >= created_at),
    CHECK (used_at IS NULL OR revoked_at IS NULL),
    UNIQUE (company_id, id)
);

CREATE UNIQUE INDEX uq_user_recovery_codes_active
    ON user_recovery_codes(company_id, user_id)
    WHERE used_at IS NULL AND revoked_at IS NULL;

CREATE INDEX idx_user_recovery_codes_lookup
    ON user_recovery_codes(company_id, user_id, code_hash);

CREATE UNIQUE INDEX uq_users_company_username_normalized
    ON users(company_id, lower(trim(username)));

CREATE UNIQUE INDEX uq_document_sequences_company_year_type
    ON document_sequences(company_id, fiscal_year_id, document_type);
