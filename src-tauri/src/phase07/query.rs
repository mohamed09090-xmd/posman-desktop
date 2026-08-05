use rusqlite::params;

use crate::phase06::{
    error::{Phase06Error, Phase06Result},
    get_document_connection,
};

use super::{
    dto::{DocumentQuery, DocumentView, SalesLineAvailability, SalesSummary},
    Phase07Service,
};

impl Phase07Service {
    pub fn list_sales_documents(&self, query: DocumentQuery) -> Phase06Result<Vec<DocumentView>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let limit = query.limit.unwrap_or(100).clamp(1, 500);
            let search = query.search.as_deref().unwrap_or("").trim();
            let pattern = format!("%{search}%");
            let mut statement = connection.prepare(
                "SELECT id FROM commercial_documents
                 WHERE company_id=?1
                   AND document_type IN ('SALES_ORDER','DELIVERY_NOTE','SALES_INVOICE','SALES_RETURN','SALES_CREDIT_NOTE')
                   AND (?2 IS NULL OR document_type=?2)
                   AND (?3 IS NULL OR workflow_status=?3)
                   AND (?4='' OR document_number LIKE ?5 OR COALESCE(notes,'') LIKE ?5)
                 ORDER BY commercial_date DESC, created_at DESC LIMIT ?6",
            )?;
            let ids = statement
                .query_map(
                    params![context.company_id, query.document_type, query.status, search, pattern, limit],
                    |row| row.get::<_, String>(0),
                )?
                .collect::<Result<Vec<_>, _>>()?;
            ids.into_iter()
                .map(|id| get_document_connection(connection, &context.company_id, &id))
                .collect()
        })
    }

    pub fn get_sales_document(&self, document_id: String) -> Phase06Result<DocumentView> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            get_document_connection(connection, &context.company_id, &document_id)
        })
    }

    pub fn get_sales_line_availability(
        &self,
        document_id: String,
    ) -> Phase06Result<Vec<SalesLineAvailability>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                "SELECT line.id,line.product_id,line.quantity_scaled,
                   COALESCE(SUM(CASE WHEN link.transformation_type='ORDER_TO_DELIVERY' THEN link.transformed_quantity_scaled ELSE 0 END),0),
                   COALESCE(SUM(CASE WHEN link.transformation_type IN ('DELIVERY_TO_INVOICE','ORDER_TO_INVOICE') THEN link.transformed_quantity_scaled ELSE 0 END),0),
                   COALESCE(SUM(CASE WHEN link.transformation_type='DOCUMENT_TO_RETURN' THEN link.transformed_quantity_scaled ELSE 0 END),0)
                 FROM commercial_document_lines line
                 LEFT JOIN document_line_links link ON link.source_line_id=line.id AND link.company_id=line.company_id
                 WHERE line.document_id=?1 AND line.company_id=?2
                 GROUP BY line.id,line.product_id,line.quantity_scaled ORDER BY line.line_number",
            )?;
            let rows = statement.query_map(params![document_id, context.company_id], |row| {
                let original = row.get::<_, i64>(2)?;
                let delivered = row.get::<_, i64>(3)?;
                let invoiced = row.get::<_, i64>(4)?;
                let returned = row.get::<_, i64>(5)?;
                Ok(SalesLineAvailability {
                    source_line_id: row.get(0)?,
                    product_id: row.get(1)?,
                    original_quantity_scaled: original,
                    delivered_quantity_scaled: delivered,
                    invoiced_quantity_scaled: invoiced,
                    returned_quantity_scaled: returned,
                    remaining_quantity_scaled: original.saturating_sub(delivered.max(invoiced).max(returned)),
                })
            })?.collect::<Result<Vec<_>,_>>()?;
            if rows.is_empty() { return Err(Phase06Error::not_found()); }
            Ok(rows)
        })
    }

    pub fn get_sales_summary(&self) -> Phase06Result<SalesSummary> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            Ok(SalesSummary {
                draft_orders: count_documents(connection,&context.company_id,"SALES_ORDER","DRAFT")?,
                confirmed_orders: count_documents(connection,&context.company_id,"SALES_ORDER","CONFIRMED")?,
                partial_orders: count_documents(connection,&context.company_id,"SALES_ORDER","PARTIALLY_DELIVERED")?,
                uninvoiced_deliveries: connection.query_row(
                    "SELECT COUNT(*) FROM commercial_documents WHERE company_id=?1 AND document_type='DELIVERY_NOTE'
                     AND posting_status='POSTED' AND workflow_status IN ('POSTED','PARTIALLY_INVOICED')",
                    [context.company_id.as_str()],|row|row.get(0),
                )?,
                posted_invoices: connection.query_row(
                    "SELECT COUNT(*) FROM commercial_documents WHERE company_id=?1 AND document_type='SALES_INVOICE' AND posting_status='POSTED'",
                    [context.company_id.as_str()],|row|row.get(0),
                )?,
                below_cost_overrides: connection.query_row(
                    "SELECT COUNT(*) FROM audit_logs WHERE company_id=?1 AND action_code LIKE 'sales%.below_cost'",
                    [context.company_id.as_str()],|row|row.get(0),
                )?,
            })
        })
    }
}

fn count_documents(
    connection: &rusqlite::Connection,
    company_id: &str,
    kind: &str,
    status: &str,
) -> Phase06Result<i64> {
    Ok(connection.query_row(
        "SELECT COUNT(*) FROM commercial_documents WHERE company_id=?1 AND document_type=?2 AND workflow_status=?3",
        params![company_id,kind,status],|row|row.get(0),
    )?)
}
