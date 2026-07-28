-- GENERATED FILE: ordered migrations are authoritative.
-- Regenerate with: python scripts/verify_schema.py --write-schema

-- BEGIN MIGRATION 0001_system_company_security.sql
-- POSMAN Phase 01 - system, company, fiscal, and security foundation.

CREATE TABLE app_migrations (
    id INTEGER PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64),
    applied_at TEXT NOT NULL
);

CREATE TABLE companies (
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE CHECK (length(trim(code)) > 0),
    domain TEXT NOT NULL CHECK (length(trim(domain)) > 0),
    description_ar TEXT NOT NULL CHECK (length(trim(description_ar)) > 0),
    description_fr TEXT NOT NULL CHECK (length(trim(description_fr)) > 0),
    is_sensitive INTEGER NOT NULL DEFAULT 0 CHECK (is_sensitive IN (0, 1)),
    created_at TEXT NOT NULL
);

CREATE TABLE user_roles (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    assigned_at TEXT NOT NULL,
    assigned_by TEXT,
    UNIQUE (company_id, user_id, role_id)
);

CREATE TABLE role_permissions (
    id TEXT PRIMARY KEY,
    company_id TEXT REFERENCES companies(id) ON DELETE RESTRICT,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    granted_at TEXT NOT NULL,
    granted_by TEXT,
    UNIQUE (role_id, permission_id)
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
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
-- END MIGRATION 0001_system_company_security.sql

-- BEGIN MIGRATION 0002_reference_catalog_partners.sql
-- POSMAN Phase 01 - reference data, catalog, warehouses, pricing, and partners.

CREATE TABLE units (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT NOT NULL CHECK (length(trim(name_fr)) > 0),
    decimal_scale INTEGER NOT NULL DEFAULT 0 CHECK (decimal_scale BETWEEN 0 AND 6),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code)
);

CREATE TABLE tax_rates (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT NOT NULL CHECK (length(trim(name_fr)) > 0),
    rate_scaled INTEGER NOT NULL CHECK (rate_scaled BETWEEN 0 AND 1000000),
    valid_from TEXT NOT NULL CHECK (length(valid_from) = 10),
    valid_to TEXT CHECK (valid_to IS NULL OR (length(valid_to) = 10 AND valid_to >= valid_from)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code, valid_from)
);

CREATE TABLE payment_terms (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT NOT NULL CHECK (length(trim(name_fr)) > 0),
    due_days INTEGER NOT NULL DEFAULT 0 CHECK (due_days BETWEEN 0 AND 3650),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code)
);

CREATE TABLE payment_methods (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT NOT NULL CHECK (length(trim(name_fr)) > 0),
    method_kind TEXT NOT NULL CHECK (method_kind IN ('CASH', 'CARD', 'CHEQUE', 'BANK_TRANSFER', 'OTHER')),
    reference_required INTEGER NOT NULL DEFAULT 0 CHECK (reference_required IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code)
);

CREATE TABLE warehouses (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    address_text TEXT,
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code)
);

CREATE UNIQUE INDEX uq_warehouses_one_default
    ON warehouses(company_id)
    WHERE is_default = 1 AND is_active = 1;

CREATE TABLE warehouse_locations (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    warehouse_id TEXT NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, warehouse_id, code)
);

CREATE TABLE product_families (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    parent_family_id TEXT REFERENCES product_families(id) ON DELETE RESTRICT,
    default_tax_rate_id TEXT REFERENCES tax_rates(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    default_margin_rate_scaled INTEGER CHECK (default_margin_rate_scaled IS NULL OR default_margin_rate_scaled BETWEEN 0 AND 1000000),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code),
    CHECK (parent_family_id IS NULL OR parent_family_id <> id)
);

