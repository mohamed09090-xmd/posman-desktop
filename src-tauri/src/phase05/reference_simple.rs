use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::json;

use super::{
    dto::{Page, PageRequest, ReferenceInput, ReferenceRecord, ReferenceUpdate, SetActiveRequest},
    error::{Phase05Error, Phase05Result},
    state::{audit, new_id, now_iso, trim_optional, trim_required, Phase05Service},
};

#[derive(Clone, Copy)]
enum ReferenceKind {
    Unit,
    TaxRate,
    PaymentTerm,
    PaymentMethod,
    Warehouse,
    WarehouseLocation,
    ProductFamily,
}

impl ReferenceKind {
    fn table(self) -> &'static str {
        match self {
            Self::Unit => "units",
            Self::TaxRate => "tax_rates",
            Self::PaymentTerm => "payment_terms",
            Self::PaymentMethod => "payment_methods",
            Self::Warehouse => "warehouses",
            Self::WarehouseLocation => "warehouse_locations",
            Self::ProductFamily => "product_families",
        }
    }

    fn manage_permission(self) -> &'static str {
        match self {
            Self::ProductFamily => "catalog.manage",
            _ => "settings.manage",
        }
    }

    fn view_permission(self) -> &'static str {
        match self {
            Self::ProductFamily => "catalog.view",
            _ => "settings.manage",
        }
    }
}

macro_rules! reference_methods {
    ($list:ident, $create:ident, $update:ident, $active:ident, $kind:expr) => {
        pub fn $list(&self, page: PageRequest) -> Phase05Result<Page<ReferenceRecord>> {
            self.list_reference($kind, page)
        }
        pub fn $create(&self, input: ReferenceInput) -> Phase05Result<ReferenceRecord> {
            self.create_reference($kind, input)
        }
        pub fn $update(&self, input: ReferenceUpdate) -> Phase05Result<ReferenceRecord> {
            self.update_reference($kind, input)
        }
        pub fn $active(&self, input: SetActiveRequest) -> Phase05Result<ReferenceRecord> {
            self.set_reference_active($kind, input)
        }
    };
}

impl Phase05Service {
    reference_methods!(list_units, create_unit, update_unit, set_unit_active, ReferenceKind::Unit);
    reference_methods!(list_tax_rates, create_tax_rate, update_tax_rate, set_tax_rate_active, ReferenceKind::TaxRate);
    reference_methods!(list_payment_terms, create_payment_term, update_payment_term, set_payment_term_active, ReferenceKind::PaymentTerm);
    reference_methods!(list_payment_methods, create_payment_method, update_payment_method, set_payment_method_active, ReferenceKind::PaymentMethod);
    reference_methods!(list_warehouses, create_warehouse, update_warehouse, set_warehouse_active, ReferenceKind::Warehouse);
    reference_methods!(list_warehouse_locations, create_warehouse_location, update_warehouse_location, set_warehouse_location_active, ReferenceKind::WarehouseLocation);
    reference_methods!(list_product_families, create_product_family, update_product_family, set_product_family_active, ReferenceKind::ProductFamily);

    fn list_reference(&self, kind: ReferenceKind, request: PageRequest) -> Phase05Result<Page<ReferenceRecord>> {
        let context = self.require_session(Some(kind.view_permission()))?;
        let (page, page_size, offset, search) = pagination(request)?;
        let pattern = format!("%{search}%");
        let table = kind.table();
        let connection = self.open()?;
        let count_sql = format!("SELECT COUNT(*) FROM {table} WHERE company_id=?1 AND (?2='' OR lower(code) LIKE ?3 OR lower(name_ar) LIKE ?3 OR lower(COALESCE(name_fr,'')) LIKE ?3)");
        let total: i64 = connection.query_row(&count_sql, params![context.company_id, search, pattern], |row| row.get(0))?;
        let select_sql = format!("SELECT id, code, name_ar, name_fr, is_active, row_version FROM {table} WHERE company_id=?1 AND (?2='' OR lower(code) LIKE ?3 OR lower(name_ar) LIKE ?3 OR lower(COALESCE(name_fr,'')) LIKE ?3) ORDER BY is_active DESC, code LIMIT ?4 OFFSET ?5");
        let mut statement = connection.prepare(&select_sql)?;
        let rows = statement.query_map(params![context.company_id, search, pattern, page_size, offset], |row| {
            Ok(ReferenceRecord { id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_fr: row.get(3)?, is_active: row.get::<_, i64>(4)? == 1, row_version: row.get(5)?, details: json!({}) })
        })?;
        let items = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(Page { items, page, page_size, total: u64::try_from(total).unwrap_or(0) })
    }

