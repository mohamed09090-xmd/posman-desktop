use tauri::State;

use crate::phase05::{
    dto::*,
    error::{Phase05Error, Phase05Result},
    Phase05Service,
};
use crate::phase09::Phase09Service;

async fn run_blocking<T: Send + 'static>(
    service: Phase05Service,
    operation: impl FnOnce(Phase05Service) -> Phase05Result<T> + Send + 'static,
) -> Phase05Result<T> {
    tauri::async_runtime::spawn_blocking(move || operation(service))
        .await
        .map_err(|_| Phase05Error::internal())?
}

#[tauri::command]
pub async fn get_setup_status(state: State<'_, Phase05Service>) -> Phase05Result<SetupStatus> {
    run_blocking(state.inner().clone(), |service| service.get_setup_status()).await
}

#[tauri::command]
pub async fn load_setup_draft(
    state: State<'_, Phase05Service>,
) -> Phase05Result<Option<SetupDraft>> {
    run_blocking(state.inner().clone(), |service| service.load_setup_draft()).await
}

#[tauri::command]
pub async fn discard_setup_draft(state: State<'_, Phase05Service>) -> Phase05Result<()> {
    run_blocking(state.inner().clone(), |service| {
        service.discard_setup_draft()
    })
    .await
}

#[tauri::command]
pub async fn get_current_session(state: State<'_, Phase05Service>) -> Phase05Result<SessionView> {
    run_blocking(state.inner().clone(), |service| {
        service.get_current_session()
    })
    .await
}

#[tauri::command]
pub async fn logout(state: State<'_, Phase05Service>) -> Phase05Result<()> {
    run_blocking(state.inner().clone(), |service| service.logout()).await
}

#[tauri::command]
pub async fn lock_session(state: State<'_, Phase05Service>) -> Phase05Result<()> {
    run_blocking(state.inner().clone(), |service| service.lock_session()).await
}

#[tauri::command]
pub async fn rotate_recovery_code(
    state: State<'_, Phase05Service>,
) -> Phase05Result<RecoveryCodeResult> {
    run_blocking(state.inner().clone(), |service| {
        service.rotate_recovery_code()
    })
    .await
}

#[tauri::command]
pub async fn get_company_profile(
    state: State<'_, Phase05Service>,
) -> Phase05Result<CompanyProfile> {
    run_blocking(state.inner().clone(), |service| {
        service.get_company_profile()
    })
    .await
}

#[tauri::command]
pub async fn get_fiscal_setup(state: State<'_, Phase05Service>) -> Phase05Result<FiscalSetup> {
    run_blocking(state.inner().clone(), |service| service.get_fiscal_setup()).await
}

#[tauri::command]
pub async fn list_document_sequences(
    state: State<'_, Phase05Service>,
) -> Phase05Result<Vec<DocumentSequenceView>> {
    run_blocking(state.inner().clone(), |service| {
        service.list_document_sequences()
    })
    .await
}

#[tauri::command]
pub async fn list_roles(state: State<'_, Phase05Service>) -> Phase05Result<Vec<RoleView>> {
    run_blocking(state.inner().clone(), |service| service.list_roles()).await
}

#[tauri::command]
pub async fn save_setup_draft(
    state: State<'_, Phase05Service>,
    request: SaveSetupDraftRequest,
) -> Phase05Result<SetupDraft> {
    run_blocking(state.inner().clone(), move |service| {
        service.save_setup_draft(request)
    })
    .await
}

#[tauri::command]
pub async fn complete_initial_setup(
    state: State<'_, Phase05Service>,
    request: InitialSetupRequest,
) -> Phase05Result<CompleteSetupResult> {
    run_blocking(state.inner().clone(), move |service| {
        service.complete_initial_setup(request)
    })
    .await
}

#[tauri::command]
pub async fn login(
    state: State<'_, Phase05Service>,
    phase09_state: State<'_, Phase09Service>,
    request: LoginRequest,
) -> Phase05Result<SessionView> {
    let session =
        run_blocking(state.inner().clone(), move |service| service.login(request)).await?;
    let phase09 = phase09_state.inner().clone();
    drop(tauri::async_runtime::spawn_blocking(move || {
        let _ = phase09.attempt_automatic_backup_after_login();
    }));
    Ok(session)
}

#[tauri::command]
pub async fn recover_admin_password(
    state: State<'_, Phase05Service>,
    request: RecoverPasswordRequest,
) -> Phase05Result<RecoveryCodeResult> {
    run_blocking(state.inner().clone(), move |service| {
        service.recover_admin_password(request)
    })
    .await
}

#[tauri::command]
pub async fn unlock_session(
    state: State<'_, Phase05Service>,
    request: UnlockSessionRequest,
) -> Phase05Result<SessionView> {
    run_blocking(state.inner().clone(), move |service| {
        service.unlock_session(request)
    })
    .await
}

