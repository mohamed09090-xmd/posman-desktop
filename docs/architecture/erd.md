# POSMAN SQLite ERD

The schema is split by domain for readability. Mermaid does not express CHECK constraints, partial indexes, trigger behavior, or aggregate application-service invariants; those are documented in the data dictionary and database decisions.

## System, company, fiscal structure, and security

```mermaid
erDiagram
    companies ||--|| company_settings : configures
    companies ||--o{ fiscal_years : owns
    fiscal_years ||--o{ fiscal_periods : contains
    fiscal_years ||--o{ document_sequences : numbers
    companies ||--o{ users : employs
    users ||--o{ sessions : opens
    users ||--o{ user_roles : receives
    roles ||--o{ user_roles : assigned
    roles ||--o{ role_permissions : grants
    permissions ||--o{ role_permissions : includes
    companies {
        TEXT id PK
        TEXT code UK
        TEXT currency_code
    }
    fiscal_years {
        TEXT id PK
        TEXT company_id FK
        TEXT code
    }
    fiscal_periods {
        TEXT id PK
        TEXT fiscal_year_id FK
        TEXT status
    }
    users {
        TEXT id PK
        TEXT company_id FK
        TEXT username
        TEXT password_hash
    }
    roles {
        TEXT id PK
        TEXT company_id FK
        TEXT code
        INTEGER is_system
    }
    permissions {
        TEXT id PK
        TEXT code UK
    }
```

`app_migrations` is installation-scoped. System role templates and permissions are global safe reference data; no administrator is seeded.

## Reference data, catalog, and partners

```mermaid
erDiagram
    companies ||--o{ units : owns
    companies ||--o{ tax_rates : defines
    companies ||--o{ payment_terms : defines
    companies ||--o{ payment_methods : defines
    companies ||--o{ warehouses : owns
    warehouses ||--o{ warehouse_locations : contains
    product_families ||--o{ product_families : parent_of
    product_families ||--o{ products : groups
    units ||--o{ products : measures
    tax_rates ||--o{ products : defaults
    price_lists ||--o{ product_prices : contains
    products ||--o{ product_prices : priced
    partners ||--o{ partner_addresses : has
    partners ||--o{ partner_contacts : has
    payment_terms ||--o{ partners : defaults
    products {
        TEXT id PK
        TEXT company_id FK
        TEXT code
        TEXT barcode
        INTEGER minimum_stock_scaled
        INTEGER default_sale_price_scaled
    }
    tax_rates {
        TEXT id PK
        INTEGER rate_scaled
        TEXT valid_from
        TEXT valid_to
    }
    partners {
        TEXT id PK
        INTEGER is_customer
        INTEGER is_supplier
        INTEGER credit_limit_minor
    }
```

## Commercial documents and quantity lineage

```mermaid
erDiagram
    companies ||--o{ commercial_documents : owns
    fiscal_years ||--o{ commercial_documents : scopes
    fiscal_periods ||--o{ commercial_documents : posts_in
    partners ||--o{ commercial_documents : party
    warehouses ||--o{ commercial_documents : fulfills
    commercial_documents ||--o{ commercial_document_lines : contains
    products ||--o{ commercial_document_lines : snapshots
    commercial_document_lines ||--o{ document_line_links : source
    commercial_document_lines ||--o{ document_line_links : target
    commercial_documents ||--o{ document_status_history : records
    partners ||--o{ payments : pays_or_receives
    payment_methods ||--o{ payments : uses
    payments ||--o{ payment_allocations : allocates
    commercial_documents ||--o{ payment_allocations : settles
    commercial_documents {
        TEXT id PK
        TEXT document_type
        TEXT document_number
        TEXT workflow_status
        TEXT posting_status
        INTEGER total_ttc_minor
    }
    commercial_document_lines {
        TEXT id PK
        INTEGER quantity_scaled
        INTEGER unit_price_scaled
        INTEGER tax_rate_scaled
        INTEGER line_ttc_minor
    }
    document_line_links {
        TEXT id PK
        TEXT source_line_id FK
        TEXT target_line_id FK
        INTEGER transformed_quantity_scaled
    }
```

### Partial conversion path

```mermaid
flowchart LR
    O[Order line<br/>20.000000] -->|8.000000| D1[Delivery line 1]
    O -->|12.000000| D2[Delivery line 2]
    D1 -->|8.000000| I1[Invoice line 1]
    D2 -->|12.000000| I2[Invoice line 2]
```

One source line may feed several targets. Compatible source lines may feed one target. Remaining quantity is derived from source quantity minus linked transformed quantities.

## Inventory

```mermaid
erDiagram
    products ||--o{ stock_movements : moves
    warehouses ||--o{ stock_movements : locates
    warehouse_locations ||--o{ stock_movements : refines
    commercial_documents ||--o{ stock_movements : originates
    commercial_document_lines ||--o{ stock_movements : originates
    stock_movements ||--o{ stock_movements : reverses
    products ||--o{ stock_balances : projects
    warehouses ||--o{ stock_balances : projects
    products ||--o{ stock_reservations : reserves
    commercial_document_lines ||--o{ stock_reservations : demands
    warehouses ||--o{ inventory_counts : counts
    inventory_counts ||--o{ inventory_count_lines : contains
    products ||--o{ inventory_count_lines : counted
    stock_movements {
        TEXT id PK
        TEXT movement_type
        INTEGER quantity_delta_scaled
        INTEGER unit_cost_scaled
        TEXT posting_event_key UK
    }
    stock_balances {
        TEXT id PK
        INTEGER on_hand_scaled
        INTEGER reserved_scaled
        INTEGER available_scaled
        INTEGER average_cost_scaled
    }
```

`stock_movements` is append-only and authoritative. `stock_balances` is a disposable/rebuildable projection. Transfers use paired movements sharing `transfer_group_id`.

## Accounting, print history, audit, and backup

```mermaid
erDiagram
    accounts ||--o{ accounts : parent_of
    accounting_journals ||--o{ posting_rules : selected_by
    accounts ||--o{ posting_rules : maps
    fiscal_periods ||--o{ journal_entries : permits
    accounting_journals ||--o{ journal_entries : contains
    commercial_documents ||--o{ journal_entries : sources
    journal_entries ||--o{ journal_entry_lines : contains
    accounts ||--o{ journal_entry_lines : posted_to
    journal_entries ||--o{ posting_attempts : results
    document_templates ||--o{ document_template_versions : versions
    document_template_versions ||--o{ rendered_documents : renders
    commercial_documents ||--o{ rendered_documents : snapshots
    companies ||--o{ attachments : owns
    companies ||--o{ audit_logs : audits
    companies ||--o{ idempotency_keys : deduplicates
    companies ||--o{ backup_history : records
    journal_entries {
        TEXT id PK
        TEXT status
        TEXT idempotency_key UK
    }
    journal_entry_lines {
        TEXT id PK
        INTEGER debit_minor
        INTEGER credit_minor
    }
    audit_logs {
        TEXT id PK
        TEXT action_code
        TEXT occurred_at
    }
```

## Relationships not fully represented by Mermaid

- Human document numbers are unique by company, fiscal year, and document type.
- Product code and non-empty barcode uniqueness are company-scoped.
- Nullable location uniqueness uses expression indexes for balances and count lines.
- Posted immutability and append-only rules are trigger-protected.
- Journal balance and fiscal-period openness are validated on the `DRAFT` → `POSTED` transition.
- Aggregate over-conversion, company-scope consistency, CUMP, and stock authorization remain explicit Rust application-service invariants.