    fn create_reference(&self, kind: ReferenceKind, input: ReferenceInput) -> Phase05Result<ReferenceRecord> {
        validate_reference(kind, &input)?;
        let context = self.require_session(Some(kind.manage_permission()))?;
        let id = new_id();
        let timestamp = now_iso()?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        insert_reference(&transaction, kind, &context.company_id, &context.user_id, &id, &input, &timestamp)?;
        audit(&transaction, &context, &format!("{}.create", kind.table()), kind.table(), &id, None)?;
        transaction.commit()?;
        self.get_reference(kind, &id)
    }

    fn update_reference(&self, kind: ReferenceKind, input: ReferenceUpdate) -> Phase05Result<ReferenceRecord> {
        let shape = ReferenceInput { code: input.code.clone(), name_ar: input.name_ar.clone(), name_fr: input.name_fr.clone(), numeric_value: input.numeric_value, secondary_numeric_value: input.secondary_numeric_value, kind: input.kind.clone(), parent_id: input.parent_id.clone(), related_id: input.related_id.clone(), address_text: input.address_text.clone(), flag: input.flag };
        validate_reference(kind, &shape)?;
        let context = self.require_session(Some(kind.manage_permission()))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let changed = update_reference_row(&transaction, kind, &context.company_id, &context.user_id, &input, &now_iso()?)?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        audit(&transaction, &context, &format!("{}.update", kind.table()), kind.table(), &input.id, None)?;
        transaction.commit()?;
        self.get_reference(kind, &input.id)
    }

    fn set_reference_active(&self, kind: ReferenceKind, input: SetActiveRequest) -> Phase05Result<ReferenceRecord> {
        let context = self.require_session(Some(kind.manage_permission()))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let sql = format!("UPDATE {} SET is_active=?1, updated_at=?2, updated_by=?3, row_version=row_version+1 WHERE id=?4 AND company_id=?5 AND row_version=?6", kind.table());
        let changed = transaction.execute(&sql, params![if input.is_active {1_i64} else {0_i64}, now_iso()?, context.user_id, input.id, context.company_id, input.row_version])?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        audit(&transaction, &context, &format!("{}.set_active", kind.table()), kind.table(), &input.id, None)?;
        transaction.commit()?;
        self.get_reference(kind, &input.id)
    }

    fn get_reference(&self, kind: ReferenceKind, id: &str) -> Phase05Result<ReferenceRecord> {
        let context = self.require_session(Some(kind.view_permission()))?;
        let sql = format!("SELECT id, code, name_ar, name_fr, is_active, row_version FROM {} WHERE id=?1 AND company_id=?2", kind.table());
        self.open()?.query_row(&sql, params![id, context.company_id], |row| {
            Ok(ReferenceRecord { id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_fr: row.get(3)?, is_active: row.get::<_, i64>(4)? == 1, row_version: row.get(5)?, details: json!({}) })
        }).optional()?.ok_or_else(|| Phase05Error::new("NOT_FOUND", "The record was not found."))
    }
}

