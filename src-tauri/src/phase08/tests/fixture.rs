fn context(company: &str) -> Phase06AuthContext {
    Phase06AuthContext {
        company_id: company.to_owned(),
        user_id: format!("user-{company}"),
        session_id: format!("session-{company}"),
    }
}

fn fixture() -> Connection {
    let connection = Connection::open_in_memory().expect("open sqlite fixture");
    connection
        .execute_batch("PRAGMA foreign_keys=ON;")
        .expect("enable foreign keys");
    for (index, (name, sql)) in TEST_MIGRATIONS.iter().enumerate() {
        connection.execute_batch(sql).expect("apply ordered migration");
        let checksum = format!("{:x}", Sha256::digest(sql.as_bytes()));
        connection.execute(
            "INSERT INTO app_migrations(id,version,name,checksum_sha256,applied_at) VALUES (?1,?2,?3,?4,?5)",
            params![(index + 1) as i64, format!("{:04}", index + 1), name, checksum, NOW],
        ).expect("record ordered migration");
    }
    for (id, code) in [
        ("perm-receipt-allocate", "payment.receipt.allocate"),
        ("perm-disbursement-allocate", "payment.disbursement.allocate"),
    ] {
        connection.execute(
            "INSERT INTO permissions(id,code,domain,description_ar,description_fr,is_sensitive,created_at) VALUES (?1,?2,'payments',?2,?2,0,?3)",
            params![id, code, NOW],
        ).unwrap();
    }
    for company in ["company-1", "company-2"] {
        let user = format!("user-{company}");
        let fiscal_year = format!("fy-{company}");
        let period = format!("period-{company}");
        connection.execute(
            "INSERT INTO companies(id,code,legal_name,name_ar,name_fr,created_at,updated_at) VALUES (?1,?2,?2,?2,?2,?3,?3)",
            params![company, company, NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO fiscal_years(id,company_id,code,starts_on,ends_on,status,created_at,updated_at) VALUES (?1,?2,'2026','2026-01-01','2026-12-31','OPEN',?3,?3)",
            params![fiscal_year, company, NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO fiscal_periods(id,company_id,fiscal_year_id,period_number,name,starts_on,ends_on,status,created_at,updated_at) VALUES (?1,?2,?3,1,'2026','2026-01-01','2026-12-31','OPEN',?4,?4)",
            params![period, company, fiscal_year, NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO users(id,company_id,username,display_name,password_hash,is_active,created_at,updated_at) VALUES (?1,?2,?1,?1,'01234567890123456789',1,?3,?3)",
            params![user, company, NOW],
        ).unwrap();
        let role = format!("role-{company}");
        connection.execute(
            "INSERT INTO roles(id,company_id,code,name_ar,name_fr,is_system,is_active,created_at,updated_at) VALUES (?1,?2,'OWNER','مالك','Propriétaire',1,1,?3,?3)",
            params![role, company, NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO user_roles(id,company_id,user_id,role_id,assigned_at) VALUES (?1,?2,?3,?4,?5)",
            params![format!("ur-{company}"), company, user, role, NOW],
        ).unwrap();
        for permission in ["perm-receipt-allocate", "perm-disbursement-allocate"] {
            connection.execute(
                "INSERT INTO role_permissions(id,company_id,role_id,permission_id,granted_at) VALUES (?1,?2,?3,?4,?5)",
                params![format!("rp-{company}-{permission}"), company, role, permission, NOW],
            ).unwrap();
        }
        connection.execute(
            "INSERT INTO sessions(id,company_id,user_id,token_hash,created_at,expires_at,last_seen_at) VALUES (?1,?2,?3,?4,?5,'2099-12-31T23:59:59Z',?5)",
            params![format!("session-{company}"), company, user, format!("token-{company}-012345678901234567890123456789"), NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO units(id,company_id,code,name_ar,name_fr,decimal_scale,created_at,updated_at) VALUES (?1,?2,'PC','قطعة','Pièce',6,?3,?3)",
            params![format!("unit-{company}"), company, NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO warehouses(id,company_id,code,name_ar,name_fr,is_active,created_at,updated_at) VALUES (?1,?2,'MAIN','المخزن','Dépôt',1,?3,?3)",
            params![format!("warehouse-{company}"), company, NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO products(id,company_id,unit_id,code,name_ar,name_fr,product_kind,stock_tracked,is_active,created_at,updated_at) VALUES (?1,?2,?3,'ITEM','صنف','Article','STOCK_ITEM',1,1,?4,?4)",
            params![format!("product-{company}"), company, format!("unit-{company}"), NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO accounting_setups(company_id,is_enabled,current_fiscal_year_id,created_at,created_by,updated_at,updated_by,row_version) VALUES (?1,1,?2,?3,?4,?3,?4,1)",
            params![company, fiscal_year, NOW, user],
        ).unwrap();
        connection.execute(
            "INSERT INTO accounting_journals(id,company_id,code,name_ar,name_fr,journal_type,is_active,created_at,updated_at) VALUES (?1,?2,'GJ','يومية','Journal','GENERAL',1,?3,?3)",
            params![format!("journal-{company}"), company, NOW],
        ).unwrap();
        let accounts = [
            ("CUSTOMER_RECEIVABLE", "ASSET", "DEBIT"),
            ("SUPPLIER_PAYABLE", "LIABILITY", "CREDIT"),
            ("CASH", "ASSET", "DEBIT"),
            ("BANK", "ASSET", "DEBIT"),
            ("SALES_REVENUE", "REVENUE", "CREDIT"),
            ("COLLECTED_TAX", "LIABILITY", "CREDIT"),
            ("RECOVERABLE_TAX", "ASSET", "DEBIT"),
            ("PURCHASE_RETURNS", "EXPENSE", "DEBIT"),
            ("INVENTORY", "ASSET", "DEBIT"),
            ("COGS", "EXPENSE", "DEBIT"),
        ];
        for (index, (role, kind, side)) in accounts.into_iter().enumerate() {
            let account = format!("account-{company}-{role}");
            connection.execute(
                "INSERT INTO accounts(id,company_id,code,name_ar,name_fr,account_type,normal_side,allow_posting,is_active,created_at,updated_at) VALUES (?1,?2,?3,?3,?3,?4,?5,1,1,?6,?6)",
                params![account, company, format!("{:02}-{role}", index + 1), kind, side, NOW],
            ).unwrap();
            connection.execute(
                "INSERT INTO accounting_account_roles(id,company_id,role_code,account_id,created_at,created_by,updated_at,updated_by,row_version) VALUES (?1,?2,?3,?4,?5,?6,?5,?6,1)",
                params![format!("role-{company}-{role}"), company, role, account, NOW, user],
            ).unwrap();
        }
        connection.execute(
            "INSERT INTO payment_methods(id,company_id,code,name_ar,name_fr,method_kind,reference_required,is_active,created_at,updated_at) VALUES (?1,?2,'CASH','نقد','Espèces','CASH',0,1,?3,?3)",
            params![format!("method-{company}"), company, NOW],
        ).unwrap();
        connection.execute(
            "INSERT INTO payment_method_accounting(id,company_id,payment_method_id,account_id,account_role_code,created_at,created_by,updated_at,updated_by,row_version) VALUES (?1,?2,?3,NULL,'CASH',?4,?5,?4,?5,1)",
            params![format!("method-map-{company}"), company, format!("method-{company}"), NOW, user],
        ).unwrap();
        connection.execute(
            "INSERT INTO partners(id,company_id,code,legal_name,display_name_ar,display_name_fr,is_customer,is_supplier,is_active,created_at,updated_at) VALUES (?1,?2,'P1','Partner','شريك','Partenaire',1,1,1,?3,?3)",
            params![format!("partner-{company}"), company, NOW],
        ).unwrap();
    }
    add_rule(&connection, "company-1", "SALES_INVOICE", &[("DEBIT", "CUSTOMER_RECEIVABLE", "DOCUMENT_TTC"), ("CREDIT", "SALES_REVENUE", "DOCUMENT_HT"), ("CREDIT", "COLLECTED_TAX", "DOCUMENT_TAX")]);
    add_rule(&connection, "company-1", "PURCHASE_INVOICE", &[("DEBIT", "INVENTORY", "DOCUMENT_HT"), ("DEBIT", "RECOVERABLE_TAX", "DOCUMENT_TAX"), ("CREDIT", "SUPPLIER_PAYABLE", "DOCUMENT_TTC")]);
    add_rule(&connection, "company-1", "PURCHASE_RECEIVE_INVOICE", &[("DEBIT", "INVENTORY", "DOCUMENT_HT"), ("DEBIT", "RECOVERABLE_TAX", "DOCUMENT_TAX"), ("CREDIT", "SUPPLIER_PAYABLE", "DOCUMENT_TTC")]);
    add_rule(&connection, "company-1", "DELIVERY_COGS", &[("DEBIT", "COGS", "STOCK_COST"), ("CREDIT", "INVENTORY", "STOCK_COST")]);
    add_rule(&connection, "company-1", "DIRECT_SALE", &[("DEBIT", "CUSTOMER_RECEIVABLE", "DOCUMENT_TTC"), ("CREDIT", "SALES_REVENUE", "DOCUMENT_HT"), ("CREDIT", "COLLECTED_TAX", "DOCUMENT_TAX"), ("DEBIT", "COGS", "STOCK_COST"), ("CREDIT", "INVENTORY", "STOCK_COST")]);
    add_rule(&connection, "company-1", "SALES_RETURN", &[("DEBIT", "SALES_REVENUE", "DOCUMENT_HT"), ("DEBIT", "COLLECTED_TAX", "DOCUMENT_TAX"), ("CREDIT", "CUSTOMER_RECEIVABLE", "DOCUMENT_TTC"), ("DEBIT", "INVENTORY", "STOCK_COST"), ("CREDIT", "COGS", "STOCK_COST")]);
    add_rule(&connection, "company-1", "PURCHASE_RETURN", &[("DEBIT", "SUPPLIER_PAYABLE", "DOCUMENT_TTC"), ("CREDIT", "INVENTORY", "STOCK_COST"), ("CREDIT", "RECOVERABLE_TAX", "DOCUMENT_TAX")]);
    add_rule(&connection, "company-1", "CUSTOMER_RECEIPT", &[("DEBIT", "CASH", "PAYMENT_AMOUNT"), ("CREDIT", "CUSTOMER_RECEIVABLE", "PAYMENT_AMOUNT")]);
    add_rule(&connection, "company-1", "SUPPLIER_PAYMENT", &[("DEBIT", "SUPPLIER_PAYABLE", "PAYMENT_AMOUNT"), ("CREDIT", "CASH", "PAYMENT_AMOUNT")]);
    add_rule(&connection, "company-1", "CUSTOMER_RECEIPT_REVERSAL", &[("DEBIT", "CUSTOMER_RECEIVABLE", "PAYMENT_AMOUNT"), ("CREDIT", "CASH", "PAYMENT_AMOUNT")]);
    add_rule(&connection, "company-1", "SUPPLIER_PAYMENT_REVERSAL", &[("DEBIT", "CASH", "PAYMENT_AMOUNT"), ("CREDIT", "SUPPLIER_PAYABLE", "PAYMENT_AMOUNT")]);
    connection
}
