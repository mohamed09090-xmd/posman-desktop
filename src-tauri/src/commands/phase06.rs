use tauri::State;

use crate::phase06::{dto::*, error::{Phase06Error, Phase06Result}, Phase06Service};

async fn run<T:Send+'static>(service:Phase06Service,op:impl FnOnce(Phase06Service)->Phase06Result<T>+Send+'static)->Phase06Result<T>{
    tauri::async_runtime::spawn_blocking(move||op(service)).await.map_err(|_|Phase06Error::internal())?
}

macro_rules! command {
    ($name:ident,$request:ty,$result:ty,$method:ident) => {
        #[tauri::command]
        pub async fn $name(state:State<'_,Phase06Service>,request:$request)->Phase06Result<$result>{run(state.inner().clone(),move|service|service.$method(request)).await}
    };
}

command!(list_stock_balances,StockQuery,Vec<StockBalanceView>,list_stock_balances);
command!(list_stock_movements,StockQuery,Vec<MovementView>,list_stock_movements);
command!(create_opening_stock,OpeningDraftRequest,EntityResult,create_opening_stock);
command!(review_opening_stock,DocumentActionRequest,EntityResult,review_opening_stock);
command!(post_opening_stock,IdempotentRequest<DocumentActionRequest>,EntityResult,post_opening_stock);
command!(post_stock_adjustment,IdempotentRequest<AdjustmentRequest>,EntityResult,post_adjustment);
command!(post_stock_transfer,IdempotentRequest<TransferRequest>,EntityResult,post_transfer);
command!(create_inventory_count,CreateCountRequest,CountView,create_inventory_count);
command!(update_inventory_count,UpdateCountRequest,CountView,update_inventory_count);
command!(review_inventory_count,DocumentActionRequest,CountView,review_inventory_count);
command!(post_inventory_count,IdempotentRequest<DocumentActionRequest>,EntityResult,post_inventory_count);
command!(get_inventory_count,String,CountView,get_inventory_count);
command!(create_stock_reservation,IdempotentRequest<ReservationRequest>,EntityResult,create_reservation);
command!(release_stock_reservation,IdempotentRequest<ReservationActionRequest>,EntityResult,release_reservation);
command!(consume_stock_reservation,IdempotentRequest<ReservationActionRequest>,EntityResult,consume_reservation);
command!(cancel_stock_reservation,IdempotentRequest<ReservationActionRequest>,EntityResult,cancel_reservation);

#[tauri::command]
pub async fn list_active_stock_reservations(state:State<'_,Phase06Service>)->Phase06Result<Vec<ReservationView>>{run(state.inner().clone(),|service|service.list_active_reservations()).await}
#[tauri::command]
pub async fn reconcile_stock_balances(state:State<'_,Phase06Service>)->Phase06Result<ReconciliationView>{run(state.inner().clone(),|service|service.reconcile_stock_balances()).await}
command!(rebuild_stock_balances,IdempotentRequest<StockQuery>,ReconciliationView,rebuild_stock_balances);
command!(create_purchase_order,CreatePurchaseOrderRequest,EntityResult,create_purchase_order);
command!(update_purchase_order,UpdatePurchaseOrderRequest,EntityResult,update_purchase_order);
command!(confirm_purchase_order,IdempotentRequest<DocumentActionRequest>,EntityResult,confirm_purchase_order);
command!(cancel_purchase_order,IdempotentRequest<DocumentActionRequest>,EntityResult,cancel_purchase_order);
command!(hold_purchase_order,IdempotentRequest<DocumentActionRequest>,EntityResult,hold_purchase_order);
command!(create_purchase_receipt,CreateReceiptRequest,EntityResult,create_purchase_receipt);
command!(post_purchase_receipt,IdempotentRequest<DocumentActionRequest>,EntityResult,post_purchase_receipt);
command!(create_purchase_invoice,CreateInvoiceRequest,EntityResult,create_purchase_invoice);
command!(post_purchase_invoice,IdempotentRequest<DocumentActionRequest>,EntityResult,post_purchase_invoice);
command!(direct_receive_and_invoice,IdempotentRequest<DirectReceiveInvoiceRequest>,EntityResult,direct_receive_and_invoice);
command!(post_purchase_return,IdempotentRequest<PurchaseReturnRequest>,EntityResult,post_purchase_return);
command!(list_purchasing_documents,DocumentQuery,Vec<DocumentView>,list_purchasing_documents);
command!(get_purchasing_document,String,DocumentView,get_purchasing_document);