CREATE TABLE products (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    product_family_id TEXT REFERENCES product_families(id) ON DELETE RESTRICT,
    unit_id TEXT NOT NULL REFERENCES units(id) ON DELETE RESTRICT,
    default_tax_rate_id TEXT REFERENCES tax_rates(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    barcode TEXT,
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    product_kind TEXT NOT NULL DEFAULT 'STOCK_ITEM' CHECK (product_kind IN ('STOCK_ITEM', 'SERVICE', 'NON_STOCK_ITEM')),
    stock_tracked INTEGER NOT NULL DEFAULT 1 CHECK (stock_tracked IN (0, 1)),
    minimum_stock_scaled INTEGER NOT NULL DEFAULT 0 CHECK (minimum_stock_scaled >= 0),
    default_purchase_price_scaled INTEGER CHECK (default_purchase_price_scaled IS NULL OR default_purchase_price_scaled >= 0),
    default_sale_price_scaled INTEGER CHECK (default_sale_price_scaled IS NULL OR default_sale_price_scaled >= 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code),
    CHECK ((product_kind = 'STOCK_ITEM' AND stock_tracked = 1) OR product_kind <> 'STOCK_ITEM')
);

CREATE UNIQUE INDEX uq_products_company_barcode
    ON products(company_id, barcode)
    WHERE barcode IS NOT NULL AND length(trim(barcode)) > 0;

CREATE TABLE price_lists (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    name_ar TEXT NOT NULL CHECK (length(trim(name_ar)) > 0),
    name_fr TEXT,
    price_mode TEXT NOT NULL DEFAULT 'HT' CHECK (price_mode IN ('HT', 'TTC')),
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code)
);

CREATE UNIQUE INDEX uq_price_lists_one_default
    ON price_lists(company_id)
    WHERE is_default = 1 AND is_active = 1;

CREATE TABLE product_prices (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    price_list_id TEXT NOT NULL REFERENCES price_lists(id) ON DELETE RESTRICT,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    unit_price_scaled INTEGER NOT NULL CHECK (unit_price_scaled >= 0),
    valid_from TEXT NOT NULL CHECK (length(valid_from) = 10),
    valid_to TEXT CHECK (valid_to IS NULL OR (length(valid_to) = 10 AND valid_to >= valid_from)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, price_list_id, product_id, valid_from)
);

CREATE TABLE partners (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    payment_term_id TEXT REFERENCES payment_terms(id) ON DELETE RESTRICT,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    legal_name TEXT NOT NULL CHECK (length(trim(legal_name)) > 0),
    display_name_ar TEXT NOT NULL CHECK (length(trim(display_name_ar)) > 0),
    display_name_fr TEXT,
    is_customer INTEGER NOT NULL DEFAULT 0 CHECK (is_customer IN (0, 1)),
    is_supplier INTEGER NOT NULL DEFAULT 0 CHECK (is_supplier IN (0, 1)),
    tax_identifier TEXT,
    trade_register_number TEXT,
    credit_limit_minor INTEGER NOT NULL DEFAULT 0 CHECK (credit_limit_minor >= 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, code),
    CHECK (is_customer = 1 OR is_supplier = 1)
);

