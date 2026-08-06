pub(crate) fn resolve_open_period(tx:&Transaction<'_>,company:&str,date:&str)->Phase08Result<(String,String)>{
    let mut s=tx.prepare("SELECT fy.id,fp.id,fy.status,fp.status FROM fiscal_periods fp JOIN fiscal_years fy ON fy.id=fp.fiscal_year_id AND fy.company_id=fp.company_id WHERE fp.company_id=?1 AND fp.starts_on<=?2 AND fp.ends_on>=?2 ORDER BY fp.period_number")?;
    let rows = s.query_map(params![company,date],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?)))?.collect::<Result<Vec<_>,_>>()?;
    if rows.len()!=1{return Err(Phase08Error::new("FISCAL_PERIOD_NOT_FOUND","Exactly one fiscal period must contain the posting date.",false));}
    if rows[0].2!="OPEN"||rows[0].3!="OPEN"{return Err(Phase08Error::new("FISCAL_PERIOD_CLOSED","The posting date belongs to a closed or locked fiscal period.",false));}
    Ok((rows[0].0.clone(),rows[0].1.clone()))
}

fn next_entry_number(tx:&Transaction<'_>,company:&str,year:&str,journal:&str,date:&str)->Phase08Result<String>{
    let count:i64=tx.query_row("SELECT COUNT(*)+1 FROM journal_entries WHERE company_id=?1 AND fiscal_year_id=?2 AND accounting_journal_id=?3",params![company,year,journal],|r|r.get(0))?;
    Ok(format!("{}-{:06}",date.replace('-', ""),count))
}

type GeneratedPostingLine = (i64,String,Option<String>,Option<String>,String,i64,i64);

