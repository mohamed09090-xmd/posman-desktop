mod application;
mod commands;
mod error;
mod infrastructure;
mod phase05;
mod phase06;
mod phase07;
mod phase08;
mod phase09;

use std::{error::Error, path::PathBuf};

pub use application::RuntimeStatus;
use error::RuntimeError;
use infrastructure::{database::RuntimeDatabase, paths::RuntimePaths};
use phase05::Phase05Service;
use phase06::Phase06Service;
use phase07::Phase07Service;
use phase08::Phase08Service;
use phase09::Phase09Service;
use tauri::{Manager, Runtime};

#[derive(Clone)]
pub struct RuntimeService {
    database: RuntimeDatabase,
}

impl RuntimeService {
    fn initialize(root: PathBuf) -> Result<Self, RuntimeError> {
        let paths = RuntimePaths::create_all(root)?;
        let database = RuntimeDatabase::initialize(&paths.database)?;
        Ok(Self { database })
    }

    pub fn status(&self) -> RuntimeStatus {
        self.database.status()
    }
}

#[derive(Clone)]
enum RuntimeRoot {
    SystemLocalData,
    #[cfg(test)]
    Explicit(PathBuf),
}

impl RuntimeRoot {
    fn resolve<R: Runtime>(&self, app: &tauri::AppHandle<R>) -> Result<PathBuf, RuntimeError> {
        match self {
            Self::SystemLocalData => app
                .path()
                .local_data_dir()
                .map(|root| root.join("POSMAN"))
                .map_err(|error| RuntimeError::PathResolution {
                    detail: error.to_string(),
                }),
            #[cfg(test)]
            Self::Explicit(root) => Ok(root.clone()),
        }
    }
}

fn configure_runtime<R: Runtime>(
    builder: tauri::Builder<R>,
    runtime_root: RuntimeRoot,
) -> tauri::Builder<R> {
    builder.setup(move |app| {
        let root = runtime_root
            .resolve(app.handle())
            .map_err(boxed_runtime_error)?;
        let phase09_paths = RuntimePaths::from_root(root.clone());
        let runtime = RuntimeService::initialize(root).map_err(boxed_runtime_error)?;
        let phase05 = Phase05Service::new(runtime.database.path())
            .map_err(|_| "POSMAN PHASE 05 service could not initialize")?;
        let phase06 = Phase06Service::new(phase05.clone())
            .map_err(|_| "POSMAN PHASE 06 service could not initialize")?;
        let phase07 = Phase07Service::new(phase05.clone())
            .map_err(|_| "POSMAN PHASE 07 service could not initialize")?;
        let phase08 = Phase08Service::new(phase05.clone())
            .map_err(|_| "POSMAN PHASE 08 service could not initialize")?;
        let phase09 = Phase09Service::new(phase05.clone(), phase09_paths)
            .map_err(|_| "POSMAN PHASE 09 service could not initialize")?;
        if !app.manage(runtime) {
            return Err("POSMAN runtime state was already managed".into());
        }
        if !app.manage(phase05) {
            return Err("POSMAN PHASE 05 state was already managed".into());
        }
        if !app.manage(phase06) {
            return Err("POSMAN PHASE 06 state was already managed".into());
        }
        if !app.manage(phase07) {
            return Err("POSMAN PHASE 07 state was already managed".into());
        }
        if !app.manage(phase08) {
            return Err("POSMAN PHASE 08 state was already managed".into());
        }
        if !app.manage(phase09) {
            return Err("POSMAN PHASE 09 state was already managed".into());
        }
        Ok(())
    })
}