CREATE TABLE partner_addresses (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    partner_id TEXT NOT NULL REFERENCES partners(id) ON DELETE CASCADE,
    address_kind TEXT NOT NULL CHECK (address_kind IN ('BILLING', 'DELIVERY', 'BOTH')),
    label TEXT,
    address_line_1 TEXT NOT NULL CHECK (length(trim(address_line_1)) > 0),
    address_line_2 TEXT,
    city TEXT,
    province TEXT,
    postal_code TEXT,
    country_code TEXT NOT NULL DEFAULT 'DZ' CHECK (country_code = 'DZ'),
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE UNIQUE INDEX uq_partner_addresses_default
    ON partner_addresses(company_id, partner_id, address_kind)
    WHERE is_default = 1 AND is_active = 1;

CREATE TABLE partner_contacts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    partner_id TEXT NOT NULL REFERENCES partners(id) ON DELETE CASCADE,
    full_name TEXT NOT NULL CHECK (length(trim(full_name)) > 0),
    job_title TEXT,
    phone TEXT,
    email TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0 CHECK (is_primary IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    CHECK ((phone IS NOT NULL AND length(trim(phone)) > 0) OR (email IS NOT NULL AND length(trim(email)) > 0))
);

CREATE UNIQUE INDEX uq_partner_contacts_primary
    ON partner_contacts(company_id, partner_id)
    WHERE is_primary = 1 AND is_active = 1;

CREATE INDEX idx_tax_rates_company_validity ON tax_rates(company_id, valid_from, valid_to);
CREATE INDEX idx_products_company_active ON products(company_id, is_active);
CREATE INDEX idx_product_prices_lookup ON product_prices(company_id, product_id, valid_from, valid_to);
CREATE INDEX idx_partners_company_kind ON partners(company_id, is_customer, is_supplier, is_active);
-- END MIGRATION 0002_reference_catalog_partners.sql

-- BEGIN MIGRATION 0003_commerce_inventory.sql
-- POSMAN Phase 01 - commercial documents, conversion lineage, payments, and inventory.

CREATE TABLE commercial_documents (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    fiscal_period_id TEXT REFERENCES fiscal_periods(id) ON DELETE RESTRICT,
    partner_id TEXT REFERENCES partners(id) ON DELETE RESTRICT,
    warehouse_id TEXT REFERENCES warehouses(id) ON DELETE RESTRICT,
    source_document_id TEXT REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    document_type TEXT NOT NULL CHECK (document_type IN (
        'SALES_ORDER', 'DELIVERY_NOTE', 'SALES_INVOICE', 'SALES_RETURN', 'SALES_CREDIT_NOTE',
        'PURCHASE_REQUEST', 'PURCHASE_ORDER', 'PURCHASE_RECEIPT', 'PURCHASE_INVOICE',
        'PURCHASE_RETURN', 'PURCHASE_CREDIT_NOTE', 'OPENING_STOCK', 'STOCK_ADJUSTMENT',
        'STOCK_TRANSFER', 'INVENTORY_COUNT'
    )),
    document_number TEXT NOT NULL CHECK (length(trim(document_number)) > 0),
    workflow_status TEXT NOT NULL,
    posting_status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (posting_status IN ('DRAFT', 'POSTED', 'REVERSED', 'FAILED')),
    commercial_date TEXT NOT NULL CHECK (length(commercial_date) = 10),
    posting_date TEXT CHECK (posting_date IS NULL OR length(posting_date) = 10),
    due_date TEXT CHECK (due_date IS NULL OR length(due_date) = 10),
    currency_code TEXT NOT NULL DEFAULT 'DZD' CHECK (currency_code = 'DZD'),
    price_mode TEXT NOT NULL DEFAULT 'HT' CHECK (price_mode IN ('HT', 'TTC')),
    header_discount_rate_scaled INTEGER NOT NULL DEFAULT 0 CHECK (header_discount_rate_scaled BETWEEN 0 AND 1000000),
    header_discount_minor INTEGER NOT NULL DEFAULT 0 CHECK (header_discount_minor >= 0),
    total_ht_minor INTEGER NOT NULL DEFAULT 0 CHECK (total_ht_minor >= 0),
    total_tax_minor INTEGER NOT NULL DEFAULT 0 CHECK (total_tax_minor >= 0),
    total_ttc_minor INTEGER NOT NULL DEFAULT 0 CHECK (total_ttc_minor >= 0),
    notes TEXT,
    idempotency_key TEXT,
    posted_at TEXT,
    posted_by TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, fiscal_year_id, document_type, document_number),
    UNIQUE (company_id, idempotency_key),
    CHECK (source_document_id IS NULL OR source_document_id <> id),
    CHECK (
        (document_type = 'SALES_ORDER' AND workflow_status IN ('DRAFT', 'CONFIRMED', 'PARTIALLY_DELIVERED', 'DELIVERED', 'CLOSED', 'CANCELLED', 'ON_HOLD'))
        OR (document_type = 'DELIVERY_NOTE' AND workflow_status IN ('DRAFT', 'RESERVED', 'POSTED', 'PARTIALLY_INVOICED', 'INVOICED', 'REVERSED', 'CANCELLED'))
        OR (document_type = 'SALES_INVOICE' AND workflow_status IN ('DRAFT', 'VALIDATED', 'POSTED', 'PARTIALLY_PAID', 'PAID', 'CREDITED', 'REVERSED', 'CANCELLED'))
        OR (document_type = 'PURCHASE_RECEIPT' AND workflow_status IN ('DRAFT', 'POSTED', 'PARTIALLY_INVOICED', 'INVOICED', 'REVERSED', 'CANCELLED'))
        OR (document_type = 'PURCHASE_INVOICE' AND workflow_status IN ('DRAFT', 'VALIDATED', 'POSTED', 'PARTIALLY_PAID', 'PAID', 'CREDITED', 'REVERSED', 'CANCELLED'))
        OR (document_type IN ('PURCHASE_REQUEST', 'PURCHASE_ORDER') AND workflow_status IN ('DRAFT', 'CONFIRMED', 'CLOSED', 'CANCELLED', 'ON_HOLD'))
        OR (document_type IN ('SALES_RETURN', 'SALES_CREDIT_NOTE', 'PURCHASE_RETURN', 'PURCHASE_CREDIT_NOTE') AND workflow_status IN ('DRAFT', 'VALIDATED', 'POSTED', 'REVERSED', 'CANCELLED'))
        OR (document_type IN ('OPENING_STOCK', 'STOCK_ADJUSTMENT', 'STOCK_TRANSFER', 'INVENTORY_COUNT') AND workflow_status IN ('DRAFT', 'POSTED', 'REVERSED', 'CANCELLED'))
    )
);

