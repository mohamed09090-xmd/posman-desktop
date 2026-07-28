-- POSMAN Phase 01 - reference data, catalog, warehouses, pricing, and partners.

CREATE TABLE units (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
