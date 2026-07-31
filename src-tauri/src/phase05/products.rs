use rusqlite::{params, OptionalExtension};

use super::{
    dto::{
        CreateProductRequest, Page, PageRequest, ProductPriceInput, ProductView,
        SetActiveRequest, UpdateProductRequest,
    },
    error::{Phase05Error, Phase05Result},
    pricing::calculate_pricing,
    state::{audit, new_id, now_iso, trim_optional, trim_required, Phase05Service},
};

impl Phase05Service {
    pub fn list_products(&self, request: PageRequest) -> Phase05Result<Page<ProductView>> {
        let context = self.require_session(Some("catalog.view"))?;
        let page = request.page.unwrap_or(1).max(1);
        let page_size = request.page_size.unwrap_or(25).clamp(1, 100);
        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        let search = request.search.unwrap_or_default().trim().to_lowercase();
        if search.chars().count() > 100 { return Err(Phase05Error::invalid("search")); }
        let pattern = format!("%{search}%");
        let connection = self.open()?;
        let total: i64 = connection.query_row(
            r#"
            SELECT COUNT(*) FROM products WHERE company_id=?1 AND (
                ?2='' OR lower(code) LIKE ?3 OR lower(name_ar) LIKE ?3
                OR lower(COALESCE(name_fr,'')) LIKE ?3
                OR lower(COALESCE(barcode,'')) LIKE ?3)
            "#,
            params![context.company_id, search, pattern], |row| row.get(0))?;
        let mut statement = connection.prepare(
            r#"
            SELECT id, code, name_ar, name_fr, unit_id, product_family_id,
                   default_tax_rate_id, COALESCE(default_purchase_price_scaled,0),
                   COALESCE(default_sale_price_scaled,0), is_active, row_version
            FROM products WHERE company_id=?1 AND (
                ?2='' OR lower(code) LIKE ?3 OR lower(name_ar) LIKE ?3
                OR lower(COALESCE(name_fr,'')) LIKE ?3
                OR lower(COALESCE(barcode,'')) LIKE ?3)
            ORDER BY is_active DESC, code LIMIT ?4 OFFSET ?5
            "#,
        )?;
        let rows = statement.query_map(params![context.company_id, search, pattern, page_size, offset], |row| {
            let sale: i64 = row.get(8)?;
            Ok(ProductView { id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_fr: row.get(3)?, unit_id: row.get(4)?, product_family_id: row.get(5)?, tax_rate_id: row.get(6)?, purchase_price_scaled: row.get(7)?, sale_price_scaled: sale, suggested_sale_price_scaled: sale, is_active: row.get::<_, i64>(9)? == 1, row_version: row.get(10)? })
        })?;
        let items = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(Page { items, page, page_size, total: u64::try_from(total).unwrap_or(0) })
    }

