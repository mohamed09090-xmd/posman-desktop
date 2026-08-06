impl Phase08Service {
    pub fn install_accounting_template(&self, request: InstallAccountingTemplateRequest) -> Phase08Result<EntityVersion> {
        let context = self.context(Some("accounting.configure"))?;
        self.immediate(|tx| {
            let now = now_iso()?;
            if let Some(year) = request.current_fiscal_year_id.as_deref() {
                require_company_row(tx, "fiscal_years", year, &context.company_id, "FISCAL_YEAR_NOT_FOUND")?;
            }
            tx.execute(
                r#"INSERT INTO accounting_setups(company_id,is_enabled,current_fiscal_year_id,created_at,created_by,updated_at,updated_by,row_version)
                   VALUES (?1,?2,?3,?4,?5,?4,?5,1)
                   ON CONFLICT(company_id) DO UPDATE SET is_enabled=excluded.is_enabled,
                     current_fiscal_year_id=excluded.current_fiscal_year_id,updated_at=excluded.updated_at,
                     updated_by=excluded.updated_by,row_version=accounting_setups.row_version+1"#,
                params![context.company_id, boolean(request.enabled), request.current_fiscal_year_id, now, context.user_id],
            )?;
            for mapping in &request.roles {
                require_active_postable_account(tx, &context.company_id, &mapping.account_id)?;
                tx.execute(
                    r#"INSERT INTO accounting_account_roles(id,company_id,role_code,account_id,created_at,created_by,updated_at,updated_by,row_version)
                       VALUES (?1,?2,?3,?4,?5,?6,?5,?6,1)
                       ON CONFLICT(company_id,role_code) DO UPDATE SET account_id=excluded.account_id,
                         updated_at=excluded.updated_at,updated_by=excluded.updated_by,row_version=accounting_account_roles.row_version+1"#,
                    params![new_id(), context.company_id, mapping.role_code, mapping.account_id, now, context.user_id],
                )?;
            }
            for mapping in &request.payment_methods {
                if mapping.account_id.is_some() == mapping.account_role_code.is_some() {
                    return Err(Phase08Error::validation(
                        "A payment method must map to exactly one account or account role.",
                    ));
                }
                require_company_row(
                    tx,
                    "payment_methods",
                    &mapping.payment_method_id,
                    &context.company_id,
                    "PAYMENT_METHOD_NOT_FOUND",
                )?;
                if let Some(account_id) = mapping.account_id.as_deref() {
                    require_active_postable_account(tx, &context.company_id, account_id)?;
                }
                tx.execute(
                    r#"INSERT INTO payment_method_accounting(
                         id,company_id,payment_method_id,account_id,account_role_code,
                         created_at,created_by,updated_at,updated_by,row_version)
                       VALUES (?1,?2,?3,?4,?5,?6,?7,?6,?7,1)
                       ON CONFLICT(company_id,payment_method_id) DO UPDATE SET
                         account_id=excluded.account_id,account_role_code=excluded.account_role_code,
                         updated_at=excluded.updated_at,updated_by=excluded.updated_by,
                         row_version=payment_method_accounting.row_version+1"#,
                    params![
                        new_id(),
                        context.company_id,
                        mapping.payment_method_id,
                        mapping.account_id,
                        mapping.account_role_code,
                        now,
                        context.user_id,
                    ],
                )?;
            }
            let version = tx.query_row("SELECT row_version FROM accounting_setups WHERE company_id=?1", [&context.company_id], |r| r.get(0))?;
            Ok(EntityVersion { id: context.company_id, row_version: version })
        })
    }

}
