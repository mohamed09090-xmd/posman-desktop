-- POSMAN Phase 01 - system, company, fiscal, and security foundation.

CREATE TABLE app_migrations (
    id INTEGER PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64),
    applied_at TEXT NOT NULL
);

CREATE TABLE companies (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    code TEXT NOT NULL UNIQUE CHECK (length(trim(code)) > 0),
    legal_name TEXT NOT NULL CHECK (length(trim(legal_name)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    currency_code TEXT NOT NULL DEFAULT 'DZD' CHECK (currency_code = 'DZD'),
    timezone_name TEXT NOT NULL DEFAULT 'Africa/Algiers',
    tax_identifier TEXT,
    trade_register_number TEXT,
    address_text TEXT,
    phone TEXT,
    email TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE TABLE company_settings (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL UNIQUE REFERENCES companies(id) ON DELETE RESTRICT,
    default_language TEXT NOT NULL DEFAULT 'ar' CHECK (default_language IN ('ar', 'fr')),
    price_input_mode TEXT NOT NULL DEFAULT 'HT' CHECK (price_input_mode IN ('HT', 'TTC')),
    margin_method TEXT NOT NULL DEFAULT 'COST_MARKUP' CHECK (margin_method IN ('COST_MARKUP', 'SALE_MARK')),
    inventory_cost_method TEXT NOT NULL DEFAULT 'MOVING_WEIGHTED_AVERAGE' CHECK (inventory_cost_method = 'MOVING_WEIGHTED_AVERAGE'),
    negative_stock_policy TEXT NOT NULL DEFAULT 'BLOCK' CHECK (negative_stock_policy IN ('BLOCK', 'PRIVILEGED_OVERRIDE')),
    preferred_journal_mode TEXT NOT NULL DEFAULT 'WAL' CHECK (preferred_journal_mode IN ('WAL', 'DELETE')),
    automatic_backup_enabled INTEGER NOT NULL DEFAULT 1 CHECK (automatic_backup_enabled IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE TABLE fiscal_years (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    starts_on TEXT NOT NULL CHECK (length(starts_on) = 10),
    ends_on TEXT NOT NULL CHECK (length(ends_on) = 10 AND ends_on >= starts_on),
    status TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'CLOSED', 'LOCKED')),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code),
    UNIQUE (company_id, id)
);

CREATE TABLE fiscal_periods (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    period_number INTEGER NOT NULL CHECK (period_number BETWEEN 1 AND 53),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    starts_on TEXT NOT NULL CHECK (length(starts_on) = 10),
    ends_on TEXT NOT NULL CHECK (length(ends_on) = 10 AND ends_on >= starts_on),
    status TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'CLOSED', 'LOCKED')),
    closed_at TEXT,
    closed_by TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, fiscal_year_id, period_number),
    UNIQUE (company_id, id)
);

CREATE TABLE document_sequences (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    document_type TEXT NOT NULL CHECK (length(trim(document_type)) > 0),
    prefix TEXT NOT NULL DEFAULT '',
    next_number INTEGER NOT NULL DEFAULT 1 CHECK (next_number >= 1),
    padding_width INTEGER NOT NULL DEFAULT 6 CHECK (padding_width BETWEEN 1 AND 12),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, fiscal_year_id, document_type, prefix)
);

CREATE TABLE users (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    username TEXT NOT NULL CHECK (length(trim(username)) > 0),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    password_hash TEXT NOT NULL CHECK (length(password_hash) >= 20),
    preferred_language TEXT NOT NULL DEFAULT 'ar' CHECK (preferred_language IN ('ar', 'fr')),
    failed_login_count INTEGER NOT NULL DEFAULT 0 CHECK (failed_login_count >= 0),
    locked_until TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    last_login_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, username),
    UNIQUE (company_id, id)
);

CREATE TABLE roles (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT NOT NULL CHECK (length(trim(name_fr)) > 0),
    is_system INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE UNIQUE INDEX uq_roles_global_code
    ON roles(code)
    WHERE company_id IS NULL;

CREATE UNIQUE INDEX uq_roles_company_code
    ON roles(company_id, code)
    WHERE company_id IS NOT NULL;

CREATE TABLE permissions (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    code TEXT NOT NULL UNIQUE CHECK (length(trim(code)) > 0),
    domain TEXT NOT NULL CHECK (length(trim(domain)) > 0),
    description_ar TEXT NOT NULL CHECK (length(trim(description_ar)) > 0),
    description_fr TEXT NOT NULL CHECK (length(trim(description_fr)) > 0),
    is_sensitive INTEGER NOT NULL DEFAULT 0 CHECK (is_sensitive IN (0, 1)),
    created_at TEXT NOT NULL
);

CREATE TABLE user_roles (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    assigned_at TEXT NOT NULL,
    assigned_by TEXT,
    UNIQUE (company_id, user_id, role_id)
);

CREATE TABLE role_permissions (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT REFERENCES companies(id) ON DELETE RESTRICT,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    granted_at TEXT NOT NULL,
    granted_by TEXT,
    UNIQUE (role_id, permission_id)
);

CREATE TABLE sessions (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE CHECK (length(token_hash) >= 32),
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    revoked_at TEXT,
    CHECK (expires_at > created_at)
);

CREATE INDEX idx_fiscal_periods_company_dates ON fiscal_periods(company_id, starts_on, ends_on);
CREATE INDEX idx_users_company_active ON users(company_id, is_active);
CREATE INDEX idx_sessions_user_active ON sessions(user_id, revoked_at, expires_at);
