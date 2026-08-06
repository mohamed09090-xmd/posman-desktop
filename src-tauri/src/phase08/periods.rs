use rusqlite::{params, OptionalExtension};

use super::{
    dto::{EntityVersion, FiscalPeriodView, PeriodActionInput},
    error::{Phase08Error, Phase08Result},
    service::{new_id, now_iso},
    Phase08Service,
};

impl Phase08Service {
    pub fn list_fiscal_periods(&self, _: ()) -> Phase08Result<Vec<FiscalPeriodView>> {
        let context=self.context(Some("accounting.read"))?;
        self.read(|c| {
            let mut s=c.prepare("SELECT id,fiscal_year_id,period_number,name,starts_on,ends_on,status,row_version FROM fiscal_periods WHERE company_id=?1 ORDER BY starts_on")?;
            let rows=s.query_map([context.company_id],|r| Ok(FiscalPeriodView{id:r.get(0)?,fiscal_year_id:r.get(1)?,period_number:r.get(2)?,name:r.get(3)?,starts_on:r.get(4)?,ends_on:r.get(5)?,status:r.get(6)?,row_version:r.get(7)?}))?;
            rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
        })
    }

    pub fn close_fiscal_period(&self, input: PeriodActionInput) -> Phase08Result<EntityVersion> { self.change_period(input,"CLOSED","CLOSED") }
    pub fn reopen_fiscal_period(&self, input: PeriodActionInput) -> Phase08Result<EntityVersion> { self.change_period(input,"OPEN","REOPENED") }
    fn change_period(&self,input:PeriodActionInput,new_status:&str,event:&str)->Phase08Result<EntityVersion>{
        let context=self.context(Some("accounting.period.manage"))?;
        if input.reason.trim().is_empty(){return Err(Phase08Error::validation("A period-change reason is required."));}
        self.immediate(|tx|{
            let old:String=tx.query_row("SELECT status FROM fiscal_periods WHERE id=?1 AND company_id=?2",params![input.fiscal_period_id,context.company_id],|r|r.get(0)).optional()?.ok_or_else(||Phase08Error::new("FISCAL_PERIOD_NOT_FOUND","Fiscal period was not found.",false))?;
            if old=="LOCKED" {return Err(Phase08Error::new("FISCAL_PERIOD_LOCKED","A locked fiscal period cannot be changed.",false));}
            let now=now_iso()?;
            let changed=tx.execute("UPDATE fiscal_periods SET status=?1,closed_at=CASE WHEN ?1='CLOSED' THEN ?2 ELSE NULL END,closed_by=CASE WHEN ?1='CLOSED' THEN ?3 ELSE NULL END,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5 AND row_version=?6",
                params![new_status,now,context.user_id,input.fiscal_period_id,context.company_id,input.row_version])?;
            if changed!=1{return Err(Phase08Error::new("FISCAL_PERIOD_CONFLICT","The fiscal period changed; reload and retry.",true));}
            tx.execute("INSERT INTO fiscal_period_events(id,company_id,fiscal_period_id,event_type,reason,previous_status,new_status,occurred_at,occurred_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![new_id(),context.company_id,input.fiscal_period_id,event,input.reason,old,new_status,now,context.user_id])?;
            Ok(EntityVersion{id:input.fiscal_period_id,row_version:input.row_version+1})
        })
    }
}
