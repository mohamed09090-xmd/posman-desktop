use crate::{
    infrastructure::native_dialog,
    phase09::{
        error::{Phase09Error, Phase09Result},
        models::*,
        Phase09Service,
    },
};
use tauri::State;

async fn run<T: Send + 'static>(
    service: Phase09Service,
    operation: impl FnOnce(Phase09Service) -> Phase09Result<T> + Send + 'static,
) -> Phase09Result<T> {
    tauri::async_runtime::spawn_blocking(move || operation(service))
        .await
        .map_err(|_| Phase09Error::internal())?
}

macro_rules! command {
    ($name:ident,$request:ty,$result:ty,$method:ident) => {
        #[tauri::command]
        pub async fn $name(
            state: State<'_, Phase09Service>,
            request: $request,
        ) -> Phase09Result<$result> {
            run(state.inner().clone(), move |service| {
                service.$method(request)
            })
            .await
        }
    };
}

macro_rules! query {
    ($name:ident,$result:ty,$method:ident) => {
        #[tauri::command]
        pub async fn $name(state: State<'_, Phase09Service>) -> Phase09Result<$result> {
            run(state.inner().clone(), move |service| service.$method(())).await
        }
    };
}

query!(phase09_list_templates, Vec<TemplateSummary>, list_templates);
command!(
    phase09_get_template,
    TemplateKeyRequest,
    TemplateDetail,
    get_template
);
command!(
    phase09_create_template_draft,
    CreateTemplateDraftRequest,
    TemplateDraftView,
    create_template_draft
);
command!(
    phase09_update_template_draft,
    UpdateTemplateDraftRequest,
    TemplateDraftView,
    update_template_draft
);
command!(
    phase09_publish_template,
    PublishTemplateRequest,
    TemplateVersionView,
    publish_template
);
command!(
    phase09_retire_template,
    RetireTemplateRequest,
    TemplateVersionView,
    retire_template
);

#[tauri::command]
pub async fn phase09_preview_document(
    app: tauri::AppHandle,
    state: State<'_, Phase09Service>,
    request: DocumentRequest,
) -> Phase09Result<PreviewResult> {
    let app_handle = app.clone();
    run(state.inner().clone(), move |service| {
        service.preview_document(&app_handle, request)
    })
    .await
}

command!(
    phase09_get_preview_content,
    String,
    PreviewContent,
    get_preview_content
);

#[tauri::command]
pub async fn phase09_render_document(
    app: tauri::AppHandle,
    state: State<'_, Phase09Service>,
    request: DocumentRequest,
) -> Phase09Result<RenderedDocumentView> {
    let app_handle = app.clone();
    run(state.inner().clone(), move |service| {
        service.render_document(&app_handle, request)
    })
    .await
}

command!(
    phase09_list_rendered_documents,
    RenderedDocumentsRequest,
    Paged<RenderedDocumentView>,
    list_rendered_documents
);
command!(
    phase09_get_rendered_document,
    RenderedDocumentKeyRequest,
    RenderedDocumentView,
    get_rendered_document
);
command!(
    phase09_verify_rendered_document,
    RenderedDocumentKeyRequest,
    RenderedDocumentView,
    verify_rendered_document
);

#[tauri::command]
pub async fn phase09_export_rendered_pdf(
    state: State<'_, Phase09Service>,
    request: RenderedDocumentKeyRequest,
) -> Phase09Result<ExportResult> {
    let service = state.inner().clone();
    run(service, move |service| {
        let destination = save_path("pdf", "document.pdf")?;
        service.export_rendered_pdf_to(request, &destination)
    })
    .await
}

#[tauri::command]
pub async fn phase09_print_rendered_document(
    app: tauri::AppHandle,
    state: State<'_, Phase09Service>,
    request: RenderedDocumentKeyRequest,
) -> Phase09Result<()> {
    let app_handle = app.clone();
    run(state.inner().clone(), move |service| {
        service.print_rendered_document(&app_handle, request)
    })
    .await
}

query!(phase09_list_reports, Vec<ReportDescriptor>, list_reports);
command!(phase09_run_report, ReportRequest, ReportPage, run_report);

#[tauri::command]
pub async fn phase09_export_report_csv(
    state: State<'_, Phase09Service>,
    request: ReportRequest,
) -> Phase09Result<ExportResult> {
    run(state.inner().clone(), move |service| {
        let destination = save_path("csv", "report.csv")?;
        let result = service.export_report_csv(request)?;
        service.deliver_managed_export(&result, &destination)
    })
    .await
}

#[tauri::command]
pub async fn phase09_export_report_pdf(
    app: tauri::AppHandle,
    state: State<'_, Phase09Service>,
    request: ReportRequest,
) -> Phase09Result<ExportResult> {
    let output_app = app.clone();
    run(state.inner().clone(), move |service| {
        let destination = save_path("pdf", "report.pdf")?;
        let result = service.export_report_pdf(&output_app, request)?;
        service.deliver_managed_export(&result, &destination)
    })
    .await
}

command!(
    phase09_list_audit_events,
    AuditRequest,
    Paged<AuditEventView>,
    list_audit_events
);

#[tauri::command]
pub async fn phase09_export_audit_csv(
    state: State<'_, Phase09Service>,
    request: AuditRequest,
) -> Phase09Result<ExportResult> {
    run(state.inner().clone(), move |service| {
        let destination = save_path("csv", "audit.csv")?;
        let result = service.export_audit_csv(request)?;
        service.deliver_managed_export(&result, &destination)
    })
    .await
}

query!(
    phase09_get_backup_settings,
    BackupSettingsView,
    get_backup_settings
);
command!(
    phase09_update_backup_settings,
    UpdateBackupSettingsRequest,
    BackupSettingsView,
    update_backup_settings
);
command!(
    phase09_create_backup,
    CreateBackupRequest,
    BackupView,
    create_backup
);
command!(
    phase09_list_backups,
    BackupListRequest,
    Paged<BackupView>,
    list_backups
);
command!(
    phase09_verify_backup,
    BackupKeyRequest,
    BackupView,
    verify_backup
);

#[tauri::command]
pub async fn phase09_export_backup(
    state: State<'_, Phase09Service>,
    request: BackupKeyRequest,
) -> Phase09Result<ExportResult> {
    run(state.inner().clone(), move |service| {
        let destination = save_path("sqlite3", "posman-backup.sqlite3")?;
        service.export_backup_to(request, &destination)
    })
    .await
}

#[tauri::command]
pub async fn phase09_import_backup(state: State<'_, Phase09Service>) -> Phase09Result<BackupView> {
    run(state.inner().clone(), move |service| {
        let source = native_dialog::open_file()?;
        service.import_backup_from(&source)
    })
    .await
}

command!(
    phase09_restore_backup,
    RestoreBackupRequest,
    (),
    restore_backup
);
command!(phase09_delete_backup, BackupKeyRequest, (), delete_backup);

fn save_path(extension: &str, file_name: &str) -> Phase09Result<std::path::PathBuf> {
    native_dialog::save_file(file_name, extension)
}
