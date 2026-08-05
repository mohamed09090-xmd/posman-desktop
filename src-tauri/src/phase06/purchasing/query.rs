pub fn list_purchasing_documents(
    &self,
    query: DocumentQuery,
) -> Phase06Result<Vec<DocumentView>> {
    let context = self.context(Some("stock.read"))?;
    self.read(|connection| {
        let mut statement = connection.prepare(
            "SELECT id FROM commercial_documents
             WHERE company_id = ?1
               AND document_type IN (
                 'PURCHASE_ORDER', 'PURCHASE_RECEIPT', 'PURCHASE_INVOICE', 'PURCHASE_RETURN'
               )
               AND (?2 IS NULL OR document_type = ?2)
               AND (?3 IS NULL OR workflow_status = ?3)
               AND (?4 IS NULL OR document_number LIKE '%' || ?4 || '%')
             ORDER BY commercial_date DESC, document_number DESC
             LIMIT ?5",
        )?;
        let ids = statement
            .query_map(
                params![
                    context.company_id,
                    query.document_type,
                    query.status,
                    query.search,
                    query.limit.unwrap_or(200).clamp(1, 1000),
                ],
                |row| row.get::<_, String>(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        ids.iter()
            .map(|id| get_document_connection(connection, &context.company_id, id))
            .collect()
    })
}

pub fn get_purchasing_document(&self, id: String) -> Phase06Result<DocumentView> {
    let context = self.context(Some("stock.read"))?;
    self.read(|connection| get_document_connection(connection, &context.company_id, &id))
}