fn insert_reference(transaction: &Transaction<'_>, kind: ReferenceKind, company_id: &str, user_id: &str, id: &str, input: &ReferenceInput, timestamp: &str) -> Phase05Result<()> {
    let code = trim_required(&input.code, "code")?;
    let name_ar = trim_required(&input.name_ar, "nameAr")?;
    let name_fr = trim_optional(input.name_fr.as_deref());
    match kind {
        ReferenceKind::Unit => { transaction.execute(r#"INSERT INTO units (id, company_id, code, name_ar, name_fr, decimal_scale, created_at, created_by, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)"#, params![id, company_id, code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.numeric_value.unwrap_or(0), timestamp, user_id])?; }
        ReferenceKind::TaxRate => { transaction.execute(r#"INSERT INTO tax_rates (id, company_id, code, name_ar, name_fr, rate_scaled, valid_from, valid_to, created_at, created_by, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?9, ?10)"#, params![id, company_id, code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.numeric_value.unwrap_or(0), input.kind.as_deref().unwrap_or("2000-01-01"), input.address_text, timestamp, user_id])?; }
        ReferenceKind::PaymentTerm => { transaction.execute(r#"INSERT INTO payment_terms (id, company_id, code, name_ar, name_fr, due_days, created_at, created_by, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)"#, params![id, company_id, code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.numeric_value.unwrap_or(0), timestamp, user_id])?; }
        ReferenceKind::PaymentMethod => { transaction.execute(r#"INSERT INTO payment_methods (id, company_id, code, name_ar, name_fr, method_kind, reference_required, created_at, created_by, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?8, ?9)"#, params![id, company_id, code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.kind.as_deref().unwrap_or("OTHER"), if input.flag.unwrap_or(false) {1_i64} else {0_i64}, timestamp, user_id])?; }
        ReferenceKind::Warehouse => { transaction.execute(r#"INSERT INTO warehouses (id, company_id, code, name_ar, name_fr, address_text, is_default, created_at, created_by, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?8, ?9)"#, params![id, company_id, code, name_ar, name_fr, trim_optional(input.address_text.as_deref()), if input.flag.unwrap_or(false) {1_i64} else {0_i64}, timestamp, user_id])?; }
        ReferenceKind::WarehouseLocation => { transaction.execute(r#"INSERT INTO warehouse_locations (id, company_id, warehouse_id, code, name_ar, name_fr, created_at, created_by, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)"#, params![id, company_id, input.parent_id.as_deref().ok_or_else(|| Phase05Error::invalid("warehouseId"))?, code, name_ar, name_fr, timestamp, user_id])?; }
        ReferenceKind::ProductFamily => { transaction.execute(r#"INSERT INTO product_families (id, company_id, parent_family_id, default_tax_rate_id, code, name_ar, name_fr, default_margin_rate_scaled, created_at, created_by, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?9, ?10)"#, params![id, company_id, input.parent_id, input.related_id, code, name_ar, name_fr, input.numeric_value, timestamp, user_id])?; }
    }
    Ok(())
}

fn update_reference_row(transaction: &Transaction<'_>, kind: ReferenceKind, company_id: &str, user_id: &str, input: &ReferenceUpdate, timestamp: &str) -> Phase05Result<usize> {
    let code = trim_required(&input.code, "code")?;
    let name_ar = trim_required(&input.name_ar, "nameAr")?;
    let name_fr = trim_optional(input.name_fr.as_deref());
    let changed = match kind {
        ReferenceKind::Unit => transaction.execute("UPDATE units SET code=?1,name_ar=?2,name_fr=?3,decimal_scale=?4,updated_at=?5,updated_by=?6,row_version=row_version+1 WHERE id=?7 AND company_id=?8 AND row_version=?9", params![code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.numeric_value.unwrap_or(0), timestamp, user_id, input.id, company_id, input.row_version])?,
        ReferenceKind::TaxRate => transaction.execute("UPDATE tax_rates SET code=?1,name_ar=?2,name_fr=?3,rate_scaled=?4,valid_from=?5,valid_to=?6,updated_at=?7,updated_by=?8,row_version=row_version+1 WHERE id=?9 AND company_id=?10 AND row_version=?11", params![code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.numeric_value.unwrap_or(0), input.kind.as_deref().unwrap_or("2000-01-01"), input.address_text, timestamp, user_id, input.id, company_id, input.row_version])?,
        ReferenceKind::PaymentTerm => transaction.execute("UPDATE payment_terms SET code=?1,name_ar=?2,name_fr=?3,due_days=?4,updated_at=?5,updated_by=?6,row_version=row_version+1 WHERE id=?7 AND company_id=?8 AND row_version=?9", params![code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.numeric_value.unwrap_or(0), timestamp, user_id, input.id, company_id, input.row_version])?,
        ReferenceKind::PaymentMethod => transaction.execute("UPDATE payment_methods SET code=?1,name_ar=?2,name_fr=?3,method_kind=?4,reference_required=?5,updated_at=?6,updated_by=?7,row_version=row_version+1 WHERE id=?8 AND company_id=?9 AND row_version=?10", params![code, name_ar, name_fr.unwrap_or_else(|| name_ar.clone()), input.kind.as_deref().unwrap_or("OTHER"), if input.flag.unwrap_or(false) {1_i64} else {0_i64}, timestamp, user_id, input.id, company_id, input.row_version])?,
        ReferenceKind::Warehouse => transaction.execute("UPDATE warehouses SET code=?1,name_ar=?2,name_fr=?3,address_text=?4,is_default=?5,updated_at=?6,updated_by=?7,row_version=row_version+1 WHERE id=?8 AND company_id=?9 AND row_version=?10", params![code, name_ar, name_fr, trim_optional(input.address_text.as_deref()), if input.flag.unwrap_or(false) {1_i64} else {0_i64}, timestamp, user_id, input.id, company_id, input.row_version])?,
        ReferenceKind::WarehouseLocation => transaction.execute("UPDATE warehouse_locations SET warehouse_id=?1,code=?2,name_ar=?3,name_fr=?4,updated_at=?5,updated_by=?6,row_version=row_version+1 WHERE id=?7 AND company_id=?8 AND row_version=?9", params![input.parent_id.as_deref().ok_or_else(|| Phase05Error::invalid("warehouseId"))?, code, name_ar, name_fr, timestamp, user_id, input.id, company_id, input.row_version])?,
        ReferenceKind::ProductFamily => transaction.execute("UPDATE product_families SET parent_family_id=?1,default_tax_rate_id=?2,code=?3,name_ar=?4,name_fr=?5,default_margin_rate_scaled=?6,updated_at=?7,updated_by=?8,row_version=row_version+1 WHERE id=?9 AND company_id=?10 AND row_version=?11", params![input.parent_id, input.related_id, code, name_ar, name_fr, input.numeric_value, timestamp, user_id, input.id, company_id, input.row_version])?,
    };
    Ok(changed)
}

fn validate_reference(kind: ReferenceKind, input: &ReferenceInput) -> Phase05Result<()> {
    trim_required(&input.code, "code")?;
    trim_required(&input.name_ar, "nameAr")?;
    match kind {
        ReferenceKind::Unit if !(0..=6).contains(&input.numeric_value.unwrap_or(0)) => Err(Phase05Error::invalid("decimalScale")),
        ReferenceKind::TaxRate if !(0..=1_000_000).contains(&input.numeric_value.unwrap_or(0)) => Err(Phase05Error::invalid("taxRate")),
        ReferenceKind::PaymentTerm if !(0..=3650).contains(&input.numeric_value.unwrap_or(0)) => Err(Phase05Error::invalid("dueDays")),
        ReferenceKind::PaymentMethod if !matches!(input.kind.as_deref(), Some("CASH" | "CARD" | "CHEQUE" | "BANK_TRANSFER" | "OTHER")) => Err(Phase05Error::invalid("methodKind")),
        ReferenceKind::WarehouseLocation if input.parent_id.is_none() => Err(Phase05Error::invalid("warehouseId")),
        ReferenceKind::ProductFamily if input.numeric_value.is_some_and(|value| !(0..=1_000_000).contains(&value)) => Err(Phase05Error::invalid("marginRate")),
        _ => Ok(()),
    }
}

fn pagination(request: PageRequest) -> Phase05Result<(u32, u32, i64, String)> {
    let page = request.page.unwrap_or(1).max(1);
    let page_size = request.page_size.unwrap_or(25).clamp(1, 100);
    let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
    let search = request.search.unwrap_or_default().trim().to_lowercase();
    if search.chars().count() > 100 { return Err(Phase05Error::invalid("search")); }
    Ok((page, page_size, offset, search))
}
