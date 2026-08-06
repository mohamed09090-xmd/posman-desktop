pub(crate) fn reverse_entry_in_tx(tx:&Transaction<'_>,context:&Phase06AuthContext,source_id:&str,date:&str,reason:&str)->Phase08Result<EntityVersion>{
    let source=tx.query_row("SELECT fiscal_year_id,accounting_journal_id,status,entry_number FROM journal_entries WHERE id=?1 AND company_id=?2",params![source_id,context.company_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).optional()?.ok_or_else(||Phase08Error::new("JOURNAL_ENTRY_NOT_FOUND","Journal entry was not found.",false))?;
    if source.2!="POSTED"{return Err(Phase08Error::new("JOURNAL_NOT_REVERSIBLE","Only a posted journal can be reversed.",false));}
    let (_,period)=resolve_open_period(tx,&context.company_id,date)?;
    let existing:i64=tx.query_row("SELECT COUNT(*) FROM journal_entries WHERE company_id=?1 AND reversal_of_entry_id=?2 AND status IN ('POSTED','REVERSED')",params![context.company_id,source_id],|r|r.get(0))?;
    if existing>0{return Err(Phase08Error::new("JOURNAL_ALREADY_REVERSED","The journal already has a reversal.",false));}
    let id=new_id();let now=now_iso()?;let number=next_entry_number(tx,&context.company_id,&source.0,&source.1,date)?;
    tx.execute("INSERT INTO journal_entries(id,company_id,fiscal_year_id,fiscal_period_id,accounting_journal_id,source_document_id,reversal_of_entry_id,entry_number,entry_date,status,source_event_type,source_event_id,idempotency_key,memo,created_at,created_by,updated_at,updated_by,row_version,reversal_reason) SELECT ?1,company_id,fiscal_year_id,?2,accounting_journal_id,source_document_id,id,?3,?4,'DRAFT','JOURNAL_REVERSAL',id,?1,?5,?6,?7,?6,?7,1,?8 FROM journal_entries WHERE id=?9",
        params![id,period,number,date,format!("Reversal of {}",source.3),now,context.user_id,reason,source_id])?;
    tx.execute("INSERT INTO journal_entry_lines(id,company_id,journal_entry_id,account_id,partner_id,product_id,line_number,description,debit_minor,credit_minor,created_at,created_by) SELECT lower(hex(randomblob(16))),company_id,?1,account_id,partner_id,product_id,line_number,'Reversal: '||description,credit_minor,debit_minor,?2,?3 FROM journal_entry_lines WHERE journal_entry_id=?4 ORDER BY line_number",params![id,now,context.user_id,source_id])?;
    validate_entry_balance(tx,&id)?;
    tx.execute("UPDATE journal_entries SET status='POSTED',posted_at=?1,posted_by=?2,updated_at=?1,updated_by=?2,row_version=2 WHERE id=?3",params![now,context.user_id,id])?;
    Ok(EntityVersion{id,row_version:2})
}

fn select_rule(
    tx: &Transaction<'_>,
    company: &str,
    source: &SourceEventRequest,
) -> Phase08Result<SelectedRule> {
    let mut statement = tx.prepare(
        "SELECT id,accounting_journal_id,priority FROM posting_rules
         WHERE company_id=?1 AND source_event_type=?2 AND is_active=1
           AND valid_from<=?3 AND (valid_to IS NULL OR valid_to>=?3)
         ORDER BY priority DESC,id",
    )?;
    let candidates = statement
        .query_map(params![company, source.source_event_type, source.event_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let first = candidates.first().ok_or_else(|| {
        Phase08Error::new(
            "POSTING_RULE_MISSING",
            "Configure an active posting rule for this source event.",
            false,
        )
    })?;
    if candidates
        .get(1)
        .is_some_and(|second| second.2 == first.2)
    {
        return Err(Phase08Error::new(
            "POSTING_RULE_AMBIGUOUS",
            "More than one posting rule has the highest priority.",
            false,
        ));
    }
    let mut lines_statement = tx.prepare(
        r#"SELECT line_number,side,account_id,account_role_code,amount_component,
                  description_ar,partner_dimension,product_dimension
           FROM posting_rule_lines WHERE posting_rule_id=?1 ORDER BY line_number"#,
    )?;
    let lines = lines_statement
        .query_map([&first.0], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)? == 1,
                row.get::<_, i64>(7)? == 1,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if lines.len() < 2 {
        return Err(Phase08Error::new(
            "POSTING_RULE_INCOMPLETE",
            "The selected posting rule requires at least two lines.",
            false,
        ));
    }
    let mut resolved = Vec::new();
    for line in lines {
        let account = if let Some(account_id) = line.2 {
            account_id
        } else {
            let role = line.3.as_deref().ok_or_else(|| {
                Phase08Error::new(
                    "ACCOUNT_ROLE_MISSING",
                    "Map every account role used by the selected rule.",
                    false,
                )
            })?;
            resolve_role_account(tx, company, role, source.payment_method_id.as_deref())?
        };
        resolved.push(RuleLine {
            line_number: line.0,
            side: line.1,
            account_id: account,
            component: line.4,
            description: line.5,
            partner_dimension: line.6,
            product_dimension: line.7,
        });
    }
    Ok(SelectedRule {
        id: first.0.clone(),
        journal_id: first.1.clone(),
        lines: resolved,
    })
}

fn resolve_role_account(
    tx: &Transaction<'_>,
    company: &str,
    role: &str,
    payment_method_id: Option<&str>,
) -> Phase08Result<String> {
    if matches!(role, "CASH" | "BANK") {
        if let Some(method_id) = payment_method_id {
            let mapped = tx
                .query_row(
                    r#"SELECT COALESCE(mapping.account_id,role_account.account_id)
                       FROM payment_method_accounting mapping
                       LEFT JOIN accounting_account_roles role_account
                         ON role_account.company_id=mapping.company_id
                        AND role_account.role_code=mapping.account_role_code
                       WHERE mapping.company_id=?1 AND mapping.payment_method_id=?2"#,
                    params![company, method_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            return mapped.ok_or_else(|| {
                Phase08Error::new(
                    "PAYMENT_METHOD_ACCOUNTING_MISSING",
                    "Configure the accounting account for the selected payment method.",
                    false,
                )
            });
        }
    }
    tx.query_row(
        "SELECT account_id FROM accounting_account_roles WHERE company_id=?1 AND role_code=?2",
        params![company, role],
        |row| row.get(0),
    )
    .optional()?
    .ok_or_else(|| {
        Phase08Error::new(
            "ACCOUNT_ROLE_MISSING",
            "Map every account role used by the selected rule.",
            false,
        )
    })
}
