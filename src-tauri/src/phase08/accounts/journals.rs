impl Phase08Service {
    pub fn create_accounting_journal(&self, input: JournalInput) -> Phase08Result<EntityVersion> { self.save_journal(input, false) }
    pub fn update_accounting_journal(&self, input: JournalInput) -> Phase08Result<EntityVersion> { self.save_journal(input, true) }
    fn save_journal(&self, input: JournalInput, update: bool) -> Phase08Result<EntityVersion> {
        let context = self.context(Some("accounting.configure"))?;
        if input.code.trim().is_empty() || input.name_ar.trim().is_empty() { return Err(Phase08Error::validation("Journal code and Arabic name are required.")); }
        self.immediate(|tx| {
            let now=now_iso()?;
            if update {
                let id=input.id.as_deref().ok_or_else(|| Phase08Error::validation("Journal id is required."))?;
                let version=input.row_version.ok_or_else(|| Phase08Error::validation("Journal row version is required."))?;
                let changed=tx.execute("UPDATE accounting_journals SET code=?1,name_ar=?2,name_fr=?3,journal_type=?4,is_active=?5,updated_at=?6,updated_by=?7,row_version=row_version+1 WHERE id=?8 AND company_id=?9 AND row_version=?10",
                    params![input.code,input.name_ar,input.name_fr,input.journal_type,boolean(input.is_active),now,context.user_id,id,context.company_id,version])?;
                if changed!=1 { return Err(Phase08Error::new("JOURNAL_CONFLICT","The journal changed; reload and retry.",true)); }
                Ok(EntityVersion{id:id.to_owned(),row_version:version+1})
            } else {
                let id=input.id.clone().unwrap_or_else(new_id);
                tx.execute("INSERT INTO accounting_journals(id,company_id,code,name_ar,name_fr,journal_type,is_active,created_at,created_by,updated_at,updated_by,row_version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?8,?9,1)",
                    params![id,context.company_id,input.code,input.name_ar,input.name_fr,input.journal_type,boolean(input.is_active),now,context.user_id])?;
                Ok(EntityVersion{id,row_version:1})
            }
        })
    }

    pub fn list_accounting_journals(&self, _: ()) -> Phase08Result<Vec<EntityVersion>> {
        let context=self.context(Some("accounting.read"))?;
        self.read(|c| {
            let mut s=c.prepare("SELECT id,row_version FROM accounting_journals WHERE company_id=?1 ORDER BY code")?;
            let rows=s.query_map([context.company_id],|r| Ok(EntityVersion{id:r.get(0)?,row_version:r.get(1)?}))?;
            rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
        })
    }

}
