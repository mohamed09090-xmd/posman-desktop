impl Phase08Service {
    pub fn list_posting_rules(&self, _: ()) -> Phase08Result<Vec<EntityVersion>> {
        let context=self.context(Some("accounting.read"))?;
        self.read(|connection| {
            let mut statement=connection.prepare("SELECT id,row_version FROM posting_rules WHERE company_id=?1 ORDER BY source_event_type,priority DESC,code")?;
            let rows=statement.query_map([context.company_id],|row|Ok(EntityVersion{id:row.get(0)?,row_version:row.get(1)?}))?;
            rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
        })
    }

}