fn validate_generated_lines(lines:&[GeneratedPostingLine])->Phase08Result<()> {
    if lines.len()<2{return Err(Phase08Error::new("UNBALANCED_GENERATED_ENTRY","Generated journal entry requires at least two non-zero lines.",false));}
    let debit = lines.iter().try_fold(0_i64, |total, line| total.checked_add(line.5).ok_or_else(Phase08Error::internal))?;
    let credit = lines.iter().try_fold(0_i64, |total, line| total.checked_add(line.6).ok_or_else(Phase08Error::internal))?;
    if debit<=0||debit!=credit{return Err(Phase08Error::new("UNBALANCED_GENERATED_ENTRY","Generated journal entry is not balanced.",false));}
    Ok(())
}
fn validate_manual_lines(input:&ManualJournalInput)->Phase08Result<()> {
    if input.lines.len()<2{return Err(Phase08Error::new("MANUAL_JOURNAL_LINES_REQUIRED","A manual journal requires at least two lines.",false));}
    let mut debit=0_i64;let mut credit=0_i64;
    for line in &input.lines {
        if line.description.trim().is_empty(){return Err(Phase08Error::validation("Every manual journal line requires a description."));}
        if !((line.debit_minor>0&&line.credit_minor==0)||(line.credit_minor>0&&line.debit_minor==0)){return Err(Phase08Error::new("MANUAL_JOURNAL_LINE_INVALID","Each line must have either a debit or a credit amount.",false));}
        debit=debit.checked_add(line.debit_minor).ok_or_else(Phase08Error::internal)?;
        credit=credit.checked_add(line.credit_minor).ok_or_else(Phase08Error::internal)?;
    }
    if debit!=credit{return Err(Phase08Error::new("MANUAL_JOURNAL_UNBALANCED","The manual journal is not balanced.",false));}
    Ok(())
}
fn validate_entry_balance(tx:&Transaction<'_>,id:&str)->Phase08Result<()> {
    let (count,debit,credit):(i64,i64,i64)=tx.query_row("SELECT COUNT(*),COALESCE(SUM(debit_minor),0),COALESCE(SUM(credit_minor),0) FROM journal_entry_lines WHERE journal_entry_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if count<2||debit<=0||debit!=credit{return Err(Phase08Error::new("JOURNAL_UNBALANCED","The journal entry requires at least two balanced lines.",false));}
    Ok(())
}
pub(crate) fn validate_hash(hash:&str)->Phase08Result<()> {
    if hash.len()!=64||!hash.bytes().all(|b|b.is_ascii_hexdigit()){return Err(Phase08Error::validation("Request hash must be a 64-character SHA-256 hexadecimal value."));}
    Ok(())
}
pub(crate) fn validate_idempotent<T: serde::Serialize>(request: &Idempotent<T>) -> Phase08Result<()> {
    if !(8..=160).contains(&request.idempotency_key.trim().len()) {
        return Err(Phase08Error::validation("Idempotency key length is invalid."));
    }
    validate_hash(&request.request_hash_sha256)?;
    let expected = request_hash(&request.payload)?;
    if !expected.eq_ignore_ascii_case(&request.request_hash_sha256) {
        return Err(Phase08Error::new(
            "REQUEST_HASH_MISMATCH",
            "The request hash does not match the submitted payload.",
            false,
        ));
    }
    Ok(())
}

pub(crate) fn normalize_idempotent<T: serde::Serialize>(
    mut request: Idempotent<T>,
) -> Phase08Result<Idempotent<T>> {
    if !(8..=160).contains(&request.idempotency_key.trim().len()) {
        return Err(Phase08Error::validation("Idempotency key length is invalid."));
    }
    validate_hash(&request.request_hash_sha256)?;
    let expected = request_hash(&request.payload)?;
    let server_hash_requested = request.request_hash_sha256.bytes().all(|byte| byte == b'0');
    if !server_hash_requested && !expected.eq_ignore_ascii_case(&request.request_hash_sha256) {
        return Err(Phase08Error::new(
            "REQUEST_HASH_MISMATCH",
            "The request hash does not match the submitted payload.",
            false,
        ));
    }
    request.request_hash_sha256 = expected;
    Ok(request)
}
fn validate_source(source:&SourceEventRequest)->Phase08Result<()> {
    if source.source_event_type.trim().is_empty()||source.source_event_id.trim().is_empty(){return Err(Phase08Error::validation("Source event type and id are required."));}
    if source.components_minor.values().any(|v|*v<0){return Err(Phase08Error::validation("Source components must use non-negative fixed-point minor units."));}
    Ok(())
}

pub fn request_hash<T: serde::Serialize>(value: &T) -> Phase08Result<String> {
    fn canonical(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut keys = map.into_iter().collect::<Vec<_>>();
                keys.sort_by(|left, right| left.0.cmp(&right.0));
                let mut result = serde_json::Map::new();
                for (key, value) in keys {
                    if !value.is_null() {
                        result.insert(key, canonical(value));
                    }
                }
                serde_json::Value::Object(result)
            }
            serde_json::Value::Array(values) => {
                serde_json::Value::Array(values.into_iter().map(canonical).collect())
            }
            other => other,
        }
    }
    let value = serde_json::to_value(value).map_err(|_| Phase08Error::internal())?;
    let bytes = serde_json::to_vec(&canonical(value)).map_err(|_| Phase08Error::internal())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn list_entries(c:&Connection,company:&str,id:Option<&str>)->Phase08Result<Vec<JournalEntryView>>{
    let mut sql="SELECT id,entry_number,entry_date,status,source_event_type,source_event_id,reversal_of_entry_id,memo FROM journal_entries WHERE company_id=?1".to_owned();
    if id.is_some(){sql.push_str(" AND id=?2");}sql.push_str(" ORDER BY entry_date DESC,entry_number DESC");
    let mut s=c.prepare(&sql)?;
    let header_rows = if let Some(id)=id { s.query_map(params![company,id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,Option<String>>(7)?)))?.collect::<Result<Vec<_>,_>>()? }
        else { s.query_map([company],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,Option<String>>(7)?)))?.collect::<Result<Vec<_>,_>>()? };
    let mut result=Vec::new();
    for h in header_rows{
        let mut ls=c.prepare("SELECT l.account_id,a.code,l.description,l.debit_minor,l.credit_minor FROM journal_entry_lines l JOIN accounts a ON a.id=l.account_id WHERE l.journal_entry_id=?1 ORDER BY l.line_number")?;
        let lines=ls.query_map([&h.0],|r|Ok(JournalLineView{account_id:r.get(0)?,account_code:r.get(1)?,description:r.get(2)?,debit_minor:r.get(3)?,credit_minor:r.get(4)?}))?.collect::<Result<Vec<_>,_>>()?;
        let debit = lines.iter().try_fold(0_i64, |total, line| total.checked_add(line.debit_minor).ok_or_else(Phase08Error::internal))?;
        let credit = lines.iter().try_fold(0_i64, |total, line| total.checked_add(line.credit_minor).ok_or_else(Phase08Error::internal))?;
        result.push(JournalEntryView{id:h.0,entry_number:h.1,entry_date:h.2,status:h.3,source_event_type:h.4,source_event_id:h.5,reversal_of_entry_id:h.6,memo:h.7,debit_total_minor:debit,credit_total_minor:credit,lines});
    }
    Ok(result)
}