CREATE TABLE commercial_document_lines (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_id TEXT NOT NULL REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    warehouse_id TEXT REFERENCES warehouses(id) ON DELETE RESTRICT,
    unit_id TEXT NOT NULL REFERENCES units(id) ON DELETE RESTRICT,
    line_number INTEGER NOT NULL CHECK (line_number >= 1),
    product_code_snapshot TEXT NOT NULL CHECK (length(trim(product_code_snapshot)) > 0),
    description_snapshot TEXT NOT NULL CHECK (length(trim(description_snapshot)) > 0),
    unit_code_snapshot TEXT NOT NULL CHECK (length(trim(unit_code_snapshot)) > 0),
    tax_code_snapshot TEXT,
    quantity_scaled INTEGER NOT NULL CHECK (quantity_scaled > 0),
    unit_price_scaled INTEGER NOT NULL DEFAULT 0 CHECK (unit_price_scaled >= 0),
    unit_cost_scaled INTEGER CHECK (unit_cost_scaled IS NULL OR unit_cost_scaled >= 0),
    line_discount_rate_scaled INTEGER NOT NULL DEFAULT 0 CHECK (line_discount_rate_scaled BETWEEN 0 AND 1000000),
    line_discount_minor INTEGER NOT NULL DEFAULT 0 CHECK (line_discount_minor >= 0),
    allocated_header_discount_minor INTEGER NOT NULL DEFAULT 0 CHECK (allocated_header_discount_minor >= 0),
    tax_rate_scaled INTEGER NOT NULL DEFAULT 0 CHECK (tax_rate_scaled BETWEEN 0 AND 1000000),
    line_ht_minor INTEGER NOT NULL DEFAULT 0 CHECK (line_ht_minor >= 0),
    line_tax_minor INTEGER NOT NULL DEFAULT 0 CHECK (line_tax_minor >= 0),
    line_ttc_minor INTEGER NOT NULL DEFAULT 0 CHECK (line_ttc_minor >= 0),
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (document_id, line_number)
);

CREATE TABLE document_line_links (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    source_line_id TEXT NOT NULL REFERENCES commercial_document_lines(id) ON DELETE RESTRICT,
    target_line_id TEXT NOT NULL REFERENCES commercial_document_lines(id) ON DELETE RESTRICT,
    transformation_type TEXT NOT NULL CHECK (transformation_type IN (
        'ORDER_TO_DELIVERY', 'ORDER_TO_INVOICE', 'DELIVERY_TO_INVOICE',
        'PURCHASE_ORDER_TO_RECEIPT', 'PURCHASE_ORDER_TO_INVOICE', 'RECEIPT_TO_INVOICE',
        'DOCUMENT_TO_RETURN', 'DOCUMENT_TO_CREDIT'
    )),
    transformed_quantity_scaled INTEGER NOT NULL CHECK (transformed_quantity_scaled > 0),
    created_at TEXT NOT NULL,
    created_by TEXT,
    UNIQUE (source_line_id, target_line_id),
    CHECK (source_line_id <> target_line_id)
);