#[tauri::command]
pub async fn change_own_password(
    state: State<'_, Phase05Service>,
    request: ChangePasswordRequest,
) -> Phase05Result<()> {
    run_blocking(state.inner().clone(), move |service| {
        service.change_own_password(request)
    })
    .await
}

#[tauri::command]
pub async fn update_company_profile(
    state: State<'_, Phase05Service>,
    request: UpdateCompanyProfileRequest,
) -> Phase05Result<CompanyProfile> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_company_profile(request)
    })
    .await
}

#[tauri::command]
pub async fn update_fiscal_setup(
    state: State<'_, Phase05Service>,
    request: UpdateFiscalSetupRequest,
) -> Phase05Result<FiscalSetup> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_fiscal_setup(request)
    })
    .await
}

#[tauri::command]
pub async fn update_document_sequence(
    state: State<'_, Phase05Service>,
    request: UpdateDocumentSequenceRequest,
) -> Phase05Result<DocumentSequenceView> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_document_sequence(request)
    })
    .await
}

#[tauri::command]
pub async fn list_users(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<UserView>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_users(request)
    })
    .await
}

#[tauri::command]
pub async fn create_user(
    state: State<'_, Phase05Service>,
    request: CreateUserRequest,
) -> Phase05Result<UserView> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_user(request)
    })
    .await
}

#[tauri::command]
pub async fn update_user(
    state: State<'_, Phase05Service>,
    request: UpdateUserRequest,
) -> Phase05Result<UserView> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_user(request)
    })
    .await
}

#[tauri::command]
pub async fn set_user_roles(
    state: State<'_, Phase05Service>,
    request: SetUserRolesRequest,
) -> Phase05Result<UserView> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_user_roles(request)
    })
    .await
}

#[tauri::command]
pub async fn reset_user_password(
    state: State<'_, Phase05Service>,
    request: ResetUserPasswordRequest,
) -> Phase05Result<()> {
    run_blocking(state.inner().clone(), move |service| {
        service.reset_user_password(request)
    })
    .await
}

#[tauri::command]
pub async fn create_role(
    state: State<'_, Phase05Service>,
    request: CreateRoleRequest,
) -> Phase05Result<RoleView> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_role(request)
    })
    .await
}

#[tauri::command]
pub async fn update_role(
    state: State<'_, Phase05Service>,
    request: UpdateRoleRequest,
) -> Phase05Result<RoleView> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_role(request)
    })
    .await
}

#[tauri::command]
pub async fn set_role_permissions(
    state: State<'_, Phase05Service>,
    request: SetRolePermissionsRequest,
) -> Phase05Result<RoleView> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_role_permissions(request)
    })
    .await
}

#[tauri::command]
pub async fn list_products(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ProductView>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_products(request)
    })
    .await
}

#[tauri::command]
pub async fn create_product(
    state: State<'_, Phase05Service>,
    request: CreateProductRequest,
) -> Phase05Result<ProductView> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_product(request)
    })
    .await
}

#[tauri::command]
pub async fn update_product(
    state: State<'_, Phase05Service>,
    request: UpdateProductRequest,
) -> Phase05Result<ProductView> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_product(request)
    })
    .await
}

#[tauri::command]
pub async fn set_product_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ProductView> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_product_active(request)
    })
    .await
}

#[tauri::command]
pub async fn set_product_price(
    state: State<'_, Phase05Service>,
    request: ProductPriceInput,
) -> Phase05Result<()> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_product_price(request)
    })
    .await
}

#[tauri::command]
pub async fn list_partners(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<PartnerView>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_partners(request)
    })
    .await
}

#[tauri::command]
pub async fn create_partner(
    state: State<'_, Phase05Service>,
    request: CreatePartnerRequest,
) -> Phase05Result<PartnerView> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_partner(request)
    })
    .await
}

#[tauri::command]
pub async fn update_partner(
    state: State<'_, Phase05Service>,
    request: UpdatePartnerRequest,
) -> Phase05Result<PartnerView> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_partner(request)
    })
    .await
}

#[tauri::command]
pub async fn set_partner_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<PartnerView> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_partner_active(request)
    })
    .await
}

#[tauri::command]
pub async fn create_partner_address(
    state: State<'_, Phase05Service>,
    request: PartnerAddressInput,
) -> Phase05Result<PartnerAddressView> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_partner_address(request)
    })
    .await
}

#[tauri::command]
pub async fn create_partner_contact(
    state: State<'_, Phase05Service>,
    request: PartnerContactInput,
) -> Phase05Result<PartnerContactView> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_partner_contact(request)
    })
    .await
}

#[tauri::command]
pub async fn list_units(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ReferenceRecord>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_units(request)
    })
    .await
}

