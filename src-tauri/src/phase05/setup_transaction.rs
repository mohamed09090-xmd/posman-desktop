use std::collections::{HashMap, HashSet};

use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use super::{
    dto::{CompleteSetupResult, InitialSetupRequest},
    error::{Phase05Error, Phase05Result},
    pricing::fiscal_periods,
    security::{generate_recovery_code, recovery_code_hash},
    state::{new_id, normalize_username, now_iso, trim_optional, trim_required, Phase05Service},
};

const DOCUMENT_SEQUENCES: [(&str, &str); 15] = [
    ("SALES_ORDER", "CMD"),
    ("DELIVERY_NOTE", "BL"),
    ("SALES_INVOICE", "FAC"),
    ("SALES_RETURN", "RET"),
    ("SALES_CREDIT_NOTE", "AV"),
    ("PURCHASE_REQUEST", "DA"),
    ("PURCHASE_ORDER", "BC"),
    ("PURCHASE_RECEIPT", "BR"),
    ("PURCHASE_INVOICE", "FF"),
    ("PURCHASE_RETURN", "RFA"),
    ("PURCHASE_CREDIT_NOTE", "AVF"),
    ("OPENING_STOCK", "STI"),
    ("STOCK_ADJUSTMENT", "AJS"),
    ("STOCK_TRANSFER", "TRF"),
    ("INVENTORY_COUNT", "INV"),
];

const REQUIRED_PERMISSIONS: [(&str, &str, &str, &str, bool); 10] = [
    (
        "company.view",
        "company",
        "عرض بيانات الشركة",
        "Consulter la société",
        false,
    ),
    (
        "company.manage",
        "company",
        "إدارة بيانات الشركة",
        "Gérer la société",
        true,
    ),
    (
        "settings.manage",
        "settings",
        "إدارة الإعدادات",
        "Gérer les paramètres",
        true,
    ),
    (
        "security.users.view",
        "security",
        "عرض المستخدمين",
        "Consulter les utilisateurs",
        true,
    ),
    (
        "security.users.manage",
        "security",
        "إدارة المستخدمين",
        "Gérer les utilisateurs",
        true,
    ),
    (
        "security.roles.manage",
        "security",
        "إدارة الأدوار",
        "Gérer les rôles",
        true,
    ),
    (
        "catalog.view",
        "catalog",
        "عرض دليل المواد",
        "Consulter le catalogue",
        false,
    ),
    (
        "catalog.manage",
        "catalog",
        "إدارة دليل المواد",
        "Gérer le catalogue",
        false,
    ),
    (
        "partners.view",
        "partners",
        "عرض الشركاء",
        "Consulter les partenaires",
        false,
    ),
    (
        "partners.manage",
        "partners",
        "إدارة الشركاء",
        "Gérer les partenaires",
        false,
    ),
];

