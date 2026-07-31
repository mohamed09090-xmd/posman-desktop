-- POSMAN schema invariants through PHASE 05.
-- Executed by scripts/verify_schema.py after PHASE 05 constraint fixtures.

CREATE TEMP TABLE invariant_assertions (
    assertion_name TEXT NOT NULL,
    passed INTEGER NOT NULL CHECK (passed = 1)
);

INSERT INTO invariant_assertions
SELECT 'foreign keys enabled', (SELECT foreign_keys FROM pragma_foreign_keys);

INSERT INTO invariant_assertions
SELECT 'five migrations recorded', COUNT(*) = 5 FROM app_migrations;

INSERT INTO invariant_assertions
SELECT 'schema version 0005', MAX(version) = '0005' FROM app_migrations;

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
SELECT 'session timeout default and range', COUNT(*) = 0
FROM company_settings
WHERE session_idle_timeout_minutes NOT BETWEEN 5 AND 120;

INSERT INTO invariant_assertions
SELECT 'margin default and range', COUNT(*) = 0
FROM company_settings
WHERE default_margin_rate_scaled NOT BETWEEN 0 AND 1000000;

DROP TABLE invariant_assertions;