    pub fn create_product(&self, request: CreateProductRequest) -> Phase05Result<ProductView> {
        validate_product(&request.code, &request.name_ar, &request.product_kind, request.default_purchase_price_scaled, request.manual_sale_price_scaled, request.margin_rate_scaled, request.minimum_stock_scaled)?;
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        let defaults = resolve_product_defaults(&connection, &context.company_id, request.product_family_id.as_deref(), request.default_tax_rate_id.as_deref(), request.margin_rate_scaled)?;
        ensure_company_reference(&connection, "units", &request.unit_id, &context.company_id)?;
        let suggested = calculate_pricing(request.default_purchase_price_scaled, defaults.margin_rate_scaled, 0, 0)?.sale_ht_scaled;
        let sale_price = request.manual_sale_price_scaled.unwrap_or(suggested);
        let stock_tracked = request.product_kind == "STOCK_ITEM";
        let product_id = new_id();
        let timestamp = now_iso()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"
            INSERT INTO products (
                id, company_id, product_family_id, unit_id, default_tax_rate_id,
                code, barcode, name_ar, name_fr, product_kind, stock_tracked,
                minimum_stock_scaled, default_purchase_price_scaled,
                default_sale_price_scaled, created_at, created_by, updated_at, updated_by)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?15,?16)
            "#,
            params![product_id, context.company_id, request.product_family_id, request.unit_id, defaults.tax_rate_id, trim_required(&request.code, "code")?, trim_optional(request.barcode.as_deref()), trim_required(&request.name_ar, "nameAr")?, trim_optional(request.name_fr.as_deref()), request.product_kind, if stock_tracked {1_i64} else {0_i64}, request.minimum_stock_scaled, request.default_purchase_price_scaled, sale_price, timestamp, context.user_id],
        )?;
        let price_list_id: String = transaction.query_row("SELECT id FROM price_lists WHERE company_id=?1 AND is_default=1 AND is_active=1 LIMIT 1", [context.company_id.as_str()], |row| row.get(0))?;
        transaction.execute(
            r#"
            INSERT INTO product_prices (id,company_id,price_list_id,product_id,
                unit_price_scaled,valid_from,created_at,created_by,updated_at,updated_by)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?7,?8)
            "#,
            params![new_id(), context.company_id, price_list_id, product_id, sale_price, current_date_text(), timestamp, context.user_id],
        )?;
        audit(&transaction, &context, "catalog.product.create", "products", &product_id, Some(&format!("{{\"suggestedSalePriceScaled\":{suggested}}}")))?;
        transaction.commit()?;
        self.get_product(&product_id, suggested)
    }

    pub fn update_product(&self, request: UpdateProductRequest) -> Phase05Result<ProductView> {
        validate_product(&request.code, &request.name_ar, &request.product_kind, request.default_purchase_price_scaled, request.manual_sale_price_scaled, request.margin_rate_scaled, request.minimum_stock_scaled)?;
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        let defaults = resolve_product_defaults(&connection, &context.company_id, request.product_family_id.as_deref(), request.default_tax_rate_id.as_deref(), request.margin_rate_scaled)?;
        ensure_company_reference(&connection, "units", &request.unit_id, &context.company_id)?;
        let suggested = calculate_pricing(request.default_purchase_price_scaled, defaults.margin_rate_scaled, 0, 0)?.sale_ht_scaled;
        let sale_price = request.manual_sale_price_scaled.unwrap_or(suggested);
        let stock_tracked = request.product_kind == "STOCK_ITEM";
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            r#"
            UPDATE products SET product_family_id=?1,unit_id=?2,default_tax_rate_id=?3,
                code=?4,barcode=?5,name_ar=?6,name_fr=?7,product_kind=?8,
                stock_tracked=?9,minimum_stock_scaled=?10,
                default_purchase_price_scaled=?11,default_sale_price_scaled=?12,
                updated_at=?13,updated_by=?14,row_version=row_version+1
            WHERE id=?15 AND company_id=?16 AND row_version=?17
            "#,
            params![request.product_family_id, request.unit_id, defaults.tax_rate_id, trim_required(&request.code, "code")?, trim_optional(request.barcode.as_deref()), trim_required(&request.name_ar, "nameAr")?, trim_optional(request.name_fr.as_deref()), request.product_kind, if stock_tracked {1_i64} else {0_i64}, request.minimum_stock_scaled, request.default_purchase_price_scaled, sale_price, now_iso()?, context.user_id, request.id, context.company_id, request.row_version],
        )?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        let price_list_id: String = transaction.query_row("SELECT id FROM price_lists WHERE company_id=?1 AND is_default=1 AND is_active=1 LIMIT 1", [context.company_id.as_str()], |row| row.get(0))?;
        let current_price_id: Option<String> = transaction.query_row("SELECT id FROM product_prices WHERE company_id=?1 AND product_id=?2 AND price_list_id=?3 AND valid_to IS NULL ORDER BY valid_from DESC LIMIT 1", params![context.company_id, request.id, price_list_id], |row| row.get(0)).optional()?;
        if let Some(price_id) = current_price_id {
            transaction.execute("UPDATE product_prices SET unit_price_scaled=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5", params![sale_price, now_iso()?, context.user_id, price_id, context.company_id])?;
        } else {
            transaction.execute("INSERT INTO product_prices (id,company_id,price_list_id,product_id,unit_price_scaled,valid_from,created_at,created_by,updated_at,updated_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?7,?8)", params![new_id(), context.company_id, price_list_id, request.id, sale_price, current_date_text(), now_iso()?, context.user_id])?;
        }
        audit(&transaction, &context, "catalog.product.update", "products", &request.id, Some(&format!("{{\"suggestedSalePriceScaled\":{suggested}}}")))?;
        transaction.commit()?;
        self.get_product(&request.id, suggested)
    }

    pub fn set_product_active(&self, request: SetActiveRequest) -> Phase05Result<ProductView> {
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute("UPDATE products SET is_active=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5 AND row_version=?6", params![if request.is_active {1_i64} else {0_i64}, now_iso()?, context.user_id, request.id, context.company_id, request.row_version])?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        audit(&transaction, &context, "catalog.product.set_active", "products", &request.id, None)?;
        transaction.commit()?;
        self.get_product(&request.id, 0)
    }

    pub fn set_product_price(&self, request: ProductPriceInput) -> Phase05Result<()> {
        if request.unit_price_scaled < 0 || request.valid_from.len() != 10 { return Err(Phase05Error::invalid("productPrice")); }
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        ensure_company_reference(&connection, "products", &request.product_id, &context.company_id)?;
        ensure_company_reference(&connection, "price_lists", &request.price_list_id, &context.company_id)?;
        let transaction = connection.transaction()?;
        transaction.execute("UPDATE product_prices SET valid_to=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE company_id=?4 AND product_id=?5 AND price_list_id=?6 AND valid_to IS NULL AND valid_from<?7", params![request.valid_from, now_iso()?, context.user_id, context.company_id, request.product_id, request.price_list_id, request.valid_from])?;
        transaction.execute(
            r#"
            INSERT INTO product_prices (id,company_id,price_list_id,product_id,
                unit_price_scaled,valid_from,created_at,created_by,updated_at,updated_by)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?7,?8)
            ON CONFLICT(company_id,price_list_id,product_id,valid_from)
            DO UPDATE SET unit_price_scaled=excluded.unit_price_scaled,valid_to=NULL,
                updated_at=excluded.updated_at,updated_by=excluded.updated_by,
                row_version=product_prices.row_version+1
            "#,
            params![new_id(), context.company_id, request.price_list_id, request.product_id, request.unit_price_scaled, request.valid_from, now_iso()?, context.user_id],
        )?;
        transaction.execute("UPDATE products SET default_sale_price_scaled=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5", params![request.unit_price_scaled, now_iso()?, context.user_id, request.product_id, context.company_id])?;
        audit(&transaction, &context, "catalog.product_price.set", "product_prices", &request.product_id, None)?;
        transaction.commit()?;
        Ok(())
    }

    fn get_product(&self, id: &str, suggested: i64) -> Phase05Result<ProductView> {
        let context = self.require_session(Some("catalog.view"))?;
        self.open()?.query_row(
            "SELECT id,code,name_ar,name_fr,unit_id,product_family_id,default_tax_rate_id,COALESCE(default_purchase_price_scaled,0),COALESCE(default_sale_price_scaled,0),is_active,row_version FROM products WHERE id=?1 AND company_id=?2",
            params![id, context.company_id], |row| Ok(ProductView { id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_fr: row.get(3)?, unit_id: row.get(4)?, product_family_id: row.get(5)?, tax_rate_id: row.get(6)?, purchase_price_scaled: row.get(7)?, sale_price_scaled: row.get(8)?, suggested_sale_price_scaled: suggested, is_active: row.get::<_, i64>(9)? == 1, row_version: row.get(10)? }))
            .optional()?.ok_or_else(|| Phase05Error::new("NOT_FOUND", "The product was not found."))
    }
}

