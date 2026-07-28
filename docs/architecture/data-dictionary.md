# POSMAN data dictionary

## Numeric and temporal conventions

- Final money columns ending in `_minor` are integer DZD minor units with scale 2.
- Unit prices and costs ending in `_scaled` use scale 4.
- Quantities ending in `_scaled` use scale 6.
- Percentage rates ending in `rate_scaled` use scale 4 percentage points; `19.0000%` is `190000`.
- Date-only fields use `YYYY-MM-DD`; event/record timestamps use ISO 8601 UTC.
- UUID-compatible business identifiers are stored as `TEXT`; human numbers are separate.
- `created_at`, `updated_at`, actor fields, and `row_version` appear on mutable business records where appropriate. Append-only ledgers intentionally omit meaningless update fields.

## Tables

| Table | Purpose | Primary key, foreign keys, and important columns | Numeric scale/unit | Uniqueness | Mutability and deletion | Source-of-truth status |
|---|---|---|---|---|---|---|
| `app_migrations` | Installed schema ledger. | PK `id`; `version`, `name`, checksum, UTC apply time. | None. | `version` unique. | Append-only by migration runner; no business deletion. | Authoritative installed version evidence. |
| `companies` | Installed merchant/company identity. | PK `id`; code, Arabic/French/legal names, DZD, timezone. | None. | Global `code` unique. | Master data; archive with `is_active`, do not delete after transactions. | Company identity. |
| `company_settings` | One-company operational policy. | PK `id`; FK/UK `company_id`; language, price, margin, CUMP, negative-stock, WAL preferences. | None. | One row per company. | Mutable with audit and `row_version`. | Company policy source. |
| `fiscal_years` | Fiscal-year boundaries and state. | PK `id`; FK `company_id`; dates and status. | None. | Company + code unique. | Mutable until closure; retained permanently after use. | Fiscal-year source. |
| `fiscal_periods` | Posting periods and locks. | PK `id`; FKs company/year; period number, date range, status. | None. | Company + year + period unique. | Status changes audited; referenced periods not deleted. | Posting lock source. |
| `document_sequences` | Human-number sequence state. | PK `id`; FKs company/year; type, prefix, next number. | `next_number` integer counter. | Company + year + type + prefix unique. | Transactional updates only; never reused silently. | Document numbering source. |
| `users` | Local user accounts. | PK `id`; FK company; username, display name, password hash, lock state. | Failed count integer. | Company + username unique. | Archive with `is_active`; no seeded users/passwords. | User identity source. |
| `roles` | Global templates and company roles. | PK `id`; nullable FK company; code and bilingual names. | None. | Expression-unique scope + code. | System templates stable; company roles archived. | Role definition source. |
| `permissions` | Stable action permissions. | PK `id`; code, domain, bilingual descriptions, sensitivity. | None. | `code` unique. | Reference data; changes require migration/review. | Permission catalog. |
| `user_roles` | User-to-role assignment. | PK `id`; FKs company, user, role; assignment actor/time. | None. | Company + user + role unique. | Mutable assignment history should be audited. | Effective role assignment. |
| `role_permissions` | Role-to-permission grant. | PK `id`; nullable company; FKs role/permission. | None. | Role + permission unique. | Reference/company configuration. | Role grant source. |
| `sessions` | Hashed local sessions. | PK `id`; FKs company/user; token hash, expiry/revocation. | None. | Token hash unique. | Revocable; expired rows may be purged by policy. | Active local session source. |
| `units` | Company measurement units. | PK `id`; FK company; code, bilingual names, decimal scale. | `decimal_scale` 0–6 metadata. | Company + code unique. | Archive with `is_active`. | Unit definition source. |
| `tax_rates` | Effective-dated tax definitions. | PK `id`; FK company; code/names, rate, validity. | `rate_scaled`: percentage scale 4. | Company + code + valid-from unique. | Never rewrite historical meaning; end-date/archive. | Tax reference source, not permanent code logic. |
| `payment_terms` | Due-date defaults. | PK `id`; FK company; code/name, due days. | Days integer. | Company + code unique. | Archive with `is_active`. | Payment-term source. |
| `payment_methods` | Allowed tender methods. | PK `id`; FK company; kind, reference requirement. | None. | Company + code unique. | Archive with `is_active`. | Payment-method source. |
| `warehouses` | Stock locations at warehouse level. | PK `id`; FK company; code/name/default flag. | None. | Company + code; one default partial index. | Archive with `is_active`; retain referenced rows. | Warehouse source. |
| `warehouse_locations` | Bins/locations inside warehouses. | PK `id`; FKs company/warehouse; code/name. | None. | Company + warehouse + code unique. | Archive with `is_active`. | Warehouse-location source. |
| `product_families` | Hierarchical product grouping/defaults. | PK `id`; self-FK parent; FKs company/default tax. | Default margin rate scale 4. | Company + code unique. | Archive with `is_active`. | Family/default metadata. |
| `products` | Commercial product master without balances. | PK `id`; FKs company/family/unit/tax; code/barcode/names/kind. | Minimum qty scale 6; default prices scale 4. | Company + code; non-empty barcode unique per company. | Archive with `is_active`; do not delete referenced products. | Product commercial source; not stock source. |
| `price_lists` | Named DZD selling-price sets. | PK `id`; FK company; price mode/default flag. | None. | Company + code; one default list. | Archive with `is_active`. | Price-list definition. |
| `product_prices` | Effective product price rows. | PK `id`; FKs company/list/product; validity. | `unit_price_scaled` scale 4. | Company + list + product + valid-from unique. | Add/end-date rows; preserve history. | Historical price source. |
| `partners` | Unified customer/supplier master. | PK `id`; FKs company/payment term; flags, identifiers, credit limit. | Credit limit minor scale 2. | Company + code unique; at least one party flag. | Archive with `is_active`. | Partner source. |
| `partner_addresses` | Billing/delivery addresses. | PK `id`; FKs company/partner; kind, address, default flag. | None. | One default per partner/kind. | Archive; cascade only while partner is deletable. | Partner address source. |
| `partner_contacts` | Partner contact people. | PK `id`; FKs company/partner; phone/email, primary flag. | None. | One primary contact per partner. | Archive; requires phone or email. | Partner contact source. |
| `commercial_documents` | Unified sales/purchase/inventory headers. | PK `id`; FKs company/fiscal/partner/warehouse/source; type, number, states, dates, totals. | Discount/tax/HT/TTC minor scale 2; rates scale 4. | Company + fiscal year + type + number; posting key unique. | `POSTED` header immutable/no delete via triggers. | Commercial header and posting state. |
| `commercial_document_lines` | Snapshot lines for documents. | PK `id`; FKs company/document/product/warehouse/unit; product/unit/tax snapshots. | Qty scale 6; prices/cost scale 4; rates scale 4; totals scale 2. | Document line number unique. | Lines of posted documents reject insert/update/delete. | Historical commercial line source. |
| `document_line_links` | Quantity-level conversion lineage. | PK `id`; FKs source/target lines; transformation type/actor/time. | Transformed quantity scale 6 and >0. | Source + target pair unique; no self-link. | Lineage touching posted docs immutable. | Order→delivery/receipt→invoice lineage source. |
| `document_status_history` | Append-only status events. | PK `id`; FKs company/document; old/new status, actor/time/reason. | Row version integer snapshot. | Event ID unique. | Update/delete rejected by triggers. | Status audit source. |
| `payments` | Receipts and disbursements. | PK `id`; FKs company/fiscal/partner/method; number, state, date. | Amount minor scale 2. | Scoped payment number; posting key unique when present. | Posted/reversed behavior enforced by future service; retained. | Payment source. |
| `payment_allocations` | Payment-to-document settlement links. | PK `id`; FKs company/payment/document/reversal. | Allocated amount minor scale 2 and >0. | Allocation row identity; status constrained. | Reverse with a new linked allocation, not destructive edit after posting. | Settlement allocation source. |
| `stock_movements` | Authoritative inventory ledger. | PK `id`; FKs company/product/warehouse/location/source/reversal; event key/transfer group. | Qty delta scale 6; unit/average cost scale 4; extended cost scale 2. | Company + posting event key unique. | All updates/deletes rejected. Corrections are new movements. | Inventory source of truth. |
| `stock_balances` | Rebuildable current stock projection. | PK `id`; FKs company/product/warehouse/location/last movement. | On-hand/reserved/available scale 6; average cost scale 4. | Company + product + warehouse + nullable location unique. | Mutable projection; may be discarded/rebuilt. | Not authoritative. |
| `stock_reservations` | Order-line reservation state. | PK `id`; FKs company/product/warehouse/location/source line. | Reserved quantity scale 6 and >0. | Reservation identity. | State-managed by future inventory service; retained for traceability. | Reservation source. |
| `inventory_counts` | Physical count sessions. | PK `id`; FKs company/warehouse/optional document; number/date/status. | None. | Company + warehouse + count number unique. | Posted count retained; corrections create adjustment evidence. | Count session source. |
| `inventory_count_lines` | Counted product quantities. | PK `id`; FKs company/count/product/location. | System/counted/variance qty scale 6; unit cost scale 4. | Count + product + location unique. | Mutable before posting; retained afterward. | Physical count observation. |
| `accounts` | Configurable chart of accounts. | PK `id`; self-FK parent; FK company; code/type/normal side. | None. | Company + account code unique. | Archive with `is_active`; posted references retained. | Account definition source. |
| `accounting_journals` | Configurable journals. | PK `id`; FK company; code/name/type. | None. | Company + code unique. | Archive with `is_active`. | Journal definition source. |
| `posting_rules` | Effective account mappings. | PK `id`; FKs company/journal/debit/credit accounts; event, dates, conditions. | Priority integer. | Company + code + valid-from unique. | Effective-date/version by new rows; no hardcoded accounts. | Posting mapping source. |
| `journal_entries` | Accounting entry headers. | PK `id`; FKs company/fiscal/journal/source/reversal; number/state/idempotency. | Header stores no derived floating totals. | Scoped entry number and company idempotency key unique. | Posting trigger requires balance/open period; posted update/delete rejected. | Accounting entry source. |
| `journal_entry_lines` | Debit/credit detail. | PK `id`; FKs company/entry/account/partner/product. | Debit/credit minor scale 2; exactly one positive side. | Entry line number unique. | Posted-entry line insert/update/delete rejected. | Accounting amount source. |
| `posting_attempts` | Posting execution/retry evidence. | PK `id`; FKs company/result entry/retry; event, status, error. | None. | Attempt identity; idempotency indexed by context. | Append-oriented; final status recorded by service. | Posting operational evidence. |
| `document_templates` | Company print-template identities. | PK `id`; FK company; code/type/name. | None. | Company + code unique. | Archive with `is_active`. | Template identity source. |
| `document_template_versions` | Immutable HTML/CSS template versions. | PK `id`; FKs company/template; version/hash/publish flag. | Version integer. | Template + version; company + hash unique. | All updates/deletes rejected. | Historical template source. |
| `rendered_documents` | Historical render metadata. | PK `id`; FKs company/source document/template version; path/hash/format. | None. | Source document + hash unique. | All updates/deletes rejected. | Historical rendered artifact evidence. |
| `attachments` | External-file metadata. | PK `id`; FK company; entity reference, path, hash, size/MIME. | Size bytes integer. | Entity + hash unique. | File lifecycle requires audited service; no binary SQL content. | Attachment metadata source. |
| `audit_logs` | Sensitive-action audit ledger. | PK `id`; FKs company/actor; action/entity/time/outcome/details. | None. | Audit ID unique. | All updates/deletes rejected. | Audit source of truth. |
| `idempotency_keys` | Cross-domain duplicate-command guard. | PK `id`; FK company; namespace/key/hash/state/result. | None. | Company + namespace + key unique. | State finalized by service; expiry policy may purge only when safe. | Idempotency coordination source. |
| `backup_history` | Backup/verification operation history. | PK `id`; FK company; kind/path/schema/hash/size/status/times. | Size bytes integer. | Backup ID unique. | Operational history retained and audited. | Backup operation evidence. |

