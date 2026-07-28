# POSMAN data dictionary
## Numeric and temporal conventions
- `*_minor`: final DZD amount, integer scale 2.
- unit price/cost `*_scaled`: integer scale 4.
- quantity `*_scaled`: integer scale 6.
- percentage `*_rate_scaled` / `rate_scaled`: percentage points scale 4 (`19.0000%` = `190000`).
- booleans: integer `0/1`; counters, versions, line/period numbers, days, bytes, and priorities are unscaled integers.
- dates: ISO `YYYY-MM-DD`; timestamps: ISO 8601 UTC text.
- every business-table `TEXT` primary key named `id` is explicitly non-null and rejects blank trimmed values.
## Tables
| Table | Purpose | Primary key / foreign keys | Important columns | Uniqueness | Mutability / archival | Related triggers | Source-of-truth status |
|---|---|---|---|---|---|---|---|
| `accounting_journals` | Configurable accounting journals. | PK `id`; FKs `company_id`→`companies`. | code/names, journal type, active flag. | company + code. | Archive with is_active. | None. | Journal definition. |
| `accounts` | Configurable chart of accounts. | PK `id`; FKs `parent_account_id`→`accounts`, `company_id`→`companies`. | parent, code/names, type, normal side, posting/active flags. | company + code. | Archive with is_active; posted references retained. | None. | Account definition. |
| `app_migrations` | Installed migration ledger. | PK `id`; no foreign keys. | version, name, checksum_sha256, applied_at. | version. | Append-only by migration runner; never business-deleted. | None. | Installed schema-version evidence. |
| `attachments` | External-file metadata only. | PK `id`; FKs `company_id`→`companies`. | entity reference, file/path/MIME/hash/size, actor/time. | entity + hash. | Lifecycle through audited service; no SQL binary content. | None. | Attachment metadata. |
| `audit_logs` | Sensitive-action audit ledger. | PK `id`; FKs `actor_user_id`→`users`, `company_id`→`companies`. | actor, action/entity, time/outcome/correlation/details. | audit id. | All updates/deletes rejected. | `trg_audit_logs_no_delete`, `trg_audit_logs_no_update`. | Audit source of truth. |
| `backup_history` | Backup and verification operation history. | PK `id`; FKs `company_id`→`companies`. | kind/path/schema/hash/size/status/times/error/actor. | backup id. | Operational history retained and audited. | None. | Backup operation evidence. |
| `commercial_document_lines` | Historical document-line snapshots. | Non-null/nonblank TEXT PK `id`; FKs `unit_id`→`units`, `warehouse_id`→`warehouses`, `product_id`→`products`, `document_id`→`commercial_documents`, `company_id`→`companies`. | document/product/unit/warehouse, snapshot text, quantity, price/cost, tax/discount/totals. | document + line number. | Posted parent blocks insert/update/delete. | `trg_commercial_lines_posted_no_delete`, `trg_commercial_lines_posted_no_insert`, `trg_commercial_lines_posted_no_update`. | Historical commercial line source. |
| `commercial_documents` | Unified sales, purchase, and inventory headers. | PK `id`; FKs `source_document_id`→`commercial_documents`, `warehouse_id`→`warehouses`, `partner_id`→`partners`, `fiscal_period_id`→`fiscal_periods`, `fiscal_year_id`→`fiscal_years`, `company_id`→`companies`. | type/number, statuses, dates, party/warehouse, totals, idempotency. | company + year + type + number; company + idempotency key. | Posted header update/delete rejected. | `trg_commercial_documents_posted_no_delete`, `trg_commercial_documents_posted_no_update`. | Commercial header and posting state. |
| `companies` | Merchant/company identity. | PK `id`; no foreign keys. | code, legal/name fields, DZD, timezone, identifiers. | global code. | Mutable master data; archive with is_active. | None. | Company identity. |
| `company_settings` | Per-company operational policy. | PK `id`; FKs `company_id`→`companies`. | language, price mode, margin method, CUMP, negative stock, WAL/backup preferences. | one row per company. | Mutable and audited by future service. | None. | Company policy. |
| `document_line_links` | Quantity-level conversion lineage. | PK `id`; FKs `target_line_id`→`commercial_document_lines`, `source_line_id`→`commercial_document_lines`, `company_id`→`companies`. | source/target lines, transformation type, transformed quantity, actor/time. | source + target. | New link allowed only into draft target; links touching posted docs cannot update/delete. | `trg_document_line_links_posted_no_delete`, `trg_document_line_links_posted_no_insert`, `trg_document_line_links_posted_no_update`. | Order→delivery/receipt→invoice lineage. |
| `document_sequences` | Human-number sequence state. | PK `id`; FKs `fiscal_year_id`→`fiscal_years`, `company_id`→`companies`. | document type, prefix, next number, padding. | company + year + type + prefix. | Transactional counter; numbers are not silently reused. | None. | Document numbering source. |
| `document_status_history` | Append-only status events. | PK `id`; FKs `document_id`→`commercial_documents`, `company_id`→`companies`. | document, old/new status, reason, row version, actor/time. | event id. | Update/delete rejected. | `trg_document_status_history_no_delete`, `trg_document_status_history_no_update`. | Document status audit. |
| `document_template_versions` | Immutable HTML/CSS template versions. | PK `id`; FKs `document_template_id`→`document_templates`, `company_id`→`companies`. | template, version, content/hash, publish flag. | template + version; company + hash. | All updates/deletes rejected. | `trg_document_template_versions_no_delete`, `trg_document_template_versions_no_update`. | Historical template source. |
| `document_templates` | Company print-template identities. | PK `id`; FKs `company_id`→`companies`. | code/type/names, active flag. | company + code. | Archive with is_active. | None. | Template identity. |
| `fiscal_periods` | Posting periods and locks. | PK `id`; FKs `fiscal_year_id`→`fiscal_years`, `company_id`→`companies`. | period number, dates, status, close actor/time. | company + year + period number. | Retained; close/reopen must be audited. | None. | Posting-lock source. |
| `fiscal_years` | Fiscal-year boundaries and state. | PK `id`; FKs `company_id`→`companies`. | code, starts_on, ends_on, status. | company + code. | Retained; status closes/locks periods. | None. | Fiscal-year source. |
| `idempotency_keys` | Cross-domain duplicate-command guard. | PK `id`; FKs `company_id`→`companies`. | namespace/key/request hash/state/result/times. | company + namespace + key. | Finalized by service; expiry only when safe. | None. | Idempotency coordination source. |
| `inventory_count_lines` | Counted product observations. | PK `id`; FKs `warehouse_location_id`→`warehouse_locations`, `product_id`→`products`, `inventory_count_id`→`inventory_counts`, `company_id`→`companies`. | count/product/location, system/counted/variance quantity, unit cost. | count + product + nullable location. | Mutable before post; retained after. | None. | Physical count observation. |
| `inventory_counts` | Physical count sessions. | PK `id`; FKs `adjustment_document_id`→`commercial_documents`, `warehouse_id`→`warehouses`, `company_id`→`companies`. | warehouse, adjustment document, number/date/status. | company + warehouse + count number. | Retained after posting. | None. | Count session source. |
| `journal_entries` | Accounting entry headers. | PK `id`; FKs `reversal_of_entry_id`→`journal_entries`, `source_document_id`→`commercial_documents`, `accounting_journal_id`→`accounting_journals`, `fiscal_period_id`→`fiscal_periods`, `fiscal_year_id`→`fiscal_years`, `company_id`→`companies`. | period/journal/source/reversal, number/date/status/event/idempotency. | scoped entry number; company + idempotency key. | Must start DRAFT; posted update/delete rejected. | `trg_journal_entries_no_direct_posted_insert`, `trg_journal_entries_posted_no_delete`, `trg_journal_entries_posted_no_update`, `trg_journal_entries_validate_posting`. | Accounting entry source. |
| `journal_entry_lines` | Debit/credit details. | Non-null/nonblank TEXT PK `id`; FKs `product_id`→`products`, `partner_id`→`partners`, `account_id`→`accounts`, `journal_entry_id`→`journal_entries`, `company_id`→`companies`. | entry/account, optional partner/product, line number, description, sides. | entry + line number. | Posted entry blocks insert/update/delete. | `trg_journal_lines_posted_no_delete`, `trg_journal_lines_posted_no_insert`, `trg_journal_lines_posted_no_update`. | Accounting amount source. |
| `partner_addresses` | Billing/delivery addresses. | PK `id`; FKs `partner_id`→`partners`, `company_id`→`companies`. | partner, kind, address fields, default/active flags. | one active default per partner/kind. | Archive; cascade only before partner is referenced. | None. | Partner address source. |
| `partner_contacts` | Partner contact people. | PK `id`; FKs `partner_id`→`partners`, `company_id`→`companies`. | partner, name/title, phone/email, primary/active flags. | one active primary per partner. | Archive; phone or email required. | None. | Partner contact source. |
| `partners` | Unified customers and suppliers. | PK `id`; FKs `payment_term_id`→`payment_terms`, `company_id`→`companies`. | code/names, party flags, identifiers, terms, credit limit. | company + code. | Archive with is_active. | None. | Partner source. |
| `payment_allocations` | Payment-to-document settlement links. | PK `id`; FKs `reversal_of_allocation_id`→`payment_allocations`, `document_id`→`commercial_documents`, `payment_id`→`payments`, `company_id`→`companies`. | payment, document, reversal link, amount, state, actor/time. | allocation id. | Reverse with linked allocation, not destructive correction. | None. | Settlement allocation source. |
| `payment_methods` | Allowed tender methods. | PK `id`; FKs `company_id`→`companies`. | code/names, kind, reference requirement. | company + code. | Archive with is_active. | None. | Payment-method source. |
| `payment_terms` | Due-date defaults. | PK `id`; FKs `company_id`→`companies`. | code/names, due days. | company + code. | Archive with is_active. | None. | Payment-term source. |
| `payments` | Receipts and disbursements. | PK `id`; FKs `payment_method_id`→`payment_methods`, `partner_id`→`partners`, `fiscal_period_id`→`fiscal_periods`, `fiscal_year_id`→`fiscal_years`, `company_id`→`companies`. | party/method, number/kind/status, dates, amount, idempotency. | scoped number; company + idempotency key. | Retained; corrections by reversal. | None. | Payment source. |
| `permissions` | Stable action permission catalog. | PK `id`; no foreign keys. | code, domain, bilingual descriptions, sensitivity. | global code. | Reference data changed through reviewed migration. | None. | Permission catalog. |
| `posting_attempts` | Posting execution/retry evidence. | PK `id`; FKs `retry_of_attempt_id`→`posting_attempts`, `result_entry_id`→`journal_entries`, `company_id`→`companies`. | event/idempotency, result/retry links, attempt/status/error/times. | attempt id. | Append-oriented operational evidence. | None. | Posting attempt source. |
| `posting_rules` | Effective account mappings. | PK `id`; FKs `credit_account_id`→`accounts`, `debit_account_id`→`accounts`, `accounting_journal_id`→`accounting_journals`, `company_id`→`companies`. | journal, debit/credit accounts, event, condition, priority, validity. | company + code + valid_from. | Version through new effective-dated rows. | None. | Posting mapping source. |
| `price_lists` | Named DZD selling-price sets. | PK `id`; FKs `company_id`→`companies`. | code/names, HT/TTC mode, default/active flags. | company + code; one active default. | Archive with is_active. | None. | Price-list definition. |
| `product_families` | Hierarchical product groups/defaults. | PK `id`; FKs `default_tax_rate_id`→`tax_rates`, `parent_family_id`→`product_families`, `company_id`→`companies`. | parent, code/names, default tax/margin. | company + code. | Archive with is_active. | None. | Family/default metadata. |
| `product_prices` | Effective product prices. | PK `id`; FKs `product_id`→`products`, `price_list_id`→`price_lists`, `company_id`→`companies`. | list, product, unit price, valid_from/to. | company + list + product + valid_from. | Add/end-date rows; preserve history. | None. | Historical price source. |
| `products` | Commercial product master without balances. | PK `id`; FKs `default_tax_rate_id`→`tax_rates`, `unit_id`→`units`, `product_family_id`→`product_families`, `company_id`→`companies`. | family, unit, tax, code/barcode, names, kind, prices/minimum. | company + code; non-empty barcode per company. | Archive with is_active; referenced rows retained. | None. | Product commercial source, not inventory. |
| `rendered_documents` | Historical render metadata. | PK `id`; FKs `template_version_id`→`document_template_versions`, `source_document_id`→`commercial_documents`, `company_id`→`companies`. | source document, template version, format/path/hash, actor/time. | source document + hash. | All updates/deletes rejected. | `trg_rendered_documents_no_delete`, `trg_rendered_documents_no_update`. | Rendered artifact evidence. |
| `role_permissions` | Role-to-permission grants. | PK `id`; FKs `permission_id`→`permissions`, `role_id`→`roles`, `company_id`→`companies`. | role, permission, optional company scope. | role + permission. | Reference/company configuration. | None. | Role grant source. |
| `roles` | Global templates and company roles. | PK `id`; FKs `company_id`→`companies`. | code, bilingual names, system/active flags. | global code or company + code via partial indexes. | System templates stable; company roles archived. | None. | Role definition. |
| `sessions` | Hashed local sessions. | PK `id`; FKs `user_id`→`users`, `company_id`→`companies`. | user, token hash, expiry, revocation, last seen. | token hash. | Revocable; expired rows may be purged by policy. | None. | Active session source. |
| `stock_balances` | Rebuildable current inventory projection. | PK `id`; FKs `last_movement_id`→`stock_movements`, `warehouse_location_id`→`warehouse_locations`, `warehouse_id`→`warehouses`, `product_id`→`products`, `company_id`→`companies`. | product/location, on-hand/reserved/available, average cost, last movement. | company + product + warehouse + nullable location. | Mutable projection; may be discarded/rebuilt. | None. | Not authoritative. |
| `stock_movements` | Authoritative inventory ledger. | PK `id`; FKs `reversal_of_movement_id`→`stock_movements`, `source_line_id`→`commercial_document_lines`, `source_document_id`→`commercial_documents`, `warehouse_location_id`→`warehouse_locations`, `warehouse_id`→`warehouses`, `product_id`→`products`, `company_id`→`companies`. | product/location/source, movement type/date, quantity/cost snapshots, event key, transfer/reversal links. | company + posting event key. | All updates/deletes rejected. | `trg_stock_movements_no_delete`, `trg_stock_movements_no_update`. | Inventory source of truth. |
| `stock_reservations` | Order-line reservation state. | PK `id`; FKs `source_line_id`→`commercial_document_lines`, `warehouse_location_id`→`warehouse_locations`, `warehouse_id`→`warehouses`, `product_id`→`products`, `company_id`→`companies`. | product/location/source line, reserved quantity, status. | reservation id. | State-managed by future service. | None. | Reservation source. |
| `tax_rates` | Effective-dated tax definitions. | PK `id`; FKs `company_id`→`companies`. | code/names, rate, valid_from/to. | company + code + valid_from. | End-date/archive; never rewrite historical meaning. | None. | Tax reference, not hardcoded logic. |
| `units` | Company measurement units. | PK `id`; FKs `company_id`→`companies`. | code, bilingual names, decimal scale. | company + code. | Archive with is_active. | None. | Unit definition. |
| `user_roles` | User-to-role assignments. | PK `id`; FKs `role_id`→`roles`, `user_id`→`users`, `company_id`→`companies`. | user, role, assigned actor/time. | company + user + role. | Assignment changes audited by future service. | None. | Effective role assignment. |
| `users` | Local user accounts. | PK `id`; FKs `company_id`→`companies`. | username, display name, password hash, lock state, language. | company + username. | Archive with is_active; no seeded user/password. | None. | User identity. |
| `warehouse_locations` | Bins/locations inside warehouses. | PK `id`; FKs `warehouse_id`→`warehouses`, `company_id`→`companies`. | warehouse, code, names. | company + warehouse + code. | Archive with is_active. | None. | Location source. |
| `warehouses` | Warehouse master. | PK `id`; FKs `company_id`→`companies`. | code/names, address, default/active flags. | company + code; one active default. | Archive with is_active. | None. | Warehouse source. |