#[tauri::command]
pub async fn create_unit(
    state: State<'_, Phase05Service>,
    request: ReferenceInput,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_unit(request)
    })
    .await
}

#[tauri::command]
pub async fn update_unit(
    state: State<'_, Phase05Service>,
    request: ReferenceUpdate,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_unit(request)
    })
    .await
}

#[tauri::command]
pub async fn set_unit_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_unit_active(request)
    })
    .await
}

#[tauri::command]
pub async fn list_tax_rates(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ReferenceRecord>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_tax_rates(request)
    })
    .await
}

#[tauri::command]
pub async fn create_tax_rate(
    state: State<'_, Phase05Service>,
    request: ReferenceInput,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_tax_rate(request)
    })
    .await
}

#[tauri::command]
pub async fn update_tax_rate(
    state: State<'_, Phase05Service>,
    request: ReferenceUpdate,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_tax_rate(request)
    })
    .await
}

#[tauri::command]
pub async fn set_tax_rate_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_tax_rate_active(request)
    })
    .await
}

#[tauri::command]
pub async fn list_payment_terms(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ReferenceRecord>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_payment_terms(request)
    })
    .await
}

#[tauri::command]
pub async fn create_payment_term(
    state: State<'_, Phase05Service>,
    request: ReferenceInput,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_payment_term(request)
    })
    .await
}

#[tauri::command]
pub async fn update_payment_term(
    state: State<'_, Phase05Service>,
    request: ReferenceUpdate,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_payment_term(request)
    })
    .await
}

#[tauri::command]
pub async fn set_payment_term_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_payment_term_active(request)
    })
    .await
}

#[tauri::command]
pub async fn list_payment_methods(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ReferenceRecord>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_payment_methods(request)
    })
    .await
}

#[tauri::command]
pub async fn create_payment_method(
    state: State<'_, Phase05Service>,
    request: ReferenceInput,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_payment_method(request)
    })
    .await
}

#[tauri::command]
pub async fn update_payment_method(
    state: State<'_, Phase05Service>,
    request: ReferenceUpdate,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_payment_method(request)
    })
    .await
}

#[tauri::command]
pub async fn set_payment_method_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_payment_method_active(request)
    })
    .await
}

#[tauri::command]
pub async fn list_warehouses(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ReferenceRecord>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_warehouses(request)
    })
    .await
}

#[tauri::command]
pub async fn create_warehouse(
    state: State<'_, Phase05Service>,
    request: ReferenceInput,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_warehouse(request)
    })
    .await
}

#[tauri::command]
pub async fn update_warehouse(
    state: State<'_, Phase05Service>,
    request: ReferenceUpdate,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_warehouse(request)
    })
    .await
}

#[tauri::command]
pub async fn set_warehouse_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_warehouse_active(request)
    })
    .await
}

#[tauri::command]
pub async fn list_warehouse_locations(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ReferenceRecord>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_warehouse_locations(request)
    })
    .await
}

#[tauri::command]
pub async fn create_warehouse_location(
    state: State<'_, Phase05Service>,
    request: ReferenceInput,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_warehouse_location(request)
    })
    .await
}

#[tauri::command]
pub async fn update_warehouse_location(
    state: State<'_, Phase05Service>,
    request: ReferenceUpdate,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_warehouse_location(request)
    })
    .await
}

#[tauri::command]
pub async fn set_warehouse_location_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_warehouse_location_active(request)
    })
    .await
}

#[tauri::command]
pub async fn list_product_families(
    state: State<'_, Phase05Service>,
    request: PageRequest,
) -> Phase05Result<Page<ReferenceRecord>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_product_families(request)
    })
    .await
}

#[tauri::command]
pub async fn create_product_family(
    state: State<'_, Phase05Service>,
    request: ReferenceInput,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.create_product_family(request)
    })
    .await
}

#[tauri::command]
pub async fn update_product_family(
    state: State<'_, Phase05Service>,
    request: ReferenceUpdate,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.update_product_family(request)
    })
    .await
}

#[tauri::command]
pub async fn set_product_family_active(
    state: State<'_, Phase05Service>,
    request: SetActiveRequest,
) -> Phase05Result<ReferenceRecord> {
    run_blocking(state.inner().clone(), move |service| {
        service.set_product_family_active(request)
    })
    .await
}

#[tauri::command]
pub async fn list_partner_addresses(
    state: State<'_, Phase05Service>,
    partner_id: String,
) -> Phase05Result<Vec<PartnerAddressView>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_partner_addresses(partner_id)
    })
    .await
}

#[tauri::command]
pub async fn list_partner_contacts(
    state: State<'_, Phase05Service>,
    partner_id: String,
) -> Phase05Result<Vec<PartnerContactView>> {
    run_blocking(state.inner().clone(), move |service| {
        service.list_partner_contacts(partner_id)
    })
    .await
}
