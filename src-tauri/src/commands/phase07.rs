use tauri::State;

use crate::phase07::{
    dto::*,
    error::{Phase07Error, Phase07Result},
    Phase07Service,
};

async fn run<T: Send + 'static>(
    service: Phase07Service,
    operation: impl FnOnce(Phase07Service) -> Phase07Result<T> + Send + 'static,
) -> Phase07Result<T> {
    tauri::async_runtime::spawn_blocking(move || operation(service))
        .await
        .map_err(|_| Phase07Error::internal())?
}

macro_rules! command {
    ($name:ident,$request:ty,$result:ty,$method:ident) => {
        #[tauri::command]
        pub async fn $name(
            state: State<'_, Phase07Service>,
            request: $request,
        ) -> Phase07Result<$result> {
            run(state.inner().clone(), move |service| {
                service.$method(request)
            })
            .await
        }
    };
}

command!(
    create_sales_order,
    CreateSalesOrderRequest,
    EntityResult,
    create_sales_order
);
command!(
    update_sales_order,
    UpdateSalesOrderRequest,
    EntityResult,
    update_sales_order
);
command!(
    confirm_sales_order,
    IdempotentRequest<SalesOrderActionRequest>,
    EntityResult,
    confirm_sales_order
);
command!(
    hold_sales_order,
    IdempotentRequest<SalesOrderActionRequest>,
    EntityResult,
    hold_sales_order
);
command!(
    resume_sales_order,
    IdempotentRequest<SalesOrderActionRequest>,
    EntityResult,
    resume_sales_order
);
command!(
    cancel_sales_order,
    IdempotentRequest<SalesOrderActionRequest>,
    EntityResult,
    cancel_sales_order
);
command!(
    deliver_sales_order,
    IdempotentRequest<DeliverSalesOrderRequest>,
    EntityResult,
    deliver_sales_order
);
command!(
    invoice_sales_delivery,
    IdempotentRequest<InvoiceDeliveryRequest>,
    EntityResult,
    invoice_sales_delivery
);
command!(
    direct_sale,
    IdempotentRequest<DirectSaleRequest>,
    SalesFlowResult,
    direct_sale
);
command!(
    post_sales_return,
    IdempotentRequest<SalesReturnRequest>,
    SalesFlowResult,
    post_sales_return
);
command!(
    list_sales_documents,
    DocumentQuery,
    Vec<DocumentView>,
    list_sales_documents
);
command!(get_sales_document, String, DocumentView, get_sales_document);
command!(
    get_sales_line_availability,
    String,
    Vec<SalesLineAvailability>,
    get_sales_line_availability
);

#[tauri::command]
pub async fn get_sales_summary(state: State<'_, Phase07Service>) -> Phase07Result<SalesSummary> {
    run(state.inner().clone(), |service| service.get_sales_summary()).await
}
