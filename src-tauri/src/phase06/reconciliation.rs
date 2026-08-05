use std::collections::{BTreeMap, BTreeSet};

use rusqlite::params;

use super::{
    audit, authorize_transaction, begin_idempotency,
    dto::{IdempotentRequest, ReconciliationRow, ReconciliationView, StockQuery},
    error::{Phase06Error, Phase06Result}, finish_idempotency, new_id, now_iso, request_hash,
    IdempotencyStart, Phase06Service,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Rebuilt { on_hand:i64,reserved:i64,average:i64,last_movement:Option<String> }
type Key=(String,String,Option<String>);

impl Phase06Service {
    pub fn reconcile_stock_balances(&self)->Phase06Result<ReconciliationView>{
        let context=self.context(Some("stock.reconcile"))?;
        self.read(|connection|reconcile_connection(connection,&context.company_id,false))
    }
    pub fn rebuild_stock_balances(&self,request:IdempotentRequest<StockQuery>)->Phase06Result<ReconciliationView>{
        let context=self.context(Some("stock.reconcile"))?;let hash=request_hash(&request.payload)?;
        self.immediate(|tx|{
            authorize_transaction(tx, &context, "stock.reconcile")?;
            if let IdempotencyStart::Replayed(_)=begin_idempotency(tx,&context,"stock.balance.rebuild",&request.idempotency_key,&hash)?{
                let mut view=reconcile_connection(tx,&context.company_id,false)?;view.rebuilt=true;return Ok(view);
            }
            let rebuilt=rebuild_map(tx,&context.company_id)?;
            tx.execute("DELETE FROM stock_balances WHERE company_id=?1",[&context.company_id])?;
            let now=now_iso()?;
            for ((product,warehouse,location),row) in &rebuilt{
                let movement=row.last_movement.as_deref().ok_or_else(Phase06Error::internal)?;
                tx.execute("INSERT INTO stock_balances (id,company_id,product_id,warehouse_id,warehouse_location_id,last_movement_id,on_hand_scaled,reserved_scaled,available_scaled,average_cost_scaled,rebuilt_at,row_version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?7-?8,?9,?10,1)",params![new_id(),context.company_id,product,warehouse,location,movement,row.on_hand,row.reserved,row.average,now])?;
            }
            audit(tx,&context,"stock.balance.rebuild","stock_balance_projection",&context.company_id,None)?;
            finish_idempotency(tx,&context,"stock.balance.rebuild",&request.idempotency_key,"stock_balance_projection",&context.company_id)?;
            let mut view=reconcile_connection(tx,&context.company_id,false)?;view.rebuilt=true;Ok(view)
        })
    }
}

fn rebuild_map(connection:&rusqlite::Connection,company:&str)->Phase06Result<BTreeMap<Key,Rebuilt>>{
    let mut result=BTreeMap::<Key,Rebuilt>::new();
    let mut stmt=connection.prepare("SELECT id,product_id,warehouse_id,warehouse_location_id,quantity_delta_scaled,average_cost_after_scaled FROM stock_movements WHERE company_id=?1 ORDER BY occurred_at,id")?;
    let movements=stmt.query_map([company],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,i64>(4)?,r.get::<_,Option<i64>>(5)?)))?.collect::<Result<Vec<_>,_>>()?;
    for (id,product,warehouse,location,delta,average) in movements{
        let aggregate=result.entry((product.clone(),warehouse.clone(),None)).or_default();aggregate.on_hand=aggregate.on_hand.checked_add(delta).ok_or_else(||Phase06Error::new("NUMERIC_OVERFLOW","The calculation exceeds supported fixed-point limits."))?;aggregate.average=average.unwrap_or(aggregate.average);aggregate.last_movement=Some(id.clone());
        if let Some(location)=location{let local=result.entry((product,warehouse,Some(location))).or_default();local.on_hand=local.on_hand.checked_add(delta).ok_or_else(||Phase06Error::new("NUMERIC_OVERFLOW","The calculation exceeds supported fixed-point limits."))?;local.average=average.unwrap_or(local.average);local.last_movement=Some(id);}
    }
    let mut stmt=connection.prepare("SELECT product_id,warehouse_id,warehouse_location_id,SUM(reserved_quantity_scaled) FROM stock_reservations WHERE company_id=?1 AND status IN ('ACTIVE','PARTIALLY_CONSUMED') GROUP BY product_id,warehouse_id,warehouse_location_id")?;
    let reservations=stmt.query_map([company],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,i64>(3)?)))?.collect::<Result<Vec<_>,_>>()?;
    for (product,warehouse,location,qty) in reservations{
        result.entry((product.clone(),warehouse.clone(),None)).or_default().reserved+=qty;
        if let Some(location)=location{result.entry((product,warehouse,Some(location))).or_default().reserved+=qty;}
    }
    let averages=result.iter().filter_map(|((p,w,l),v)|l.is_none().then_some(((p.clone(),w.clone()),v.average))).collect::<BTreeMap<_,_>>();
    for ((product,warehouse,location),row) in &mut result{if location.is_some(){row.average=*averages.get(&(product.clone(),warehouse.clone())).unwrap_or(&row.average);}}
    Ok(result)
}

fn reconcile_connection(connection:&rusqlite::Connection,company:&str,rebuilt_flag:bool)->Phase06Result<ReconciliationView>{
    let rebuilt=rebuild_map(connection,company)?;let mut keys=rebuilt.keys().cloned().collect::<BTreeSet<_>>();
    let mut projection=BTreeMap::<Key,(i64,i64,i64)>::new();let mut stmt=connection.prepare("SELECT product_id,warehouse_id,warehouse_location_id,on_hand_scaled,reserved_scaled,average_cost_scaled FROM stock_balances WHERE company_id=?1")?;
    for row in stmt.query_map([company],|r|Ok(((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?),(r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,i64>(5)?)))?{let (key,value)=row?;keys.insert(key.clone());projection.insert(key,value);}
    let rows=keys.into_iter().map(|key|{let expected=rebuilt.get(&key).cloned().unwrap_or_default();let actual=projection.get(&key).copied().unwrap_or_default();ReconciliationRow{product_id:key.0,warehouse_id:key.1,warehouse_location_id:key.2,projection_on_hand_scaled:actual.0,rebuilt_on_hand_scaled:expected.on_hand,projection_reserved_scaled:actual.1,rebuilt_reserved_scaled:expected.reserved,projection_average_cost_scaled:actual.2,rebuilt_average_cost_scaled:expected.average,matches:actual==(expected.on_hand,expected.reserved,expected.average)}}).collect::<Vec<_>>();
    let mismatch_count=rows.iter().filter(|row|!row.matches).count();Ok(ReconciliationView{rows,mismatch_count,rebuilt:rebuilt_flag})
}
