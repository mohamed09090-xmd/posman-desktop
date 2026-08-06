pub fn post_source_event_in_tx(
    tx: &Transaction<'_>,
    context: &Phase06AuthContext,
    request: &Idempotent<SourceEventRequest>,
) -> Phase08Result<PostingResult> {
    validate_idempotent(request)?;
    validate_source(&request.payload)?;

    if let Some(existing) = tx
        .query_row(
            r#"SELECT e.id,e.request_hash_sha256,a.id
               FROM journal_entries e JOIN posting_attempts a ON a.result_entry_id=e.id AND a.status='SUCCEEDED'
               WHERE e.company_id=?1 AND e.source_event_type=?2 AND e.source_event_id=?3
               ORDER BY a.attempt_number LIMIT 1"#,
            params![context.company_id, request.payload.source_event_type, request.payload.source_event_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()?
    {
        if existing.1.as_deref() == Some(request.request_hash_sha256.as_str()) {
            return Ok(PostingResult { journal_entry_id: existing.0, posting_attempt_id: existing.2, replayed: true });
        }
        return Err(Phase08Error::new(
            "ACCOUNTING_IDEMPOTENCY_CONFLICT",
            "This source event was already posted with different content.",
            false,
        ));
    }

    let enabled: i64 = tx.query_row(
        "SELECT COALESCE((SELECT is_enabled FROM accounting_setups WHERE company_id=?1),0)",
        [&context.company_id],
        |row| row.get(0),
    )?;
    if enabled == 0 {
        return Err(Phase08Error::new("ACCOUNTING_DISABLED", "Enable accounting before posting this source event.", false));
    }

    let (year_id, period_id) = resolve_open_period(tx, &context.company_id, &request.payload.event_date)?;
    let rule = select_rule(tx, &context.company_id, &request.payload)?;
    let mut generated = Vec::new();
    for line in &rule.lines {
        let amount = request.payload.components_minor.get(&line.component).copied().unwrap_or(0);
        if amount < 0 { return Err(Phase08Error::new("NEGATIVE_POSTING_COMPONENT", "Posting components must be non-negative fixed-point minor units.", false)); }
        if amount == 0 { continue; }
        require_active_postable_account(tx, &context.company_id, &line.account_id)?;
        generated.push((
            line.line_number,
            line.account_id.clone(),
            if line.partner_dimension { request.payload.partner_id.clone() } else { None },
            if line.product_dimension { request.payload.product_id.clone() } else { None },
            line.description.clone(),
            if line.side == "DEBIT" { amount } else { 0 },
            if line.side == "CREDIT" { amount } else { 0 },
        ));
    }
    validate_generated_lines(&generated)?;

    let entry_id = new_id();
    let attempt_id = new_id();
    let now = now_iso()?;
    let attempt_number: i64 = tx.query_row(
        "SELECT COALESCE(MAX(attempt_number),0)+1 FROM posting_attempts WHERE company_id=?1 AND source_event_type=?2 AND source_event_id=?3",
        params![context.company_id,request.payload.source_event_type,request.payload.source_event_id],
        |row| row.get(0),
    )?;
    let entry_number = next_entry_number(tx, &context.company_id, &year_id, &rule.journal_id, &request.payload.event_date)?;
    tx.execute(
        r#"INSERT INTO journal_entries(
            id,company_id,fiscal_year_id,fiscal_period_id,accounting_journal_id,source_document_id,reversal_of_entry_id,
            entry_number,entry_date,status,source_event_type,source_event_id,idempotency_key,memo,posted_at,posted_by,
            created_at,created_by,updated_at,updated_by,row_version,request_hash_sha256,posting_rule_id,reversal_reason)
            VALUES (?1,?2,?3,?4,?5,?6,NULL,?7,?8,'DRAFT',?9,?10,?11,?12,NULL,NULL,?13,?14,?13,?14,1,?15,?16,NULL)"#,
        params![entry_id,context.company_id,year_id,period_id,rule.journal_id,request.payload.source_document_id,
            entry_number,request.payload.event_date,request.payload.source_event_type,request.payload.source_event_id,
            request.idempotency_key,request.payload.memo,now,context.user_id,request.request_hash_sha256,rule.id],
    )?;
    if request.payload.inject_failure_after_header {
        return Err(Phase08Error::new("INJECTED_POSTING_FAILURE", "The accounting posting failed before journal lines were committed.", true));
    }
    for (line_number, account_id, partner_id, product_id, description, debit, credit) in &generated {
        tx.execute(
            r#"INSERT INTO journal_entry_lines(id,company_id,journal_entry_id,account_id,partner_id,product_id,line_number,description,debit_minor,credit_minor,created_at,created_by)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)"#,
            params![new_id(),context.company_id,entry_id,account_id,partner_id,product_id,line_number,description,debit,credit,now,context.user_id],
        )?;
    }
    tx.execute("UPDATE journal_entries SET status='POSTED',posted_at=?1,posted_by=?2,updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE id=?3 AND status='DRAFT'",params![now,context.user_id,entry_id])?;
    let retry_of_attempt_id: Option<String> = tx.query_row(
        "SELECT id FROM posting_attempts WHERE company_id=?1 AND source_event_type=?2 AND source_event_id=?3 ORDER BY attempt_number DESC LIMIT 1",
        params![context.company_id, request.payload.source_event_type, request.payload.source_event_id],
        |row| row.get(0),
    ).optional()?;
    tx.execute(
        r#"INSERT INTO posting_attempts(id,company_id,result_entry_id,retry_of_attempt_id,source_event_type,source_event_id,idempotency_key,attempt_number,status,error_code,error_message,started_at,completed_at,request_hash_sha256,recorded_at)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'SUCCEEDED',NULL,NULL,?9,?9,?10,?9)"#,
        params![attempt_id,context.company_id,entry_id,retry_of_attempt_id,request.payload.source_event_type,request.payload.source_event_id,request.idempotency_key,attempt_number,now,request.request_hash_sha256],
    )?;
    Ok(PostingResult { journal_entry_id: entry_id, posting_attempt_id: attempt_id, replayed: false })
}

pub fn record_failed_attempt_after_rollback(
    connection: &mut Connection,
    context: &Phase06AuthContext,
    request: &Idempotent<SourceEventRequest>,
    error: &Phase08Error,
) -> Phase08Result<String> {
    validate_idempotent(request)?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let attempt_id = new_id();
    let now = now_iso()?;
    let attempt_number: i64 = tx.query_row(
        "SELECT COALESCE(MAX(attempt_number),0)+1 FROM posting_attempts WHERE company_id=?1 AND source_event_type=?2 AND source_event_id=?3",
        params![context.company_id,request.payload.source_event_type,request.payload.source_event_id],
        |row| row.get(0),
    )?;
    let retry_of_attempt_id: Option<String> = tx.query_row(
        "SELECT id FROM posting_attempts WHERE company_id=?1 AND source_event_type=?2 AND source_event_id=?3 ORDER BY attempt_number DESC LIMIT 1",
        params![context.company_id, request.payload.source_event_type, request.payload.source_event_id],
        |row| row.get(0),
    ).optional()?;
    tx.execute(
        r#"INSERT INTO posting_attempts(id,company_id,result_entry_id,retry_of_attempt_id,source_event_type,source_event_id,idempotency_key,attempt_number,status,error_code,error_message,started_at,completed_at,request_hash_sha256,recorded_at)
           VALUES (?1,?2,NULL,?3,?4,?5,?6,?7,'FAILED',?8,NULL,?9,?9,?10,?9)"#,
        params![attempt_id,context.company_id,retry_of_attempt_id,request.payload.source_event_type,request.payload.source_event_id,request.idempotency_key,attempt_number,error.code,now,request.request_hash_sha256],
    )?;
    tx.commit()?;
    Ok(attempt_id)
}
