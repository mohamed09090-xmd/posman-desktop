impl Phase08Service {
    pub fn post_source_event(&self, request: Idempotent<SourceEventRequest>) -> Phase08Result<PostingResult> {
        let request = normalize_idempotent(request)?;
        let context = self.context(Some("accounting.manual.post"))?;
        let mut connection = self.phase05.phase06_open().map_err(|_| Phase08Error::internal())?;
        let outcome = {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            match post_source_event_in_tx(&tx, &context, &request) {
                Ok(result) => { tx.commit()?; Ok(result) }
                Err(error) => Err(error),
            }
        };
        match outcome {
            Ok(result) => Ok(result),
            Err(error) => {
                record_failed_attempt_after_rollback(&mut connection, &context, &request, &error)?;
                Err(error)
            }
        }
    }

    pub fn retry_posting_attempt(&self, request: Idempotent<SourceEventRequest>) -> Phase08Result<PostingResult> {
        self.post_source_event(request)
    }

    pub fn list_accounting_posting_queue(&self, _: ()) -> Phase08Result<Vec<PostingAttemptView>> {
        let context=self.context(Some("accounting.read"))?;
        self.read(|c| {
            let mut s=c.prepare("SELECT id,source_event_type,source_event_id,attempt_number,status,error_code,started_at,completed_at FROM posting_attempts WHERE company_id=?1 ORDER BY started_at DESC,attempt_number DESC")?;
            let rows=s.query_map([context.company_id],|r|Ok(PostingAttemptView{id:r.get(0)?,source_event_type:r.get(1)?,source_event_id:r.get(2)?,attempt_number:r.get(3)?,status:r.get(4)?,error_code:r.get(5)?,started_at:r.get(6)?,completed_at:r.get(7)?}))?;
            rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
        })
    }

    pub fn create_manual_journal_entry(&self, input: ManualJournalInput) -> Phase08Result<EntityVersion> {
        self.save_manual(input,false)
    }
    pub fn update_manual_journal_entry(&self, input: ManualJournalInput) -> Phase08Result<EntityVersion> {
        self.save_manual(input,true)
    }
    fn save_manual(&self,input:ManualJournalInput,update:bool)->Phase08Result<EntityVersion>{
        let context=self.context(Some("accounting.manual.post"))?;
        validate_manual_lines(&input)?;
        self.immediate(|tx|{
            require_company_row(tx,"accounting_journals",&input.accounting_journal_id,&context.company_id,"JOURNAL_NOT_FOUND")?;
            let (year,period)=resolve_open_period(tx,&context.company_id,&input.entry_date)?;
            let now=now_iso()?;
            let id=input.id.clone().unwrap_or_else(new_id);
            if update {
                let version=input.row_version.ok_or_else(||Phase08Error::validation("Manual journal row version is required."))?;
                let changed=tx.execute("UPDATE journal_entries SET fiscal_year_id=?1,fiscal_period_id=?2,accounting_journal_id=?3,entry_date=?4,memo=?5,updated_at=?6,updated_by=?7,row_version=row_version+1 WHERE id=?8 AND company_id=?9 AND status='DRAFT' AND row_version=?10",
                    params![year,period,input.accounting_journal_id,input.entry_date,input.memo,now,context.user_id,id,context.company_id,version])?;
                if changed!=1{return Err(Phase08Error::new("MANUAL_JOURNAL_CONFLICT","Only the current draft version can be edited.",true));}
                tx.execute("DELETE FROM journal_entry_lines WHERE journal_entry_id=?1",[&id])?;
            } else {
                let number=next_entry_number(tx,&context.company_id,&year,&input.accounting_journal_id,&input.entry_date)?;
                tx.execute("INSERT INTO journal_entries(id,company_id,fiscal_year_id,fiscal_period_id,accounting_journal_id,source_document_id,reversal_of_entry_id,entry_number,entry_date,status,source_event_type,source_event_id,idempotency_key,memo,created_at,created_by,updated_at,updated_by,row_version) VALUES (?1,?2,?3,?4,?5,NULL,NULL,?6,?7,'DRAFT','MANUAL_JOURNAL',?1,?1,?8,?9,?10,?9,?10,1)",
                    params![id,context.company_id,year,period,input.accounting_journal_id,number,input.entry_date,input.memo,now,context.user_id])?;
            }
            for (idx,line) in input.lines.iter().enumerate(){
                require_active_postable_account(tx,&context.company_id,&line.account_id)?;
                tx.execute("INSERT INTO journal_entry_lines(id,company_id,journal_entry_id,account_id,partner_id,product_id,line_number,description,debit_minor,credit_minor,created_at,created_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                    params![new_id(),context.company_id,id,line.account_id,line.partner_id,line.product_id,(idx+1)as i64,line.description,line.debit_minor,line.credit_minor,now,context.user_id])?;
            }
            Ok(EntityVersion{id,row_version:input.row_version.unwrap_or(0)+1})
        })
    }

    pub fn post_manual_journal_entry(&self, id: String) -> Phase08Result<EntityVersion> {
        let context=self.context(Some("accounting.manual.post"))?;
        self.immediate(|tx|{
            let (status,date,version):(String,String,i64)=tx.query_row("SELECT status,entry_date,row_version FROM journal_entries WHERE id=?1 AND company_id=?2",params![id,context.company_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?.ok_or_else(||Phase08Error::new("JOURNAL_ENTRY_NOT_FOUND","Journal entry was not found.",false))?;
            if status!="DRAFT"{return Err(Phase08Error::new("JOURNAL_NOT_DRAFT","Only a draft journal can be posted.",false));}
            resolve_open_period(tx,&context.company_id,&date)?;
            validate_entry_balance(tx,&id)?;
            let now=now_iso()?;
            tx.execute("UPDATE journal_entries SET status='POSTED',posted_at=?1,posted_by=?2,updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE id=?3 AND status='DRAFT'",params![now,context.user_id,id])?;
            Ok(EntityVersion{id,row_version:version+1})
        })
    }

    pub fn reverse_journal_entry(&self, input: ReverseJournalRequest) -> Phase08Result<EntityVersion> {
        let context=self.context(Some("accounting.reverse"))?;
        if input.reason.trim().is_empty(){return Err(Phase08Error::validation("A reversal reason is required."));}
        self.immediate(|tx| reverse_entry_in_tx(tx,&context,&input.journal_entry_id,&input.reversal_date,&input.reason))
    }

    pub fn list_journal_entries(&self, _: ()) -> Phase08Result<Vec<JournalEntryView>> {
        let context=self.context(Some("accounting.read"))?;
        self.read(|c| list_entries(c,&context.company_id,None))
    }

    pub fn get_journal_entry(&self,id:String)->Phase08Result<JournalEntryView>{
        let context=self.context(Some("accounting.read"))?;
        self.read(|c| list_entries(c,&context.company_id,Some(&id))?.into_iter().next().ok_or_else(||Phase08Error::new("JOURNAL_ENTRY_NOT_FOUND","Journal entry was not found.",false)))
    }
}