CREATE TABLE document_status_history (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    document_id TEXT NOT NULL REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    old_status TEXT,
    new_status TEXT NOT NULL CHECK (length(trim(new_status)) > 0),
    reason TEXT,
    row_version_snapshot INTEGER NOT NULL CHECK (row_version_snapshot >= 1),
    changed_at TEXT NOT NULL,
    changed_by TEXT
);

CREATE TABLE payments (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    fiscal_period_id TEXT REFERENCES fiscal_periods(id) ON DELETE RESTRICT,
    partner_id TEXT NOT NULL REFERENCES partners(id) ON DELETE RESTRICT,
    payment_method_id TEXT NOT NULL REFERENCES payment_methods(id) ON DELETE RESTRICT,
    payment_number TEXT NOT NULL CHECK (length(trim(payment_number)) > 0),
    payment_kind TEXT NOT NULL CHECK (payment_kind IN ('RECEIPT', 'DISBURSEMENT')),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'POSTED', 'PARTIALLY_ALLOCATED', 'ALLOCATED', 'REVERSED', 'CANCELLED')),
    commercial_date TEXT NOT NULL CHECK (length(commercial_date) = 10),
    posting_date TEXT CHECK (posting_date IS NULL OR length(posting_date) = 10),
    amount_minor INTEGER NOT NULL CHECK (amount_minor > 0),
    currency_code TEXT NOT NULL DEFAULT 'DZD' CHECK (currency_code = 'DZD'),
    external_reference TEXT,
    idempotency_key TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, fiscal_year_id, payment_kind, payment_number),
    UNIQUE (company_id, idempotency_key)
);

CREATE TABLE payment_allocations (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    payment_id TEXT NOT NULL REFERENCES payments(id) ON DELETE RESTRICT,
    document_id TEXT NOT NULL REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    reversal_of_allocation_id TEXT REFERENCES payment_allocations(id) ON DELETE RESTRICT,
    allocated_amount_minor INTEGER NOT NULL CHECK (allocated_amount_minor > 0),
    allocation_status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (allocation_status IN ('ACTIVE', 'REVERSED')),
    allocated_at TEXT NOT NULL,
    allocated_by TEXT,
    CHECK (reversal_of_allocation_id IS NULL OR reversal_of_allocation_id <> id)
);

CREATE TABLE stock_movements (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    warehouse_id TEXT NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    warehouse_location_id TEXT REFERENCES warehouse_locations(id) ON DELETE RESTRICT,
    source_document_id TEXT REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    source_line_id TEXT REFERENCES commercial_document_lines(id) ON DELETE RESTRICT,
    reversal_of_movement_id TEXT REFERENCES stock_movements(id) ON DELETE RESTRICT,
    movement_type TEXT NOT NULL CHECK (movement_type IN (
        'OPENING', 'PURCHASE_RECEIPT', 'SALES_DELIVERY', 'SALES_RETURN', 'PURCHASE_RETURN',
        'TRANSFER_OUT', 'TRANSFER_IN', 'ADJUSTMENT_IN', 'ADJUSTMENT_OUT', 'COUNT_VARIANCE'
    )),
    business_date TEXT NOT NULL CHECK (length(business_date) = 10),
    occurred_at TEXT NOT NULL,
    quantity_delta_scaled INTEGER NOT NULL CHECK (quantity_delta_scaled <> 0),
    quantity_before_scaled INTEGER NOT NULL,
    quantity_after_scaled INTEGER NOT NULL,
    unit_cost_scaled INTEGER CHECK (unit_cost_scaled IS NULL OR unit_cost_scaled >= 0),
    average_cost_before_scaled INTEGER CHECK (average_cost_before_scaled IS NULL OR average_cost_before_scaled >= 0),
    average_cost_after_scaled INTEGER CHECK (average_cost_after_scaled IS NULL OR average_cost_after_scaled >= 0),
    extended_cost_minor INTEGER CHECK (extended_cost_minor IS NULL OR extended_cost_minor >= 0),
    posting_event_key TEXT NOT NULL CHECK (length(trim(posting_event_key)) > 0),
    transfer_group_id TEXT,
    notes TEXT,
    created_by TEXT,
    UNIQUE (company_id, posting_event_key),
    CHECK (quantity_after_scaled = quantity_before_scaled + quantity_delta_scaled),
    CHECK (
        (movement_type IN ('TRANSFER_OUT', 'TRANSFER_IN') AND transfer_group_id IS NOT NULL)
        OR (movement_type NOT IN ('TRANSFER_OUT', 'TRANSFER_IN'))
    ),
    CHECK (reversal_of_movement_id IS NULL OR reversal_of_movement_id <> id)
);

