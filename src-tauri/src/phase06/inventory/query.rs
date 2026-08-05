use super::*;

impl Phase06Service {
    pub fn list_stock_balances(&self, query: StockQuery) -> Phase06Result<Vec<StockBalanceView>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                r#"
                SELECT balance.product_id, product.code,
                       COALESCE(NULLIF(product.name_ar, ''), product.name_fr),
                       balance.warehouse_id,
                       COALESCE(NULLIF(warehouse.name_ar, ''), warehouse.name_fr),
                       balance.warehouse_location_id,
                       CASE WHEN location.id IS NULL THEN NULL
                            ELSE COALESCE(NULLIF(location.name_ar, ''), location.name_fr)
                       END,
                       balance.on_hand_scaled, balance.reserved_scaled,
                       balance.available_scaled, balance.average_cost_scaled,
                       balance.row_version
                FROM stock_balances AS balance
                JOIN products AS product
                  ON product.id=balance.product_id
                 AND product.company_id=balance.company_id
                JOIN warehouses AS warehouse
                  ON warehouse.id=balance.warehouse_id
                 AND warehouse.company_id=balance.company_id
                LEFT JOIN warehouse_locations AS location
                  ON location.id=balance.warehouse_location_id
                 AND location.company_id=balance.company_id
                WHERE balance.company_id=?1
                  AND (?2 IS NULL OR balance.product_id=?2)
                  AND (?3 IS NULL OR balance.warehouse_id=?3)
                  AND (?4 IS NULL OR ifnull(balance.warehouse_location_id, '')=ifnull(?4, ''))
                ORDER BY product.code, warehouse.code, ifnull(location.code, '')
                LIMIT ?5
                "#,
            )?;
            let raw = statement
                .query_map(
                    params![
                        context.company_id,
                        query.product_id,
                        query.warehouse_id,
                        query.warehouse_location_id,
                        query.limit.unwrap_or(500).clamp(1, 2_000)
                    ],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, Option<String>>(6)?,
                            row.get::<_, i64>(7)?,
                            row.get::<_, i64>(8)?,
                            row.get::<_, i64>(9)?,
                            row.get::<_, i64>(10)?,
                            row.get::<_, i64>(11)?,
                        ))
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;

            raw.into_iter()
                .map(
                    |(
                        product_id,
                        product_code,
                        product_name,
                        warehouse_id,
                        warehouse_name,
                        warehouse_location_id,
                        location_name,
                        on_hand_scaled,
                        reserved_scaled,
                        available_scaled,
                        average_cost_scaled,
                        row_version,
                    )| {
                        let absolute = super::super::fixed_point::extended_cost_minor(
                            on_hand_scaled,
                            average_cost_scaled,
                        )?;
                        let inventory_value_minor = if on_hand_scaled < 0 {
                            -absolute
                        } else {
                            absolute
                        };
                        Ok(StockBalanceView {
                            product_id,
                            product_code,
                            product_name,
                            warehouse_id,
                            warehouse_name,
                            warehouse_location_id,
                            location_name,
                            on_hand_scaled,
                            reserved_scaled,
                            available_scaled,
                            average_cost_scaled,
                            inventory_value_minor,
                            row_version,
                        })
                    },
                )
                .collect()
        })
    }

    pub fn list_stock_movements(&self, query: StockQuery) -> Phase06Result<Vec<MovementView>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                r#"
                SELECT id, product_id, warehouse_id, warehouse_location_id,
                       source_document_id, movement_type, business_date,
                       quantity_delta_scaled, quantity_after_scaled,
                       unit_cost_scaled, average_cost_after_scaled,
                       extended_cost_minor, notes
                FROM stock_movements
                WHERE company_id=?1
                  AND (?2 IS NULL OR product_id=?2)
                  AND (?3 IS NULL OR warehouse_id=?3)
                  AND (?4 IS NULL OR ifnull(warehouse_location_id, '')=ifnull(?4, ''))
                ORDER BY occurred_at DESC, id DESC
                LIMIT ?5
                "#,
            )?;
            let rows = statement
                .query_map(
                    params![
                        context.company_id,
                        query.product_id,
                        query.warehouse_id,
                        query.warehouse_location_id,
                        query.limit.unwrap_or(500).clamp(1, 2_000)
                    ],
                    |row| {
                        Ok(MovementView {
                            id: row.get(0)?,
                            product_id: row.get(1)?,
                            warehouse_id: row.get(2)?,
                            warehouse_location_id: row.get(3)?,
                            source_document_id: row.get(4)?,
                            movement_type: row.get(5)?,
                            business_date: row.get(6)?,
                            quantity_delta_scaled: row.get(7)?,
                            quantity_after_scaled: row.get(8)?,
                            unit_cost_scaled: row.get(9)?,
                            average_cost_after_scaled: row.get(10)?,
                            extended_cost_minor: row.get(11)?,
                            notes: row.get(12)?,
                        })
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
}
