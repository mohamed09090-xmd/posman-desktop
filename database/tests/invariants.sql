-- This file is executed after deterministic positive fixtures are created.
-- A failed CHECK aborts verification with the assertion name in the inserted row context.

CREATE TEMP TABLE invariant_assertions (
    assertion_name TEXT NOT NULL,
    passed INTEGER NOT NULL CHECK (passed = 1)
);

INSERT INTO invariant_assertions
SELECT 'foreign keys enabled', (SELECT foreign_keys FROM pragma_foreign_keys);

INSERT INTO invariant_assertions
SELECT 'five migrations recorded', COUNT(*) = 5
FROM app_migrations;

INSERT INTO invariant_assertions
SELECT 'schema version 0005', MAX(version) = '0005'
FROM app_migrations;

INSERT INTO invariant_assertions
SELECT 'expected application table count', COUNT(*) = 52
FROM sqlite_schema
WHERE type = 'table' AND name NOT LIKE 'sqlite_%';

INSERT INTO invariant_assertions
SELECT 'no declared REAL columns', COUNT(*) = 0
FROM sqlite_schema AS schema_object,
     pragma_table_info(schema_object.name) AS column_info
WHERE schema_object.type = 'table'
  AND schema_object.name NOT LIKE 'sqlite_%'
  AND upper(trim(column_info.type)) LIKE '%REAL%';

INSERT INTO invariant_assertions
SELECT 'all text business primary keys explicitly not null',
       COUNT(*) = 51
       AND SUM(CASE WHEN column_info."notnull" = 1 THEN 1 ELSE 0 END) = 51
FROM sqlite_schema AS schema_object,
     pragma_table_info(schema_object.name) AS column_info
WHERE schema_object.type = 'table'
  AND schema_object.name NOT LIKE 'sqlite_%'
  AND column_info.name = 'id'
  AND upper(trim(column_info.type)) = 'TEXT'
  AND column_info.pk > 0;

INSERT INTO invariant_assertions
SELECT 'foreign key check clean', COUNT(*) = 0
FROM pragma_foreign_key_check;

INSERT INTO invariant_assertions
SELECT 'opening stock movement exists once', COUNT(*) = 1
FROM stock_movements
WHERE posting_event_key = 'opening-stock:opening-doc:opening-line';

INSERT INTO invariant_assertions
SELECT 'stock balance equation holds', COUNT(*) = 0
FROM stock_balances
WHERE available_scaled <> on_hand_scaled - reserved_scaled;

INSERT INTO invariant_assertions
SELECT 'order delivered quantity equals source quantity',
       COALESCE(SUM(link.transformed_quantity_scaled), 0) = source_line.quantity_scaled
FROM commercial_document_lines AS source_line
LEFT JOIN document_line_links AS link
       ON link.source_line_id = source_line.id
      AND link.transformation_type = 'ORDER_TO_DELIVERY'
WHERE source_line.id = 'line-sales-order-1';

INSERT INTO invariant_assertions
SELECT 'delivered quantity invoiced completely',
       COALESCE(SUM(link.transformed_quantity_scaled), 0) = 20000000
FROM document_line_links AS link
WHERE link.transformation_type = 'DELIVERY_TO_INVOICE'
  AND link.source_line_id IN ('line-delivery-1', 'line-delivery-2');

INSERT INTO invariant_assertions
SELECT 'posted invoice fixture line count unchanged', COUNT(*) = 2
FROM commercial_document_lines
WHERE document_id = 'sales-invoice-1';

INSERT INTO invariant_assertions
SELECT 'posted balanced entry fixture line count unchanged', COUNT(*) = 2
FROM journal_entry_lines
WHERE journal_entry_id = 'entry-balanced';

INSERT INTO invariant_assertions
SELECT 'posted journal entries are balanced', COUNT(*) = 0
FROM journal_entries AS entry
WHERE entry.status = 'POSTED'
  AND (
      (SELECT COALESCE(SUM(line.debit_minor), 0)
       FROM journal_entry_lines AS line
       WHERE line.journal_entry_id = entry.id)
      <>
      (SELECT COALESCE(SUM(line.credit_minor), 0)
       FROM journal_entry_lines AS line
       WHERE line.journal_entry_id = entry.id)
  );

INSERT INTO invariant_assertions
SELECT 'safe roles seeded', COUNT(*) = 6
FROM roles
WHERE company_id IS NULL AND is_system = 1;

INSERT INTO invariant_assertions
SELECT 'safe permissions seeded', COUNT(*) = 22
FROM permissions;

INSERT INTO invariant_assertions
SELECT 'setup singleton enforced', COUNT(*) <= 1
FROM setup_drafts
WHERE is_active = 1;

INSERT INTO invariant_assertions
SELECT 'one active recovery code per user', COUNT(*) = 0
FROM (
    SELECT company_id, user_id, COUNT(*) AS active_count
    FROM user_recovery_codes
    WHERE used_at IS NULL AND revoked_at IS NULL
    GROUP BY company_id, user_id
    HAVING active_count > 1
);

INSERT INTO invariant_assertions
SELECT 'normalized username unique index exists', COUNT(*) = 1
FROM sqlite_schema
WHERE type = 'index' AND name = 'uq_users_company_username_normalized';

INSERT INTO invariant_assertions
SELECT 'document sequence type scope index exists', COUNT(*) = 1
FROM sqlite_schema
WHERE type = 'index' AND name = 'uq_document_sequences_company_year_type';

INSERT INTO invariant_assertions
SELECT 'session timeout range holds', COUNT(*) = 0
FROM company_settings
WHERE session_idle_timeout_minutes NOT BETWEEN 5 AND 120;

INSERT INTO invariant_assertions
SELECT 'default margin range holds', COUNT(*) = 0
FROM company_settings
WHERE default_margin_rate_scaled NOT BETWEEN 0 AND 1000000;

DROP TABLE invariant_assertions;