CREATE TABLE stock_balances (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    warehouse_id TEXT NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    warehouse_location_id TEXT REFERENCES warehouse_locations(id) ON DELETE RESTRICT,
    last_movement_id TEXT REFERENCES stock_movements(id) ON DELETE RESTRICT,
    on_hand_scaled INTEGER NOT NULL DEFAULT 0,
    reserved_scaled INTEGER NOT NULL DEFAULT 0 CHECK (reserved_scaled >= 0),
    available_scaled INTEGER NOT NULL DEFAULT 0,
    average_cost_scaled INTEGER NOT NULL DEFAULT 0 CHECK (average_cost_scaled >= 0),
    rebuilt_at TEXT NOT NULL,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    CHECK (available_scaled = on_hand_scaled - reserved_scaled)
);

CREATE UNIQUE INDEX uq_stock_balances_scope
    ON stock_balances(company_id, product_id, warehouse_id, ifnull(warehouse_location_id, ''));

CREATE TABLE stock_reservations (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    warehouse_id TEXT NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    warehouse_location_id TEXT REFERENCES warehouse_locations(id) ON DELETE RESTRICT,
    source_line_id TEXT NOT NULL REFERENCES commercial_document_lines(id) ON DELETE RESTRICT,
    reserved_quantity_scaled INTEGER NOT NULL CHECK (reserved_quantity_scaled > 0),
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'PARTIALLY_CONSUMED', 'CONSUMED', 'RELEASED', 'CANCELLED')),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);

CREATE TABLE inventory_counts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    warehouse_id TEXT NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    adjustment_document_id TEXT REFERENCES commercial_documents(id) ON DELETE RESTRICT,
    count_number TEXT NOT NULL CHECK (length(trim(count_number)) > 0),
    commercial_date TEXT NOT NULL CHECK (length(commercial_date) = 10),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'COUNTING', 'REVIEWED', 'POSTED', 'CANCELLED')),
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    UNIQUE (company_id, warehouse_id, count_number)
);

CREATE TABLE inventory_count_lines (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    inventory_count_id TEXT NOT NULL REFERENCES inventory_counts(id) ON DELETE RESTRICT,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    warehouse_location_id TEXT REFERENCES warehouse_locations(id) ON DELETE RESTRICT,
    system_quantity_scaled INTEGER NOT NULL,
    counted_quantity_scaled INTEGER NOT NULL CHECK (counted_quantity_scaled >= 0),
    variance_quantity_scaled INTEGER NOT NULL,
    unit_cost_scaled INTEGER CHECK (unit_cost_scaled IS NULL OR unit_cost_scaled >= 0),
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    row_version INTEGER NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    CHECK (variance_quantity_scaled = counted_quantity_scaled - system_quantity_scaled)
);

CREATE UNIQUE INDEX uq_inventory_count_lines_scope
    ON inventory_count_lines(inventory_count_id, product_id, ifnull(warehouse_location_id, ''));

