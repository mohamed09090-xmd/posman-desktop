impl Phase08Service {
    pub fn list_accounts(&self, _: ()) -> Phase08Result<Vec<AccountView>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| {
            let mut stmt = connection.prepare("SELECT id,code,name_ar,name_fr,account_type,normal_side,allow_posting,is_active,row_version FROM accounts WHERE company_id=?1 ORDER BY code")?;
            let rows = stmt.query_map([context.company_id], |row| Ok(AccountView {
                id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_fr: row.get(3)?,
                account_type: row.get(4)?, normal_side: row.get(5)?, allow_posting: row.get::<_,i64>(6)?==1,
                is_active: row.get::<_,i64>(7)?==1, row_version: row.get(8)?,
            }))?;
            rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
        })
    }

    pub fn create_account(&self, input: AccountInput) -> Phase08Result<EntityVersion> {
        self.save_account(input, false)
    }
    pub fn update_account(&self, input: AccountInput) -> Phase08Result<EntityVersion> {
        self.save_account(input, true)
    }
    fn save_account(&self, input: AccountInput, update: bool) -> Phase08Result<EntityVersion> {
        let context = self.context(Some("accounting.configure"))?;
        validate_account(&input)?;
        self.immediate(|tx| {
            let now = now_iso()?;
            if update {
                let id = input.id.as_deref().ok_or_else(|| Phase08Error::validation("Account id is required."))?;
                let version = input.row_version.ok_or_else(|| Phase08Error::validation("Account row version is required."))?;
                let changed = tx.execute(
                    r#"UPDATE accounts SET parent_account_id=?1,code=?2,name_ar=?3,name_fr=?4,account_type=?5,normal_side=?6,
                       allow_posting=?7,is_active=?8,updated_at=?9,updated_by=?10,row_version=row_version+1
                       WHERE id=?11 AND company_id=?12 AND row_version=?13"#,
                    params![input.parent_account_id,input.code,input.name_ar,input.name_fr,input.account_type,input.normal_side,
                        boolean(input.allow_posting),boolean(input.is_active),now,context.user_id,id,context.company_id,version],
                )?;
                if changed != 1 { return Err(Phase08Error::new("ACCOUNT_CONFLICT", "The account changed; reload and retry.", true)); }
                Ok(EntityVersion { id: id.to_owned(), row_version: version+1 })
            } else {
                let id = input.id.clone().unwrap_or_else(new_id);
                tx.execute(
                    r#"INSERT INTO accounts(id,company_id,parent_account_id,code,name_ar,name_fr,account_type,normal_side,allow_posting,is_active,created_at,created_by,updated_at,updated_by,row_version)
                       VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?11,?12,1)"#,
                    params![id,context.company_id,input.parent_account_id,input.code,input.name_ar,input.name_fr,input.account_type,input.normal_side,
                        boolean(input.allow_posting),boolean(input.is_active),now,context.user_id],
                )?;
                Ok(EntityVersion { id, row_version: 1 })
            }
        })
    }

}
