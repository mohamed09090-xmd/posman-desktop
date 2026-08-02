use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::json;

use super::{
    dto::{
        CreateProductRequest, Page, PageRequest, ProductPriceInput, ProductView, SetActiveRequest,
        UpdateProductRequest,
    },
    error::{Phase05Error, Phase05Result},
    pricing::calculate_pricing,
    state::{audit, new_id, now_iso, trim_optional, trim_required, Phase05Service, SessionContext},
};

const BELOW_COST_BLOCK: &str = "BLOCK";
const BELOW_COST_ADMIN_OVERRIDE: &str = "ADMIN_OVERRIDE";
const BELOW_COST_WARNING_ONLY: &str = "WARNING_ONLY";
const BELOW_COST_PERMISSION: &str = "pricing.override_below_cost";

impl Phase05Service {
    pub fn list_products(&self, request: PageRequest) -> Phase05Result<Page<ProductView>> {
        let context = self.require_session(Some("catalog.view"))?;
        let page = request.page.unwrap_or(1).max(1);
        let page_size = request.page_size.unwrap_or(25).clamp(1, 100);
        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        let search = request.search.unwrap_or_default().trim().to_lowercase();
        if search.chars().count() > 100 {
            return Err(Phase05Error::invalid("search"));
        }
        let pattern = format!("%{search}%");
        let connection = self.open()?;
        let policy = company_below_cost_policy(&connection, &context.company_id)?;
        let total: i64 = connection.query_row(
            r#"
            SELECT COUNT(*) FROM products WHERE company_id=?1 AND (
                ?2='' OR lower(code) LIKE ?3 OR lower(name_ar) LIKE ?3
                OR lower(COALESCE(name_fr,'')) LIKE ?3
                OR lower(COALESCE(barcode,'')) LIKE ?3)
            "#,
            params![context.company_id, search, pattern],
            |row| row.get(0),
        )?;
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
        let rows = statement.query_map(
            params![context.company_id, search, pattern, page_size, offset],
            |row| {
                let purchase: i64 = row.get(7)?;
                let sale: i64 = row.get(8)?;
                Ok(ProductView {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name_ar: row.get(2)?,
                    name_fr: row.get(3)?,
                    unit_id: row.get(4)?,
                    product_family_id: row.get(5)?,
                    tax_rate_id: row.get(6)?,
                    purchase_price_scaled: purchase,
                    sale_price_scaled: sale,
                    suggested_sale_price_scaled: sale,
                    pricing_warning: pricing_warning(purchase, sale).map(str::to_owned),
                    below_cost_policy: policy.clone(),
                    is_active: row.get::<_, i64>(9)? == 1,
                    row_version: row.get(10)?,
                })
            },
        )?;
        let items = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(Page {
            items,
            page,
            page_size,
            total: u64::try_from(total).unwrap_or(0),
        })
    }