CREATE INDEX idx_commercial_documents_lookup ON commercial_documents(company_id, document_type, commercial_date, workflow_status);
CREATE INDEX idx_commercial_lines_document ON commercial_document_lines(document_id, line_number);
CREATE INDEX idx_document_links_source ON document_line_links(source_line_id, transformation_type);
CREATE INDEX idx_document_links_target ON document_line_links(target_line_id, transformation_type);
CREATE INDEX idx_stock_movements_ledger ON stock_movements(company_id, product_id, warehouse_id, business_date, occurred_at);
CREATE INDEX idx_stock_reservations_active ON stock_reservations(company_id, product_id, warehouse_id, status);

CREATE TRIGGER trg_commercial_documents_posted_no_update
BEFORE UPDATE ON commercial_documents
WHEN OLD.posting_status = 'POSTED'
BEGIN
    SELECT RAISE(ABORT, 'posted commercial document is immutable');
END;

CREATE TRIGGER trg_commercial_documents_posted_no_delete
BEFORE DELETE ON commercial_documents
WHEN OLD.posting_status = 'POSTED'
BEGIN
    SELECT RAISE(ABORT, 'posted commercial document cannot be deleted');
END;

CREATE TRIGGER trg_commercial_lines_posted_no_insert
BEFORE INSERT ON commercial_document_lines
WHEN EXISTS (
    SELECT 1 FROM commercial_documents
    WHERE id = NEW.document_id AND posting_status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'cannot add a line to a posted commercial document');
END;

CREATE TRIGGER trg_commercial_lines_posted_no_update
BEFORE UPDATE ON commercial_document_lines
WHEN EXISTS (
    SELECT 1 FROM commercial_documents
    WHERE id = OLD.document_id AND posting_status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'posted commercial document line is immutable');
END;

CREATE TRIGGER trg_commercial_lines_posted_no_delete
BEFORE DELETE ON commercial_document_lines
WHEN EXISTS (
    SELECT 1 FROM commercial_documents
    WHERE id = OLD.document_id AND posting_status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'posted commercial document line cannot be deleted');
END;

CREATE TRIGGER trg_document_line_links_posted_no_insert
BEFORE INSERT ON document_line_links
WHEN EXISTS (
    SELECT 1
    FROM commercial_document_lines line
    JOIN commercial_documents document ON document.id = line.document_id
    WHERE line.id = NEW.target_line_id
      AND document.posting_status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'cannot add lineage to a posted target commercial document');
END;

CREATE TRIGGER trg_document_line_links_posted_no_update
BEFORE UPDATE ON document_line_links
WHEN EXISTS (
    SELECT 1
    FROM commercial_document_lines line
    JOIN commercial_documents document ON document.id = line.document_id
    WHERE line.id IN (OLD.source_line_id, OLD.target_line_id, NEW.source_line_id, NEW.target_line_id)
      AND document.posting_status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'posted commercial document lineage is immutable');
END;

CREATE TRIGGER trg_document_line_links_posted_no_delete
BEFORE DELETE ON document_line_links
WHEN EXISTS (
    SELECT 1
    FROM commercial_document_lines line
    JOIN commercial_documents document ON document.id = line.document_id
    WHERE line.id IN (OLD.source_line_id, OLD.target_line_id)
      AND document.posting_status = 'POSTED'
)
BEGIN
    SELECT RAISE(ABORT, 'posted commercial document lineage cannot be deleted');
END;

CREATE TRIGGER trg_document_status_history_no_update
BEFORE UPDATE ON document_status_history
BEGIN
    SELECT RAISE(ABORT, 'document status history is append-only');
END;

CREATE TRIGGER trg_document_status_history_no_delete
BEFORE DELETE ON document_status_history
BEGIN
    SELECT RAISE(ABORT, 'document status history is append-only');
END;

CREATE TRIGGER trg_stock_movements_no_update
BEFORE UPDATE ON stock_movements
BEGIN
    SELECT RAISE(ABORT, 'stock movements are append-only');
END;

CREATE TRIGGER trg_stock_movements_no_delete
BEFORE DELETE ON stock_movements
BEGIN
    SELECT RAISE(ABORT, 'stock movements are append-only');
END;
-- END MIGRATION 0003_commerce_inventory.sql

-- BEGIN MIGRATION 0004_accounting_documents_audit.sql
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
-- END MIGRATION 0004_accounting_documents_audit.sql
