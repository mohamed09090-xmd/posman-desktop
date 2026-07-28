-- POSMAN Phase 01 - commercial documents, conversion lineage, payments, and inventory.

CREATE TABLE commercial_documents (
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0),
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
    WHERE id IN (OLD.document_id, NEW.document_id) AND posting_status = 'POSTED'
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