fn configure_application(
    builder: tauri::Builder<tauri::Wry>,
    runtime_root: RuntimeRoot,
) -> tauri::Builder<tauri::Wry> {
    configure_runtime(builder, runtime_root).invoke_handler(tauri::generate_handler![
        commands::runtime::get_runtime_status,
        commands::phase05::get_setup_status,
        commands::phase05::load_setup_draft,
        commands::phase05::discard_setup_draft,
        commands::phase05::get_current_session,
        commands::phase05::logout,
        commands::phase05::lock_session,
        commands::phase05::rotate_recovery_code,
        commands::phase05::get_company_profile,
        commands::phase05::get_fiscal_setup,
        commands::phase05::list_document_sequences,
        commands::phase05::list_roles,
        commands::phase05::save_setup_draft,
        commands::phase05::complete_initial_setup,
        commands::phase05::login,
        commands::phase05::recover_admin_password,
        commands::phase05::unlock_session,
        commands::phase05::change_own_password,
        commands::phase05::update_company_profile,
        commands::phase05::update_fiscal_setup,
        commands::phase05::update_document_sequence,
        commands::phase05::list_users,
        commands::phase05::create_user,
        commands::phase05::update_user,
        commands::phase05::set_user_roles,
        commands::phase05::reset_user_password,
        commands::phase05::create_role,
        commands::phase05::update_role,
        commands::phase05::set_role_permissions,
        commands::phase05::list_products,
        commands::phase05::create_product,
        commands::phase05::update_product,
        commands::phase05::set_product_active,
        commands::phase05::set_product_price,
        commands::phase05::list_partners,
        commands::phase05::create_partner,
        commands::phase05::update_partner,
        commands::phase05::set_partner_active,
        commands::phase05::create_partner_address,
        commands::phase05::create_partner_contact,
        commands::phase05::list_units,
        commands::phase05::create_unit,
        commands::phase05::update_unit,
        commands::phase05::set_unit_active,
        commands::phase05::list_tax_rates,
        commands::phase05::create_tax_rate,
        commands::phase05::update_tax_rate,
        commands::phase05::set_tax_rate_active,
        commands::phase05::list_payment_terms,
        commands::phase05::create_payment_term,
        commands::phase05::update_payment_term,
        commands::phase05::set_payment_term_active,
        commands::phase05::list_payment_methods,
        commands::phase05::create_payment_method,
        commands::phase05::update_payment_method,
        commands::phase05::set_payment_method_active,
        commands::phase05::list_warehouses,
        commands::phase05::create_warehouse,
        commands::phase05::update_warehouse,
        commands::phase05::set_warehouse_active,
        commands::phase05::list_warehouse_locations,
        commands::phase05::create_warehouse_location,
        commands::phase05::update_warehouse_location,
        commands::phase05::set_warehouse_location_active,
        commands::phase05::list_product_families,
        commands::phase05::create_product_family,
        commands::phase05::update_product_family,
        commands::phase05::set_product_family_active,
        commands::phase05::list_partner_addresses,
        commands::phase05::list_partner_contacts,
        commands::phase06::list_stock_balances,
        commands::phase06::list_stock_movements,
        commands::phase06::create_opening_stock,
        commands::phase06::review_opening_stock,
        commands::phase06::post_opening_stock,
        commands::phase06::post_stock_adjustment,
        commands::phase06::post_stock_transfer,
        commands::phase06::create_inventory_count,
        commands::phase06::update_inventory_count,
        commands::phase06::review_inventory_count,
        commands::phase06::post_inventory_count,
        commands::phase06::get_inventory_count,
        commands::phase06::create_stock_reservation,
        commands::phase06::release_stock_reservation,
        commands::phase06::consume_stock_reservation,
        commands::phase06::cancel_stock_reservation,
        commands::phase06::list_active_stock_reservations,
        commands::phase06::reconcile_stock_balances,
        commands::phase06::rebuild_stock_balances,
        commands::phase06::create_purchase_order,
        commands::phase06::update_purchase_order,
        commands::phase06::confirm_purchase_order,
        commands::phase06::cancel_purchase_order,
        commands::phase06::hold_purchase_order,
        commands::phase06::create_purchase_receipt,
        commands::phase06::post_purchase_receipt,
        commands::phase06::create_purchase_invoice,
        commands::phase06::post_purchase_invoice,
        commands::phase06::direct_receive_and_invoice,
        commands::phase06::post_purchase_return,
        commands::phase06::list_purchasing_documents,
        commands::phase06::get_purchasing_document,
        commands::phase07::create_sales_order,
        commands::phase07::update_sales_order,
        commands::phase07::confirm_sales_order,
        commands::phase07::hold_sales_order,
        commands::phase07::resume_sales_order,
        commands::phase07::cancel_sales_order,
        commands::phase07::deliver_sales_order,
        commands::phase07::invoice_sales_delivery,
        commands::phase07::direct_sale,
        commands::phase07::post_sales_return,
        commands::phase07::list_sales_documents,
        commands::phase07::get_sales_document,
        commands::phase07::get_sales_line_availability,
        commands::phase07::get_sales_summary,
        commands::phase08::install_accounting_template,
        commands::phase08::list_accounts,
        commands::phase08::create_account,
        commands::phase08::update_account,
        commands::phase08::list_accounting_journals,
        commands::phase08::create_accounting_journal,
        commands::phase08::update_accounting_journal,
        commands::phase08::list_posting_rules,
        commands::phase08::save_posting_rule,
        commands::phase08::validate_posting_configuration,
        commands::phase08::list_accounting_posting_queue,
        commands::phase08::post_source_event,
        commands::phase08::retry_posting_attempt,
        commands::phase08::list_journal_entries,
        commands::phase08::get_journal_entry,
        commands::phase08::create_manual_journal_entry,
        commands::phase08::update_manual_journal_entry,
        commands::phase08::post_manual_journal_entry,
        commands::phase08::reverse_journal_entry,
        commands::phase08::post_customer_receipt,
        commands::phase08::post_supplier_payment,
        commands::phase08::allocate_payment,
        commands::phase08::reverse_payment_allocation,
        commands::phase08::reverse_payment,
        commands::phase08::list_payments,
        commands::phase08::get_partner_statement,
        commands::phase08::get_cash_bank_register,
        commands::phase08::get_trial_balance,
        commands::phase08::get_general_ledger,
        commands::phase08::get_account_ledger,
        commands::phase08::get_open_receivables,
        commands::phase08::get_open_payables,
        commands::phase08::list_fiscal_periods,
        commands::phase08::close_fiscal_period,
        commands::phase08::reopen_fiscal_period,
        commands::phase09::phase09_list_templates,
        commands::phase09::phase09_get_template,
        commands::phase09::phase09_create_template_draft,
        commands::phase09::phase09_update_template_draft,
        commands::phase09::phase09_publish_template,
        commands::phase09::phase09_retire_template,
        commands::phase09::phase09_preview_document,
        commands::phase09::phase09_get_preview_content,
        commands::phase09::phase09_render_document,
        commands::phase09::phase09_list_rendered_documents,
        commands::phase09::phase09_get_rendered_document,
        commands::phase09::phase09_verify_rendered_document,
        commands::phase09::phase09_export_rendered_pdf,
        commands::phase09::phase09_print_rendered_document,
        commands::phase09::phase09_list_reports,
        commands::phase09::phase09_run_report,
        commands::phase09::phase09_export_report_csv,
        commands::phase09::phase09_export_report_pdf,
        commands::phase09::phase09_list_audit_events,
        commands::phase09::phase09_export_audit_csv,
        commands::phase09::phase09_get_backup_settings,
        commands::phase09::phase09_update_backup_settings,
        commands::phase09::phase09_create_backup,
        commands::phase09::phase09_list_backups,
        commands::phase09::phase09_verify_backup,
        commands::phase09::phase09_export_backup,
        commands::phase09::phase09_import_backup,
        commands::phase09::phase09_restore_backup,
        commands::phase09::phase09_delete_backup
    ])
}

#[cfg(test)]
fn configure_test_application(
    builder: tauri::Builder<tauri::test::MockRuntime>,
    runtime_root: RuntimeRoot,
) -> tauri::Builder<tauri::test::MockRuntime> {
    configure_runtime(builder, runtime_root).invoke_handler(tauri::generate_handler![
        commands::runtime::get_runtime_status,
        commands::phase06::list_stock_balances,
        commands::phase07::get_sales_summary,
        commands::phase08::list_accounts,
        commands::phase09::phase09_list_templates
    ])
}

fn boxed_runtime_error(error: RuntimeError) -> Box<dyn Error> {
    Box::new(error)
}

fn application_builder() -> tauri::Builder<tauri::Wry> {
    configure_application(tauri::Builder::default(), RuntimeRoot::SystemLocalData)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    application_builder()
        .run(tauri::generate_context!())
        .expect("failed to initialize or run the POSMAN local desktop runtime");
}

#[cfg(test)]
mod ipc_tests;