    pub fn create_product(&self, request: CreateProductRequest) -> Phase05Result<ProductView> {
        validate_product(
            &request.code,
            &request.name_ar,
            &request.product_kind,
            request.default_purchase_price_scaled,
            request.manual_sale_price_scaled,
            request.margin_rate_scaled,
            request.minimum_stock_scaled,
        )?;
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        let defaults = resolve_product_defaults(
            &connection,
            &context.company_id,
            request.product_family_id.as_deref(),
            request.default_tax_rate_id.as_deref(),
            request.margin_rate_scaled,
        )?;
        ensure_company_reference(&connection, "units", &request.unit_id, &context.company_id)?;
        let suggested = calculate_pricing(
            request.default_purchase_price_scaled,
            defaults.margin_rate_scaled,
            0,
            0,
        )?
        .sale_ht_scaled;
        let sale_price = request.manual_sale_price_scaled.unwrap_or(suggested);
        let guard = self.guard_sale_price(
            &connection,
            &context.company_id,
            request.default_purchase_price_scaled,
            sale_price,
            request.below_cost_override_reason.as_deref(),
        )?;
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
            params![
                product_id,
                context.company_id,
                request.product_family_id,
                request.unit_id,
                defaults.tax_rate_id,
                trim_required(&request.code, "code")?,
                trim_optional(request.barcode.as_deref()),
                trim_required(&request.name_ar, "nameAr")?,
                trim_optional(request.name_fr.as_deref()),
                request.product_kind,
                if stock_tracked { 1_i64 } else { 0_i64 },
                request.minimum_stock_scaled,
                request.default_purchase_price_scaled,
                sale_price,
                timestamp,
                context.user_id
            ],
        )?;
        let price_list_id: String = transaction.query_row(
            "SELECT id FROM price_lists WHERE company_id=?1 AND is_default=1 AND is_active=1 LIMIT 1",
            [context.company_id.as_str()],
            |row| row.get(0),
        )?;
        transaction.execute(
            r#"
            INSERT INTO product_prices (id,company_id,price_list_id,product_id,
                unit_price_scaled,valid_from,created_at,created_by,updated_at,updated_by)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?7,?8)
            "#,
            params![
                new_id(),
                context.company_id,
                price_list_id,
                product_id,
                sale_price,
                current_date_text(),
                timestamp,
                context.user_id
            ],
        )?;
        audit_product_price(
            &transaction,
            &context,
            "catalog.product.create",
            &product_id,
            request.default_purchase_price_scaled,
            sale_price,
            suggested,
            &guard,
        )?;
        transaction.commit()?;
        self.get_product(&product_id, suggested)
    }

    pub fn update_product(&self, request: UpdateProductRequest) -> Phase05Result<ProductView> {
        validate_product(
            &request.code,
            &request.name_ar,
            &request.product_kind,
            request.default_purchase_price_scaled,
            request.manual_sale_price_scaled,
            request.margin_rate_scaled,
            request.minimum_stock_scaled,
        )?;
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        let defaults = resolve_product_defaults(
            &connection,
            &context.company_id,
            request.product_family_id.as_deref(),
            request.default_tax_rate_id.as_deref(),
            request.margin_rate_scaled,
        )?;
        ensure_company_reference(&connection, "units", &request.unit_id, &context.company_id)?;
        let suggested = calculate_pricing(
            request.default_purchase_price_scaled,
            defaults.margin_rate_scaled,
            0,
            0,
        )?
        .sale_ht_scaled;
        let sale_price = request.manual_sale_price_scaled.unwrap_or(suggested);
        let guard = self.guard_sale_price(
            &connection,
            &context.company_id,
            request.default_purchase_price_scaled,
            sale_price,
            request.below_cost_override_reason.as_deref(),
        )?;
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
            params![
                request.product_family_id,
                request.unit_id,
                defaults.tax_rate_id,
                trim_required(&request.code, "code")?,
                trim_optional(request.barcode.as_deref()),
                trim_required(&request.name_ar, "nameAr")?,
                trim_optional(request.name_fr.as_deref()),
                request.product_kind,
                if stock_tracked { 1_i64 } else { 0_i64 },
                request.minimum_stock_scaled,
                request.default_purchase_price_scaled,
                sale_price,
                now_iso()?,
                context.user_id,
                request.id,
                context.company_id,
                request.row_version
            ],
        )?;
        if changed != 1 {
            return Err(Phase05Error::concurrency());
        }
        let price_list_id: String = transaction.query_row(
            "SELECT id FROM price_lists WHERE company_id=?1 AND is_default=1 AND is_active=1 LIMIT 1",
            [context.company_id.as_str()],
            |row| row.get(0),
        )?;
        let current_price_id: Option<String> = transaction
            .query_row(
                "SELECT id FROM product_prices WHERE company_id=?1 AND product_id=?2 AND price_list_id=?3 AND valid_to IS NULL ORDER BY valid_from DESC LIMIT 1",
                params![context.company_id, request.id, price_list_id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(price_id) = current_price_id {
            transaction.execute(
                "UPDATE product_prices SET unit_price_scaled=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5",
                params![sale_price, now_iso()?, context.user_id, price_id, context.company_id],
            )?;
        } else {
            transaction.execute(
                "INSERT INTO product_prices (id,company_id,price_list_id,product_id,unit_price_scaled,valid_from,created_at,created_by,updated_at,updated_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?7,?8)",
                params![new_id(), context.company_id, price_list_id, request.id, sale_price, current_date_text(), now_iso()?, context.user_id],
            )?;
        }
        audit_product_price(
            &transaction,
            &context,
            "catalog.product.update",
            &request.id,
            request.default_purchase_price_scaled,
            sale_price,
            suggested,
            &guard,
        )?;
        transaction.commit()?;
        self.get_product(&request.id, suggested)
    }

    pub fn set_product_active(&self, request: SetActiveRequest) -> Phase05Result<ProductView> {
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            "UPDATE products SET is_active=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5 AND row_version=?6",
            params![if request.is_active { 1_i64 } else { 0_i64 }, now_iso()?, context.user_id, request.id, context.company_id, request.row_version],
        )?;
        if changed != 1 {
            return Err(Phase05Error::concurrency());
        }
        audit(
            &transaction,
            &context,
            "catalog.product.set_active",
            "products",
            &request.id,
            None,
        )?;
        transaction.commit()?;
        self.get_product(&request.id, 0)
    }

    pub fn set_product_price(&self, request: ProductPriceInput) -> Phase05Result<()> {
        if request.unit_price_scaled < 0 || request.valid_from.len() != 10 {
            return Err(Phase05Error::invalid("productPrice"));
        }
        let context = self.require_session(Some("catalog.manage"))?;
        let mut connection = self.open()?;
        ensure_company_reference(
            &connection,
            "products",
            &request.product_id,
            &context.company_id,
        )?;
        ensure_company_reference(
            &connection,
            "price_lists",
            &request.price_list_id,
            &context.company_id,
        )?;
        let purchase_price: i64 = connection.query_row(
            "SELECT COALESCE(default_purchase_price_scaled,0) FROM products WHERE id=?1 AND company_id=?2",
            params![request.product_id, context.company_id],
            |row| row.get(0),
        )?;
        let guard = self.guard_sale_price(
            &connection,
            &context.company_id,
            purchase_price,
            request.unit_price_scaled,
            request.below_cost_override_reason.as_deref(),
        )?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE product_prices SET valid_to=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE company_id=?4 AND product_id=?5 AND price_list_id=?6 AND valid_to IS NULL AND valid_from<?7",
            params![request.valid_from, now_iso()?, context.user_id, context.company_id, request.product_id, request.price_list_id, request.valid_from],
        )?;
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
            params![
                new_id(),
                context.company_id,
                request.price_list_id,
                request.product_id,
                request.unit_price_scaled,
                request.valid_from,
                now_iso()?,
                context.user_id
            ],
        )?;
        transaction.execute(
            "UPDATE products SET default_sale_price_scaled=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5",
            params![request.unit_price_scaled, now_iso()?, context.user_id, request.product_id, context.company_id],
        )?;
        audit_product_price(
            &transaction,
            &context,
            "catalog.product_price.set",
            &request.product_id,
            purchase_price,
            request.unit_price_scaled,
            request.unit_price_scaled,
            &guard,
        )?;
        transaction.commit()?;
        Ok(())
    }

    fn guard_sale_price(
        &self,
        connection: &rusqlite::Connection,
        company_id: &str,
        purchase_price: i64,
        sale_price: i64,
        override_reason: Option<&str>,
    ) -> Phase05Result<PricingGuard> {
        let policy = company_below_cost_policy(connection, company_id)?;
        if sale_price >= purchase_price {
            return Ok(PricingGuard {
                policy,
                warning: pricing_warning(purchase_price, sale_price).map(str::to_owned),
                override_reason: None,
            });
        }
        match policy.as_str() {
            BELOW_COST_BLOCK => Err(Phase05Error::below_cost_blocked()),
            BELOW_COST_ADMIN_OVERRIDE => {
                if !self.has_permission(BELOW_COST_PERMISSION)? {
                    return Err(Phase05Error::below_cost_override_required());
                }
                let reason = trim_required(
                    override_reason.unwrap_or_default(),
                    "belowCostOverrideReason",
                )?;
                if reason.chars().count() < 3 || reason.chars().count() > 500 {
                    return Err(Phase05Error::invalid("belowCostOverrideReason"));
                }
                Ok(PricingGuard {
                    policy,
                    warning: Some("BELOW_COST".to_owned()),
                    override_reason: Some(reason),
                })
            }
            BELOW_COST_WARNING_ONLY => Ok(PricingGuard {
                policy,
                warning: Some("BELOW_COST".to_owned()),
                override_reason: None,
            }),
            _ => Err(Phase05Error::internal()),
        }
    }

    fn get_product(&self, id: &str, suggested: i64) -> Phase05Result<ProductView> {
        let context = self.require_session(Some("catalog.view"))?;
        self.open()?
            .query_row(
                r#"
                SELECT p.id,p.code,p.name_ar,p.name_fr,p.unit_id,p.product_family_id,
                       p.default_tax_rate_id,COALESCE(p.default_purchase_price_scaled,0),
                       COALESCE(p.default_sale_price_scaled,0),p.is_active,p.row_version,
                       s.below_cost_policy
                FROM products p JOIN company_settings s ON s.company_id=p.company_id
                WHERE p.id=?1 AND p.company_id=?2
                "#,
                params![id, context.company_id],
                |row| {
                    let purchase: i64 = row.get(7)?;
                    let sale: i64 = row.get(8)?;
                    Ok(ProductView {
                        id: row.get(0)?,
                        code: row.get(1)?,
                        name_ar: row.get(2)?,
                        name_fr: row.get(3)?,
                        unit_id: row.get(4)?,
                        product_family_id: row.get(5)?,
                        tax_rate_id: row.get(6)?,
                        purchase_price_scaled: purchase,
                        sale_price_scaled: sale,
                        suggested_sale_price_scaled: suggested,
                        pricing_warning: pricing_warning(purchase, sale).map(str::to_owned),
                        below_cost_policy: row.get(11)?,
                        is_active: row.get::<_, i64>(9)? == 1,
                        row_version: row.get(10)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| Phase05Error::new("NOT_FOUND", "The product was not found."))
    }
}

struct ProductDefaults {
    tax_rate_id: Option<String>,
    margin_rate_scaled: i64,
}

struct PricingGuard {
    policy: String,
    warning: Option<String>,
    override_reason: Option<String>,
}

fn resolve_product_defaults(
    connection: &rusqlite::Connection,
    company_id: &str,
    family_id: Option<&str>,
    product_tax_id: Option<&str>,
    product_margin: Option<i64>,
) -> Phase05Result<ProductDefaults> {
    let family = if let Some(id) = family_id {
        connection
            .query_row(
                "SELECT default_tax_rate_id,default_margin_rate_scaled FROM product_families WHERE id=?1 AND company_id=?2 AND is_active=1",
                params![id, company_id],
                |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<i64>>(1)?)),
            )
            .optional()?
            .ok_or_else(|| Phase05Error::invalid("productFamilyId"))?
    } else {
        (None, None)
    };
    let company: (Option<String>, i64) = connection.query_row(
        "SELECT default_tax_rate_id,default_margin_rate_scaled FROM company_settings WHERE company_id=?1",
        [company_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if let Some(tax_id) = product_tax_id {
        ensure_company_reference(connection, "tax_rates", tax_id, company_id)?;
    }
    let margin = product_margin.or(family.1).unwrap_or(company.1);
    if !(0..=1_000_000).contains(&margin) {
        return Err(Phase05Error::invalid("marginRate"));
    }
    Ok(ProductDefaults {
        tax_rate_id: product_tax_id.map(str::to_owned).or(family.0).or(company.0),
        margin_rate_scaled: margin,
    })
}

fn company_below_cost_policy(
    connection: &rusqlite::Connection,
    company_id: &str,
) -> Phase05Result<String> {
    connection
        .query_row(
            "SELECT below_cost_policy FROM company_settings WHERE company_id=?1",
            [company_id],
            |row| row.get(0),
        )
        .map_err(Phase05Error::from)
}

fn ensure_company_reference(
    connection: &rusqlite::Connection,
    table: &str,
    id: &str,
    company_id: &str,
) -> Phase05Result<()> {
    if !["units", "tax_rates", "products", "price_lists"].contains(&table) {
        return Err(Phase05Error::internal());
    }
    let sql = format!("SELECT 1 FROM {table} WHERE id=?1 AND company_id=?2 AND is_active=1");
    connection
        .query_row(&sql, params![id, company_id], |_| Ok(()))
        .optional()?
        .ok_or_else(|| Phase05Error::invalid("reference"))
}

fn validate_product(
    code: &str,
    name_ar: &str,
    kind: &str,
    purchase: i64,
    sale: Option<i64>,
    margin: Option<i64>,
    minimum_stock: i64,
) -> Phase05Result<()> {
    trim_required(code, "code")?;
    trim_required(name_ar, "nameAr")?;
    if !matches!(kind, "STOCK_ITEM" | "SERVICE" | "NON_STOCK_ITEM")
        || purchase < 0
        || sale.is_some_and(|value| value < 0)
        || margin.is_some_and(|value| !(0..=1_000_000).contains(&value))
        || minimum_stock < 0
    {
        return Err(Phase05Error::invalid("product"));
    }
    Ok(())
}

fn pricing_warning(purchase_price: i64, sale_price: i64) -> Option<&'static str> {
    if sale_price < purchase_price {
        Some("BELOW_COST")
    } else if sale_price == purchase_price {
        Some("ZERO_MARGIN")
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn audit_product_price(
    transaction: &Transaction<'_>,
    context: &SessionContext,
    action: &str,
    product_id: &str,
    purchase_price: i64,
    sale_price: i64,
    suggested_sale_price: i64,
    guard: &PricingGuard,
) -> Phase05Result<()> {
    let details = json!({
        "purchasePriceScaled": purchase_price,
        "salePriceScaled": sale_price,
        "suggestedSalePriceScaled": suggested_sale_price,
        "belowCostPolicy": guard.policy,
        "pricingWarning": guard.warning,
        "overrideReason": guard.override_reason,
    })
    .to_string();
    audit(
        transaction,
        context,
        action,
        "products",
        product_id,
        Some(&details),
    )?;
    if guard.override_reason.is_some() {
        audit(
            transaction,
            context,
            "pricing.below_cost.override",
            "products",
            product_id,
            Some(&details),
        )?;
    }
    Ok(())
}

fn current_date_text() -> String {
    super::pricing::format_date(super::pricing::current_device_date())
}

#[cfg(test)]
mod tests {
    use super::pricing_warning;

    #[test]
    fn pricing_warning_distinguishes_below_cost_and_zero_margin() {
        assert_eq!(pricing_warning(100_000, 90_000), Some("BELOW_COST"));
        assert_eq!(pricing_warning(100_000, 100_000), Some("ZERO_MARGIN"));
        assert_eq!(pricing_warning(100_000, 120_000), None);
    }
}