## Numeric columns by table
This appendix enumerates every declared `INTEGER` application column and its unit/meaning.
- **`accounting_journals`:** `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`accounts`:** `allow_posting` — boolean 0/1; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`app_migrations`:** `id` — migration ledger integer key
- **`attachments`:** `size_bytes` — bytes
- **`audit_logs`:** No INTEGER columns.
- **`backup_history`:** `size_bytes` — bytes
- **`commercial_document_lines`:** `line_number` — unscaled integer counter/order; `quantity_scaled` — quantity, scale 6; `unit_price_scaled` — DZD unit value, scale 4; `unit_cost_scaled` — DZD unit value, scale 4; `line_discount_rate_scaled` — percentage points, scale 4; `line_discount_minor` — DZD final amount, scale 2; `allocated_header_discount_minor` — DZD final amount, scale 2; `tax_rate_scaled` — percentage points, scale 4; `line_ht_minor` — DZD final amount, scale 2; `line_tax_minor` — DZD final amount, scale 2; `line_ttc_minor` — DZD final amount, scale 2; `row_version` — optimistic concurrency counter
- **`commercial_documents`:** `header_discount_rate_scaled` — percentage points, scale 4; `header_discount_minor` — DZD final amount, scale 2; `total_ht_minor` — DZD final amount, scale 2; `total_tax_minor` — DZD final amount, scale 2; `total_ttc_minor` — DZD final amount, scale 2; `row_version` — optimistic concurrency counter
- **`companies`:** `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`company_settings`:** `automatic_backup_enabled` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`document_line_links`:** `transformed_quantity_scaled` — quantity, scale 6
- **`document_sequences`:** `next_number` — unscaled integer counter/order; `padding_width` — unscaled integer counter/order; `row_version` — optimistic concurrency counter
- **`document_status_history`:** `row_version_snapshot` — unscaled integer counter/order
- **`document_template_versions`:** `version_number` — unscaled integer counter/order; `is_published` — boolean 0/1
- **`document_templates`:** `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`fiscal_periods`:** `period_number` — unscaled integer counter/order; `row_version` — optimistic concurrency counter
- **`fiscal_years`:** `row_version` — optimistic concurrency counter
- **`idempotency_keys`:** No INTEGER columns.
- **`inventory_count_lines`:** `system_quantity_scaled` — quantity, scale 6; `counted_quantity_scaled` — quantity, scale 6; `variance_quantity_scaled` — quantity, scale 6; `unit_cost_scaled` — DZD unit value, scale 4; `row_version` — optimistic concurrency counter
- **`inventory_counts`:** `row_version` — optimistic concurrency counter
- **`journal_entries`:** `row_version` — optimistic concurrency counter
- **`journal_entry_lines`:** `line_number` — unscaled integer counter/order; `debit_minor` — DZD final amount, scale 2; `credit_minor` — DZD final amount, scale 2
- **`partner_addresses`:** `is_default` — boolean 0/1; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`partner_contacts`:** `is_primary` — boolean 0/1; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`partners`:** `is_customer` — boolean 0/1; `is_supplier` — boolean 0/1; `credit_limit_minor` — DZD final amount, scale 2; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`payment_allocations`:** `allocated_amount_minor` — DZD final amount, scale 2
- **`payment_methods`:** `reference_required` — boolean 0/1; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`payment_terms`:** `due_days` — days; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`payments`:** `amount_minor` — DZD final amount, scale 2; `row_version` — optimistic concurrency counter
- **`permissions`:** `is_sensitive` — boolean 0/1
- **`posting_attempts`:** `attempt_number` — unscaled integer counter/order
- **`posting_rules`:** `priority` — unscaled integer counter/order; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`price_lists`:** `is_default` — boolean 0/1; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`product_families`:** `default_margin_rate_scaled` — percentage points, scale 4; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`product_prices`:** `unit_price_scaled` — DZD unit value, scale 4; `row_version` — optimistic concurrency counter
- **`products`:** `stock_tracked` — boolean 0/1; `minimum_stock_scaled` — quantity, scale 6; `default_purchase_price_scaled` — DZD unit value, scale 4; `default_sale_price_scaled` — DZD unit value, scale 4; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`rendered_documents`:** No INTEGER columns.
- **`role_permissions`:** No INTEGER columns.
- **`roles`:** `is_system` — boolean 0/1; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`sessions`:** No INTEGER columns.
- **`stock_balances`:** `on_hand_scaled` — quantity, scale 6; `reserved_scaled` — quantity, scale 6; `available_scaled` — quantity, scale 6; `average_cost_scaled` — DZD unit value, scale 4; `row_version` — optimistic concurrency counter
- **`stock_movements`:** `quantity_delta_scaled` — quantity, scale 6; `quantity_before_scaled` — quantity, scale 6; `quantity_after_scaled` — quantity, scale 6; `unit_cost_scaled` — DZD unit value, scale 4; `average_cost_before_scaled` — DZD unit value, scale 4; `average_cost_after_scaled` — DZD unit value, scale 4; `extended_cost_minor` — DZD final amount, scale 2
- **`stock_reservations`:** `reserved_quantity_scaled` — quantity, scale 6; `row_version` — optimistic concurrency counter
- **`tax_rates`:** `rate_scaled` — percentage points, scale 4; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`units`:** `decimal_scale` — unit quantity precision metadata (0–6); `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`user_roles`:** No INTEGER columns.
- **`users`:** `failed_login_count` — unscaled integer counter/order; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`warehouse_locations`:** `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter
- **`warehouses`:** `is_default` — boolean 0/1; `is_active` — boolean 0/1; `row_version` — optimistic concurrency counter

## Trigger catalog and purpose
- `trg_audit_logs_no_delete` on `audit_logs`: Rejects deletion of immutable or append-only evidence.
- `trg_audit_logs_no_update` on `audit_logs`: Rejects update of immutable or append-only evidence.
- `trg_commercial_documents_posted_no_delete` on `commercial_documents`: Rejects deletion of immutable or append-only evidence.
- `trg_commercial_documents_posted_no_update` on `commercial_documents`: Rejects update of immutable or append-only evidence.
- `trg_commercial_lines_posted_no_delete` on `commercial_document_lines`: Rejects deletion of immutable or append-only evidence.
- `trg_commercial_lines_posted_no_insert` on `commercial_document_lines`: Rejects insertion that would mutate a posted parent/target.
- `trg_commercial_lines_posted_no_update` on `commercial_document_lines`: Rejects update of immutable or append-only evidence.
- `trg_document_line_links_posted_no_delete` on `document_line_links`: Rejects deletion of immutable or append-only evidence.
- `trg_document_line_links_posted_no_insert` on `document_line_links`: Rejects insertion that would mutate a posted parent/target.
- `trg_document_line_links_posted_no_update` on `document_line_links`: Rejects update of immutable or append-only evidence.
- `trg_document_status_history_no_delete` on `document_status_history`: Rejects deletion of immutable or append-only evidence.
- `trg_document_status_history_no_update` on `document_status_history`: Rejects update of immutable or append-only evidence.
- `trg_document_template_versions_no_delete` on `document_template_versions`: Rejects deletion of immutable or append-only evidence.
- `trg_document_template_versions_no_update` on `document_template_versions`: Rejects update of immutable or append-only evidence.
- `trg_journal_entries_no_direct_posted_insert` on `journal_entries`: Forces journal entries through the documented DRAFT-to-POSTED path.
- `trg_journal_entries_posted_no_delete` on `journal_entries`: Rejects deletion of immutable or append-only evidence.
- `trg_journal_entries_posted_no_update` on `journal_entries`: Rejects update of immutable or append-only evidence.
- `trg_journal_entries_validate_posting` on `journal_entries`: Validates open period, minimum line count, positive total, and debit/credit balance during posting.
- `trg_journal_lines_posted_no_delete` on `journal_entry_lines`: Rejects deletion of immutable or append-only evidence.
- `trg_journal_lines_posted_no_insert` on `journal_entry_lines`: Rejects insertion that would mutate a posted parent/target.
- `trg_journal_lines_posted_no_update` on `journal_entry_lines`: Rejects update of immutable or append-only evidence.
- `trg_rendered_documents_no_delete` on `rendered_documents`: Rejects deletion of immutable or append-only evidence.
- `trg_rendered_documents_no_update` on `rendered_documents`: Rejects update of immutable or append-only evidence.
- `trg_stock_movements_no_delete` on `stock_movements`: Rejects deletion of immutable or append-only evidence.
- `trg_stock_movements_no_update` on `stock_movements`: Rejects update of immutable or append-only evidence.

## Application-service invariants
The future Rust layer must enforce company-scope consistency, aggregate line over-conversion prevention, CUMP calculation, negative-stock authorization, reservation consumption, stock-balance projection, document/tax/discount calculations and rounding, allocation aggregate limits, posting-rule selection, detailed workflow transitions, and transfer-pair completeness inside explicit SQLite transactions.

## Patch 01A verifier coverage

The verifier inspects `PRAGMA table_info` for every expected table, requires all 48 `TEXT` business primary keys to report `notnull = 1`, rejects null and blank identifiers, and verifies that commercial and journal child lines cannot be reparented from a draft parent into a posted parent. Rejected reparenting must leave posted line counts and accounting totals/balance unchanged.