impl Phase05Service {
    pub fn complete_initial_setup(
        &self,
        request: InitialSetupRequest,
    ) -> Phase05Result<CompleteSetupResult> {
        validate_request(&request)?;
        if request.administrator_password != request.administrator_password_confirmation {
            return Err(Phase05Error::new(
                "PASSWORD_CONFIRMATION_MISMATCH",
                "The administrator password confirmation does not match.",
            ));
        }
        let idempotency_key = trim_required(&request.idempotency_key, "idempotencyKey")?;
        if !(8..=200).contains(&idempotency_key.len()) {
            return Err(Phase05Error::new(
                "SETUP_INVALID_DRAFT",
                "The setup idempotency key is invalid.",
            ));
        }
        let request_hash = deterministic_request_hash(&request);
        let periods = fiscal_periods(&request.fiscal_starts_on, &request.fiscal_ends_on)?;
        let username = normalize_username(&request.administrator_username)?;
        let password_hash = self.password_engine.hash(&request.administrator_password)?;

        let mut connection = self.open()?;
        if let Some(existing) = find_completed_request(&connection, &idempotency_key)? {
            if existing.0 != request_hash {
                return Err(idempotency_conflict());
            }
            return Ok(CompleteSetupResult {
                company_id: existing.1,
                administrator_user_id: existing.2,
                recovery_code: None,
                already_completed: true,
            });
        }
        let company_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM companies", [], |row| row.get(0))?;
        if company_count != 0 {
            return Err(setup_already_completed());
        }

        let recovery_code = generate_recovery_code();
        let recovery_hash = recovery_code_hash(&recovery_code)?;
        let result = execute_setup_transaction(
            &mut connection,
            &request,
            &periods,
            &username,
            &password_hash,
            &recovery_hash,
            &idempotency_key,
            &request_hash,
        );
        match result {
            Ok((company_id, administrator_user_id)) => Ok(CompleteSetupResult {
                company_id,
                administrator_user_id,
                recovery_code: Some(recovery_code),
                already_completed: false,
            }),
            Err(error)
                if matches!(
                    error.code.as_str(),
                    "SETUP_ALREADY_COMPLETED" | "SETUP_IDEMPOTENCY_CONFLICT"
                ) =>
            {
                Err(error)
            }
            Err(_) => Err(Phase05Error::new(
                "SETUP_TRANSACTION_FAILED",
                "Initial setup could not be completed. No setup data was saved.",
            )),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_setup_transaction(
    connection: &mut rusqlite::Connection,
    request: &InitialSetupRequest,
    periods: &[(String, String)],
    username: &str,
    password_hash: &str,
    recovery_hash: &str,
    idempotency_key: &str,
    request_hash: &str,
) -> Phase05Result<(String, String)> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let company_count: i64 =
        transaction.query_row("SELECT COUNT(*) FROM companies", [], |row| row.get(0))?;
    if company_count != 0 {
        return Err(setup_already_completed());
    }

    let timestamp = now_iso()?;
    let company_id = new_id();
    let settings_id = new_id();
    let fiscal_year_id = new_id();
    let warehouse_id = new_id();
    let unit_id = new_id();
    let price_list_id = new_id();
    let payment_term_id = new_id();
    let administrator_user_id = new_id();
    let administrator_role_id = new_id();
    let request_id = new_id();

    transaction.execute(
        r#"
        INSERT INTO initial_setup_requests (
            id, idempotency_key, request_hash_sha256, status, created_at
        ) VALUES (?1, ?2, ?3, 'IN_PROGRESS', ?4)
        "#,
        params![request_id, idempotency_key, request_hash, timestamp],
    )?;
    insert_company(
        &transaction,
        request,
        &company_id,
        &administrator_user_id,
        &timestamp,
    )?;
    transaction.execute(
        r#"
        INSERT INTO company_settings (
            id, company_id, default_language, default_margin_rate_scaled,
            session_idle_timeout_minutes, created_at, created_by,
            updated_at, updated_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?6, ?7)
        "#,
        params![
            settings_id,
            company_id,
            request.language,
            request.default_margin_rate_scaled,
            request.session_idle_timeout_minutes,
            timestamp,
            administrator_user_id
        ],
    )?;
    transaction.execute(
        r#"
        INSERT INTO fiscal_years (
            id, company_id, code, starts_on, ends_on, status,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'OPEN', ?6, ?7, ?6, ?7)
        "#,
        params![
            fiscal_year_id,
            company_id,
            &request.fiscal_starts_on[..4],
            request.fiscal_starts_on,
            request.fiscal_ends_on,
            timestamp,
            administrator_user_id
        ],
    )?;
    for (index, (starts_on, ends_on)) in periods.iter().enumerate() {
        let number = i64::try_from(index + 1).map_err(|_| Phase05Error::internal())?;
        transaction.execute(
            r#"
            INSERT INTO fiscal_periods (
                id, company_id, fiscal_year_id, period_number, name,
                starts_on, ends_on, status, created_at, created_by,
                updated_at, updated_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'OPEN', ?8, ?9, ?8, ?9)
            "#,
            params![
                new_id(),
                company_id,
                fiscal_year_id,
                number,
                format!("M{number:02}"),
                starts_on,
                ends_on,
                timestamp,
                administrator_user_id
            ],
        )?;
    }

    insert_warehouse(
        &transaction,
        request,
        &company_id,
        &warehouse_id,
        &administrator_user_id,
        &timestamp,
    )?;
    insert_unit_and_price_list(
        &transaction,
        &company_id,
        &unit_id,
        &price_list_id,
        &administrator_user_id,
        &timestamp,
    )?;
    insert_payment_defaults(
        &transaction,
        &company_id,
        &payment_term_id,
        &administrator_user_id,
        &timestamp,
    )?;
    let default_tax_id = insert_taxes(
        &transaction,
        request,
        &company_id,
        &administrator_user_id,
        &timestamp,
    )?;
    transaction.execute(
        "UPDATE company_settings SET default_tax_rate_id=?1 WHERE id=?2 AND company_id=?3",
        params![default_tax_id, settings_id, company_id],
    )?;
    insert_sequences(
        &transaction,
        &company_id,
        &fiscal_year_id,
        &administrator_user_id,
        &timestamp,
    )?;
    insert_administrator_security(
        &transaction,
        request,
        &company_id,
        &administrator_user_id,
        &administrator_role_id,
        username,
        password_hash,
        recovery_hash,
        &timestamp,
    )?;
    insert_setup_audit(
        &transaction,
        &company_id,
        &administrator_user_id,
        idempotency_key,
        &timestamp,
    )?;
    transaction.execute(
        r#"
        UPDATE initial_setup_requests
        SET status='SUCCEEDED', result_company_id=?1, completed_at=?2
        WHERE id=?3 AND status='IN_PROGRESS'
        "#,
        params![company_id, timestamp, request_id],
    )?;
    transaction.execute(
        r#"
        UPDATE setup_drafts
        SET is_active=0, updated_at=?1, row_version=row_version+1
        WHERE is_active=1
        "#,
        [timestamp],
    )?;
    transaction.commit()?;
    Ok((company_id, administrator_user_id))
}

fn insert_company(
    transaction: &Transaction<'_>,
    request: &InitialSetupRequest,
    company_id: &str,
    user_id: &str,
    timestamp: &str,
) -> Phase05Result<()> {
    transaction.execute(
        r#"
        INSERT INTO companies (
            id, code, legal_name, name_ar, name_fr, activity_description,
            legal_form, social_capital_minor, tax_identifier,
            trade_register_number, statistical_identifier, tax_article_number,
            bank_rib, wilaya_code, address_text, city, postal_code, phone, email,
            created_at, created_by, updated_at, updated_by
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?20, ?21
        )
        "#,
        params![
            company_id,
            trim_required(&request.company_code, "companyCode")?,
            trim_required(&request.legal_name, "legalName")?,
            trim_required(&request.name_ar, "nameAr")?,
            trim_optional(request.name_fr.as_deref()),
            trim_required(&request.activity_description, "activityDescription")?,
            trim_optional(request.legal_form.as_deref()),
            request.social_capital_minor,
            trim_optional(request.tax_identifier.as_deref()),
            trim_optional(request.trade_register_number.as_deref()),
            trim_optional(request.statistical_identifier.as_deref()),
            trim_optional(request.tax_article_number.as_deref()),
            trim_optional(request.bank_rib.as_deref()),
            trim_required(&request.wilaya_code, "wilayaCode")?,
            trim_required(&request.address_text, "addressText")?,
            trim_optional(request.city.as_deref()),
            trim_optional(request.postal_code.as_deref()),
            trim_required(&request.phone, "phone")?,
            trim_optional(request.email.as_deref()),
            timestamp,
            user_id
        ],
    )?;
    Ok(())
}

fn insert_warehouse(
    transaction: &Transaction<'_>,
    request: &InitialSetupRequest,
    company_id: &str,
    warehouse_id: &str,
    user_id: &str,
    timestamp: &str,
) -> Phase05Result<()> {
    transaction.execute(
        r#"
        INSERT INTO warehouses (
            id, company_id, code, name_ar, name_fr, address_text,
            is_default, created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?7, ?8)
        "#,
        params![
            warehouse_id,
            company_id,
            trim_required(&request.warehouse_code, "warehouseCode")?,
            trim_required(&request.warehouse_name_ar, "warehouseNameAr")?,
            trim_optional(request.warehouse_name_fr.as_deref()),
            trim_required(&request.address_text, "addressText")?,
            timestamp,
            user_id
        ],
    )?;
    transaction.execute(
        r#"
        INSERT INTO warehouse_locations (
            id, company_id, warehouse_id, code, name_ar, name_fr,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, ?3, 'DEFAULT', 'الموقع الافتراضي',
                  'Emplacement par défaut', ?4, ?5, ?4, ?5)
        "#,
        params![new_id(), company_id, warehouse_id, timestamp, user_id],
    )?;
    Ok(())
}

fn insert_unit_and_price_list(
    transaction: &Transaction<'_>,
    company_id: &str,
    unit_id: &str,
    price_list_id: &str,
    user_id: &str,
    timestamp: &str,
) -> Phase05Result<()> {
    transaction.execute(
        r#"
        INSERT INTO units (
            id, company_id, code, name_ar, name_fr, decimal_scale,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, 'UN', 'وحدة', 'Unité', 0, ?3, ?4, ?3, ?4)
        "#,
        params![unit_id, company_id, timestamp, user_id],
    )?;
    transaction.execute(
        r#"
        INSERT INTO price_lists (
            id, company_id, code, name_ar, name_fr, price_mode, is_default,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, 'HT', 'الأسعار دون رسم', 'Prix HT', 'HT', 1,
                  ?3, ?4, ?3, ?4)
        "#,
        params![price_list_id, company_id, timestamp, user_id],
    )?;
    Ok(())
}

fn insert_payment_defaults(
    transaction: &Transaction<'_>,
    company_id: &str,
    payment_term_id: &str,
    user_id: &str,
    timestamp: &str,
) -> Phase05Result<()> {
    transaction.execute(
        r#"
        INSERT INTO payment_terms (
            id, company_id, code, name_ar, name_fr, due_days,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, 'IMMEDIATE', 'فوري', 'Immédiat', 0,
                  ?3, ?4, ?3, ?4)
        "#,
        params![payment_term_id, company_id, timestamp, user_id],
    )?;
    for (code, name_ar, name_fr, kind, reference_required) in [
        ("CASH", "نقدي", "Espèces", "CASH", 0_i64),
        ("CARD", "بطاقة", "Carte", "CARD", 1_i64),
        ("CHEQUE", "شيك", "Chèque", "CHEQUE", 1_i64),
        (
            "BANK",
            "تحويل بنكي",
            "Virement bancaire",
            "BANK_TRANSFER",
            1_i64,
        ),
    ] {
        transaction.execute(
            r#"
            INSERT INTO payment_methods (
                id, company_id, code, name_ar, name_fr, method_kind,
                reference_required, created_at, created_by, updated_at, updated_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?8, ?9)
            "#,
            params![
                new_id(),
                company_id,
                code,
                name_ar,
                name_fr,
                kind,
                reference_required,
                timestamp,
                user_id
            ],
        )?;
    }
    Ok(())
}

fn insert_taxes(
    transaction: &Transaction<'_>,
    request: &InitialSetupRequest,
    company_id: &str,
    user_id: &str,
    timestamp: &str,
) -> Phase05Result<Option<String>> {
    let mut ids = HashMap::new();
    for tax in &request.taxes {
        let id = new_id();
        transaction.execute(
            r#"
            INSERT INTO tax_rates (
                id, company_id, code, name_ar, name_fr, rate_scaled,
                valid_from, created_at, created_by, updated_at, updated_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?8, ?9)
            "#,
            params![
                id,
                company_id,
                trim_required(&tax.code, "taxCode")?,
                trim_required(&tax.name_ar, "taxNameAr")?,
                trim_required(&tax.name_fr, "taxNameFr")?,
                tax.rate_scaled,
                request.fiscal_starts_on,
                timestamp,
                user_id
            ],
        )?;
        ids.insert(tax.code.trim().to_lowercase(), id);
    }
    Ok(request
        .default_tax_code
        .as_deref()
        .and_then(|code| ids.get(&code.trim().to_lowercase()))
        .cloned()
        .or_else(|| ids.values().next().cloned()))
}

fn insert_sequences(
    transaction: &Transaction<'_>,
    company_id: &str,
    fiscal_year_id: &str,
    user_id: &str,
    timestamp: &str,
) -> Phase05Result<()> {
    for (document_type, prefix) in DOCUMENT_SEQUENCES {
        transaction.execute(
            r#"
            INSERT INTO document_sequences (
                id, company_id, fiscal_year_id, document_type, prefix,
                next_number, padding_width, created_at, created_by,
                updated_at, updated_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, 1, 6, ?6, ?7, ?6, ?7)
            "#,
            params![
                new_id(),
                company_id,
                fiscal_year_id,
                document_type,
                prefix,
                timestamp,
                user_id
            ],
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_administrator_security(
    transaction: &Transaction<'_>,
    request: &InitialSetupRequest,
    company_id: &str,
    user_id: &str,
    role_id: &str,
    username: &str,
    password_hash: &str,
    recovery_hash: &str,
    timestamp: &str,
) -> Phase05Result<()> {
    for (code, domain, ar, fr, sensitive) in REQUIRED_PERMISSIONS {
        let id = format!("perm-phase05-{}", code.replace('.', "-"));
        transaction.execute(
            r#"
            INSERT OR IGNORE INTO permissions (
                id, code, domain, description_ar, description_fr,
                is_sensitive, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                id,
                code,
                domain,
                ar,
                fr,
                if sensitive { 1_i64 } else { 0_i64 },
                timestamp
            ],
        )?;
    }
    transaction.execute(
        r#"
        INSERT INTO roles (
            id, company_id, code, name_ar, name_fr, is_system, is_active,
            created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, 'SYSTEM_ADMINISTRATOR', 'مدير النظام',
                  'Administrateur système', 1, 1, ?3, ?4, ?3, ?4)
        "#,
        params![role_id, company_id, timestamp, user_id],
    )?;
    transaction.execute(
        r#"
        INSERT INTO users (
            id, company_id, username, display_name, password_hash,
            preferred_language, created_at, created_by, updated_at, updated_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?1, ?7, ?1)
        "#,
        params![
            user_id,
            company_id,
            username,
            trim_required(
                &request.administrator_display_name,
                "administratorDisplayName"
            )?,
            password_hash,
            request.language,
            timestamp
        ],
    )?;
    transaction.execute(
        r#"
        INSERT INTO user_roles (
            id, company_id, user_id, role_id, assigned_at, assigned_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?3)
        "#,
        params![new_id(), company_id, user_id, role_id, timestamp],
    )?;
    let mut statement = transaction.prepare("SELECT id FROM permissions ORDER BY code")?;
    let permission_ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    for permission_id in permission_ids {
        transaction.execute(
            r#"
            INSERT INTO role_permissions (
                id, company_id, role_id, permission_id, granted_at, granted_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                new_id(),
                company_id,
                role_id,
                permission_id,
                timestamp,
                user_id
            ],
        )?;
    }
    transaction.execute(
        r#"
        INSERT INTO user_recovery_codes (
            id, company_id, user_id, code_hash, created_at, created_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?3)
        "#,
        params![new_id(), company_id, user_id, recovery_hash, timestamp],
    )?;
    Ok(())
}

fn insert_setup_audit(
    transaction: &Transaction<'_>,
    company_id: &str,
    user_id: &str,
    correlation_id: &str,
    timestamp: &str,
) -> Phase05Result<()> {
    for (action, entity_type, entity_id) in [
        ("setup.company.create", "companies", company_id),
        ("setup.administrator.create", "users", user_id),
        ("setup.complete", "companies", company_id),
    ] {
        transaction.execute(
            r#"
            INSERT INTO audit_logs (
                id, company_id, actor_user_id, action_code, entity_type,
                entity_id, occurred_at, outcome, correlation_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'SUCCESS', ?8)
            "#,
            params![
                new_id(),
                company_id,
                user_id,
                action,
                entity_type,
                entity_id,
                timestamp,
                correlation_id
            ],
        )?;
    }
    Ok(())
}

fn find_completed_request(
    connection: &rusqlite::Connection,
    key: &str,
) -> Phase05Result<Option<(String, String, String)>> {
    connection
        .query_row(
            r#"
            SELECT r.request_hash_sha256, r.result_company_id, u.id
            FROM initial_setup_requests r
            JOIN users u ON u.company_id=r.result_company_id
            JOIN user_roles ur ON ur.user_id=u.id AND ur.company_id=u.company_id
            JOIN roles role ON role.id=ur.role_id
            WHERE r.idempotency_key=?1 AND r.status='SUCCEEDED'
              AND role.code='SYSTEM_ADMINISTRATOR'
            LIMIT 1
            "#,
            [key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(Phase05Error::from)
}

fn validate_request(request: &InitialSetupRequest) -> Phase05Result<()> {
    for (value, field) in [
        (&request.company_code, "companyCode"),
        (&request.name_ar, "nameAr"),
        (&request.legal_name, "legalName"),
        (&request.activity_description, "activityDescription"),
        (&request.address_text, "addressText"),
        (&request.wilaya_code, "wilayaCode"),
        (&request.phone, "phone"),
        (&request.warehouse_code, "warehouseCode"),
        (&request.warehouse_name_ar, "warehouseNameAr"),
        (&request.administrator_username, "administratorUsername"),
        (
            &request.administrator_display_name,
            "administratorDisplayName",
        ),
    ] {
        trim_required(value, field)?;
    }
    if !matches!(request.language.as_str(), "ar" | "fr")
        || !(0..=1_000_000).contains(&request.default_margin_rate_scaled)
        || !(5..=120).contains(&request.session_idle_timeout_minutes)
        || request.taxes.is_empty()
        || request.taxes.len() > 32
        || request.social_capital_minor.is_some_and(|value| value < 0)
    {
        return Err(Phase05Error::new(
            "SETUP_INVALID_DRAFT",
            "The setup data is incomplete or invalid.",
        ));
    }
    let mut codes = HashSet::new();
    for tax in &request.taxes {
        if !(0..=1_000_000).contains(&tax.rate_scaled)
            || !codes.insert(tax.code.trim().to_lowercase())
        {
            return Err(Phase05Error::new(
                "SETUP_INVALID_DRAFT",
                "The tax configuration contains an invalid or duplicate code.",
            ));
        }
    }
    Ok(())
}

fn deterministic_request_hash(request: &InitialSetupRequest) -> String {
    let mut hasher = Sha256::new();
    for value in [
        request.company_code.as_str(),
        request.name_ar.as_str(),
        request.legal_name.as_str(),
        request.activity_description.as_str(),
        request.address_text.as_str(),
        request.wilaya_code.as_str(),
        request.phone.as_str(),
        request.language.as_str(),
        request.fiscal_starts_on.as_str(),
        request.fiscal_ends_on.as_str(),
        request.administrator_username.as_str(),
        request.administrator_display_name.as_str(),
    ] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hasher.update(request.default_margin_rate_scaled.to_be_bytes());
    hasher.update(request.session_idle_timeout_minutes.to_be_bytes());
    for tax in &request.taxes {
        hasher.update(tax.code.as_bytes());
        hasher.update(tax.name_ar.as_bytes());
        hasher.update(tax.name_fr.as_bytes());
        hasher.update(tax.rate_scaled.to_be_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn setup_already_completed() -> Phase05Error {
    Phase05Error::new(
        "SETUP_ALREADY_COMPLETED",
        "Initial setup has already been completed on this installation.",
    )
}

fn idempotency_conflict() -> Phase05Error {
    Phase05Error::new(
        "SETUP_IDEMPOTENCY_CONFLICT",
        "This setup key was already used for different setup data.",
    )
}