struct ProductDefaults { tax_rate_id: Option<String>, margin_rate_scaled: i64 }

fn resolve_product_defaults(connection: &rusqlite::Connection, company_id: &str, family_id: Option<&str>, product_tax_id: Option<&str>, product_margin: Option<i64>) -> Phase05Result<ProductDefaults> {
    let family = if let Some(id) = family_id {
        connection.query_row("SELECT default_tax_rate_id,default_margin_rate_scaled FROM product_families WHERE id=?1 AND company_id=?2 AND is_active=1", params![id, company_id], |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<i64>>(1)?))).optional()?.ok_or_else(|| Phase05Error::invalid("productFamilyId"))?
    } else { (None, None) };
    let company: (Option<String>, i64) = connection.query_row("SELECT default_tax_rate_id,default_margin_rate_scaled FROM company_settings WHERE company_id=?1", [company_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
    if let Some(tax_id) = product_tax_id { ensure_company_reference(connection, "tax_rates", tax_id, company_id)?; }
    let margin = product_margin.or(family.1).unwrap_or(company.1);
    if !(0..=1_000_000).contains(&margin) { return Err(Phase05Error::invalid("marginRate")); }
    Ok(ProductDefaults { tax_rate_id: product_tax_id.map(str::to_owned).or(family.0).or(company.0), margin_rate_scaled: margin })
}

fn ensure_company_reference(connection: &rusqlite::Connection, table: &str, id: &str, company_id: &str) -> Phase05Result<()> {
    if !["units", "tax_rates", "products", "price_lists"].contains(&table) { return Err(Phase05Error::internal()); }
    let sql = format!("SELECT 1 FROM {table} WHERE id=?1 AND company_id=?2 AND is_active=1");
    connection.query_row(&sql, params![id, company_id], |_| Ok(())).optional()?.ok_or_else(|| Phase05Error::invalid("reference"))
}

fn validate_product(code: &str, name_ar: &str, kind: &str, purchase: i64, sale: Option<i64>, margin: Option<i64>, minimum_stock: i64) -> Phase05Result<()> {
    trim_required(code, "code")?; trim_required(name_ar, "nameAr")?;
    if !matches!(kind, "STOCK_ITEM" | "SERVICE" | "NON_STOCK_ITEM") || purchase < 0 || sale.is_some_and(|value| value < 0) || margin.is_some_and(|value| !(0..=1_000_000).contains(&value)) || minimum_stock < 0 { return Err(Phase05Error::invalid("product")); }
    Ok(())
}

fn current_date_text() -> String {
    super::pricing::format_date(super::pricing::current_device_date())
}