## Trigger catalog

| Trigger group | Tables | Purpose |
|---|---|---|
| Posted commercial header protection | `commercial_documents` | Reject update/delete when the old posting state is `POSTED`. |
| Posted commercial line protection | `commercial_document_lines` | Reject insert/update/delete against a posted parent. |
| Posted lineage protection | `document_line_links` | Reject changes when either source or target belongs to a posted document. |
| Append-only commercial history | `document_status_history` | Reject all updates and deletes. |
| Append-only inventory | `stock_movements` | Reject all updates and deletes. |
| Journal posting validation | `journal_entries` | On transition to `POSTED`, require matching open period, at least two lines, positive total, and balanced debit/credit. |
| Posted journal protection | `journal_entries`, `journal_entry_lines` | Reject posted header update/delete and line insert/update/delete. |
| Immutable print history | `document_template_versions`, `rendered_documents` | Reject all updates and deletes. |
| Append-only audit | `audit_logs` | Reject all updates and deletes. |

There are 24 triggers in the Phase 01 schema. No trigger calculates document totals, tax, CUMP, posting-rule selection, workflow orchestration, or stock-balance projection.

## Controlled vocabularies

CHECK constraints define allowed document types, document status families, posting states, payment states, inventory movement types, count states, journal states, account types, tender types, and audit outcomes. Detailed transition authorization remains a Rust domain-service responsibility.

## Application-service invariants

The future Rust layer must enforce company-scope consistency, aggregate line over-conversion prevention, CUMP calculation, stock availability/negative-stock authorization, document total calculation, tax rounding, payment-allocation aggregate limits, posting-rule selection, and full workflow transition graphs within explicit SQLite transactions.
