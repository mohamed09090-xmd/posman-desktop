use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use rusqlite::{params, TransactionBehavior};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Wry};

use super::{
    documents::{PreparedDocument, RenderedRecord},
    error::{Phase09Error, Phase09Result},
    models::{DocumentRequest, ExportResult, RenderedDocumentKeyRequest, RenderedDocumentView},
    new_id, now_iso, safe_component, Phase09Service,
};

pub struct PdfOutputRequest<'a> {
    pub html: &'a str,
    pub css: &'a str,
    pub destination: &'a Path,
}

pub struct PrintOutputRequest<'a> {
    pub pdf_path: &'a Path,
}

pub struct PdfArtifact {
    pub sha256: String,
    pub size_bytes: i64,
}

pub trait DocumentOutputEngine {
    fn generate_pdf(
        &self,
        app: &AppHandle<Wry>,
        request: PdfOutputRequest<'_>,
    ) -> Phase09Result<PdfArtifact>;

    fn show_print_ui(
        &self,
        app: &AppHandle<Wry>,
        request: PrintOutputRequest<'_>,
    ) -> Phase09Result<()>;
}

struct PlatformOutputEngine;

impl DocumentOutputEngine for PlatformOutputEngine {
    fn generate_pdf(
        &self,
        app: &AppHandle<Wry>,
        request: PdfOutputRequest<'_>,
    ) -> Phase09Result<PdfArtifact> {
        native::generate_pdf(app, request.html, request.css, request.destination)?;
        verify_pdf(request.destination)
    }

    fn show_print_ui(
        &self,
        app: &AppHandle<Wry>,
        request: PrintOutputRequest<'_>,
    ) -> Phase09Result<()> {
        native::show_print_ui(app, request.pdf_path)
    }
}

impl Phase09Service {
    pub fn render_document(
        &self,
        app: &AppHandle<Wry>,
        request: DocumentRequest,
    ) -> Phase09Result<RenderedDocumentView> {
        let _output_guard = self
            .output_lock
            .try_lock()
            .map_err(|_| Phase09Error::output_busy())?;
        let prepared = self.prepare_document_snapshot(&request)?;
        let render_id = new_id();
        let rendered_at = now_iso()?;
        let relative_path = document_relative_path(
            &prepared.company_id,
            &request.document_type,
            &rendered_at,
            &render_id,
        )?;
        let final_path = managed_path(&self.paths.documents, &relative_path)?;
        let parent = final_path
            .parent()
            .ok_or_else(|| Phase09Error::validation("Invalid document output path."))?;
        fs::create_dir_all(parent)?;
        if final_path.exists() {
            return Err(Phase09Error::integrity(
                "The historical output path already exists and was not overwritten.",
            ));
        }
        fs::create_dir_all(&self.paths.staging)?;
        let temporary_path = self
            .paths
            .staging
            .join(format!("phase09-output-{render_id}.pdf.tmp"));
        remove_if_exists(&temporary_path);

        let artifact = match PlatformOutputEngine.generate_pdf(
            app,
            PdfOutputRequest {
                html: &prepared.html,
                css: &prepared.css,
                destination: &temporary_path,
            },
        ) {
            Ok(artifact) => artifact,
            Err(error) => {
                remove_if_exists(&temporary_path);
                return Err(error);
            }
        };
        if let Err(error) = fs::rename(&temporary_path, &final_path) {
            remove_if_exists(&temporary_path);
            return Err(Phase09Error::from(error));
        }

        let insert = self.insert_render_record(
            &request,
            &prepared,
            &render_id,
            &relative_path,
            &artifact,
            &rendered_at,
        );
        if let Err(error) = insert {
            remove_if_exists(&final_path);
            return Err(error);
        }
        Ok(RenderedDocumentView {
            render_id,
            document_type: request.document_type,
            source_document_id: request.source_document_id,
            source_document_number: prepared.payload.document_number,
            source_document_status: prepared.payload.document_status,
            template_id: prepared.template_id,
            template_version_id: prepared.template_version_id,
            locale: prepared.locale,
            content_sha256: prepared.content_sha256,
            pdf_relative_path: relative_path,
            pdf_sha256: artifact.sha256,
            pdf_size_bytes: artifact.size_bytes,
            rendered_at,
            rendered_by: prepared.user_id,
            integrity_state: "VERIFIED".to_owned(),
        })
    }

    pub fn verify_rendered_document(
        &self,
        request: RenderedDocumentKeyRequest,
    ) -> Phase09Result<RenderedDocumentView> {
        let context = self.authorize("documents.export")?;
        let record = self.load_rendered_record(&context.company_id, &request.render_id)?;
        match self.verify_record(&record) {
            Ok(()) => Ok(record.view("VERIFIED")),
            Err(error) => {
                let _ = self.audit_failure(
                    &context,
                    "DOCUMENT_ARTIFACT_INTEGRITY_FAILED",
                    "PHASE09_RENDERED_DOCUMENT",
                    &record.render_id,
                    &error.code,
                );
                Err(error)
            }
        }
    }

    pub fn export_rendered_pdf_to(
        &self,
        request: RenderedDocumentKeyRequest,
        destination: &Path,
    ) -> Phase09Result<ExportResult> {
        let context = self.authorize("documents.export")?;
        let record = self.load_rendered_record(&context.company_id, &request.render_id)?;
        self.verify_record(&record)?;
        let source = managed_path(&self.paths.documents, &record.pdf_relative_path)?;
        copy_verified(
            &source,
            destination,
            &record.pdf_sha256,
            record.pdf_size_bytes,
        )
    }

    pub fn print_rendered_document(
        &self,
        app: &AppHandle<Wry>,
        request: RenderedDocumentKeyRequest,
    ) -> Phase09Result<()> {
        let _output_guard = self
            .output_lock
            .try_lock()
            .map_err(|_| Phase09Error::output_busy())?;
        let context = self.authorize("documents.print")?;
        let record = self.load_rendered_record(&context.company_id, &request.render_id)?;
        self.verify_record(&record)?;
        let source = managed_path(&self.paths.documents, &record.pdf_relative_path)?;
        PlatformOutputEngine.show_print_ui(app, PrintOutputRequest { pdf_path: &source })?;
        let connection = self.phase05.phase09_open_maintenance()?;
        connection.execute(
            "INSERT INTO audit_logs(id,company_id,actor_user_id,action_code,entity_type,entity_id,occurred_at,outcome,correlation_id,details_json) VALUES(?1,?2,?3,'DOCUMENT_PRINTED','PHASE09_RENDERED_DOCUMENT',?4,?5,'SUCCESS',?6,json_object('pdfSha256',?7))",
            params![new_id(), context.company_id, context.user_id, record.render_id, now_iso()?, context.session_id, record.pdf_sha256],
        )?;
        Ok(())
    }

    fn insert_render_record(
        &self,
        request: &DocumentRequest,
        prepared: &PreparedDocument,
        render_id: &str,
        relative_path: &str,
        artifact: &PdfArtifact,
        rendered_at: &str,
    ) -> Phase09Result<()> {
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            r#"INSERT INTO phase09_rendered_documents(
                   id,company_id,document_type,source_document_id,source_document_number,
                   source_document_status,document_template_id,template_version_id,locale,
                   canonical_payload_json,rendered_html,rendered_css,content_sha256,
                   pdf_relative_path,pdf_sha256,pdf_size_bytes,rendered_at,rendered_by
               ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)"#,
            params![
                render_id,
                prepared.company_id,
                request.document_type,
                request.source_document_id,
                prepared.payload.document_number,
                prepared.payload.document_status,
                prepared.template_id,
                prepared.template_version_id,
                prepared.locale,
                prepared.canonical_payload_json,
                prepared.html,
                prepared.css,
                prepared.content_sha256,
                relative_path,
                artifact.sha256,
                artifact.size_bytes,
                rendered_at,
                prepared.user_id,
            ],
        )?;
        let context = self.authorize("documents.render")?;
        if context.company_id != prepared.company_id || context.user_id != prepared.user_id {
            return Err(Phase09Error::permission());
        }
        Self::audit_success(
            &transaction,
            &context,
            "DOCUMENT_RENDERED",
            "PHASE09_RENDERED_DOCUMENT",
            render_id,
            Some(&serde_json::json!({
                "templateVersionId": prepared.template_version_id,
                "contentSha256": prepared.content_sha256,
                "pdfSha256": artifact.sha256,
                "pdfSizeBytes": artifact.size_bytes,
            })),
        )?;
        transaction.commit()?;
        Ok(())
    }

    fn verify_record(&self, record: &RenderedRecord) -> Phase09Result<()> {
        let path = managed_path(&self.paths.documents, &record.pdf_relative_path)?;
        let artifact = verify_pdf(&path).map_err(|_| {
            Phase09Error::integrity(
                "The original historical PDF is missing or does not contain a valid PDF header.",
            )
        })?;
        if artifact.sha256 != record.pdf_sha256 || artifact.size_bytes != record.pdf_size_bytes {
            return Err(Phase09Error::integrity(
                "The original historical PDF no longer matches its recorded digest.",
            ));
        }
        Ok(())
    }
}

impl RenderedRecord {
    fn view(&self, integrity_state: &str) -> RenderedDocumentView {
        RenderedDocumentView {
            render_id: self.render_id.clone(),
            document_type: self.document_type.clone(),
            source_document_id: self.source_document_id.clone(),
            source_document_number: self.source_document_number.clone(),
            source_document_status: self.source_document_status.clone(),
            template_id: self.template_id.clone(),
            template_version_id: self.template_version_id.clone(),
            locale: self.locale.clone(),
            content_sha256: self.content_sha256.clone(),
            pdf_relative_path: self.pdf_relative_path.clone(),
            pdf_sha256: self.pdf_sha256.clone(),
            pdf_size_bytes: self.pdf_size_bytes,
            rendered_at: self.rendered_at.clone(),
            rendered_by: self.rendered_by.clone(),
            integrity_state: integrity_state.to_owned(),
        }
    }
}

fn document_relative_path(
    company_id: &str,
    document_type: &str,
    rendered_at: &str,
    render_id: &str,
) -> Phase09Result<String> {
    let date = rendered_at
        .get(0..10)
        .ok_or_else(|| Phase09Error::validation("Invalid render timestamp."))?;
    let year = date
        .get(0..4)
        .ok_or_else(|| Phase09Error::validation("Invalid render year."))?;
    let month = date
        .get(5..7)
        .ok_or_else(|| Phase09Error::validation("Invalid render month."))?;
    Ok(format!(
        "{}/{}/{}/{}/{}.pdf",
        safe_component(company_id)?,
        safe_component(&document_type.to_ascii_lowercase())?,
        safe_component(year)?,
        safe_component(month)?,
        safe_component(render_id)?,
    ))
}

fn managed_path(root: &Path, relative: &str) -> Phase09Result<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative.contains("..")
        || relative.contains('\\')
        || relative_path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(Phase09Error::validation("Unsafe managed document path."));
    }
    Ok(root.join(relative_path))
}

fn verify_pdf(path: &Path) -> Phase09Result<PdfArtifact> {
    let bytes = fs::read(path)?;
    if bytes.len() < 5 || !bytes.starts_with(b"%PDF-") {
        return Err(Phase09Error::integrity(
            "The output is not a valid PDF artifact.",
        ));
    }
    let size_bytes = i64::try_from(bytes.len()).map_err(|_| Phase09Error::internal())?;
    if size_bytes == 0 {
        return Err(Phase09Error::integrity("The output PDF is empty."));
    }
    Ok(PdfArtifact {
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        size_bytes,
    })
}

fn copy_verified(
    source: &Path,
    destination: &Path,
    expected_sha256: &str,
    expected_size: i64,
) -> Phase09Result<ExportResult> {
    let artifact = verify_pdf(source)?;
    if artifact.sha256 != expected_sha256 || artifact.size_bytes != expected_size {
        return Err(Phase09Error::integrity(
            "The original historical PDF failed verification and was not exported.",
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| Phase09Error::validation("Invalid export destination."))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".posman-export-{}.tmp", new_id()));
    let mut input = OpenOptions::new().read(true).open(source)?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    std::io::copy(&mut input, &mut output)?;
    output.flush()?;
    output.sync_all()?;
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(&temporary, destination)?;
    Ok(ExportResult {
        relative_path: destination
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("document.pdf")
            .to_owned(),
        sha256: artifact.sha256,
        size_bytes: artifact.size_bytes,
    })
}

fn remove_if_exists(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(not(windows))]
mod native {
    use super::*;

    pub fn generate_pdf(
        _app: &AppHandle<Wry>,
        _html: &str,
        _css: &str,
        _destination: &Path,
    ) -> Phase09Result<()> {
        Err(Phase09Error::platform_unsupported())
    }

    pub fn show_print_ui(_app: &AppHandle<Wry>, _pdf_path: &Path) -> Phase09Result<()> {
        Err(Phase09Error::platform_unsupported())
    }
}

#[cfg(windows)]
mod native {
    use std::{os::windows::ffi::OsStrExt, sync::mpsc, time::Duration};

    use tauri::{WebviewUrl, WebviewWindowBuilder};
    use webview2_com::{
        Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_16, ICoreWebView2_7, COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM,
            COREWEBVIEW2_PRINT_ORIENTATION_LANDSCAPE, COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT,
        },
        PrintToPdfCompletedHandler,
    };
    use windows::core::{Interface, PCWSTR};

    use super::*;

    pub fn generate_pdf(
        app: &AppHandle<Wry>,
        html: &str,
        css: &str,
        destination: &Path,
    ) -> Phase09Result<()> {
        let label = format!("phase09-output-{}", new_id());
        let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
            .title("POSMAN document output")
            .visible(false)
            .skip_taskbar(true)
            .decorations(false)
            .inner_size(900.0, 1200.0)
            .build()
            .map_err(|_| Phase09Error::internal())?;
        let full = html.replace("</head>", &format!("<style>{css}</style></head>"));
        let serialized = serde_json::to_string(&full).map_err(|_| Phase09Error::internal())?;
        window
            .eval(&format!(
                "document.open();document.write({serialized});document.close();"
            ))
            .map_err(|_| Phase09Error::internal())?;
        std::thread::sleep(Duration::from_millis(350));

        let output = destination.to_path_buf();
        let landscape = css.contains("size:A4 landscape");
        let (sender, receiver) = mpsc::channel::<Phase09Result<()>>();
        window
            .as_ref()
            .with_webview(move |platform| {
                let result = unsafe {
                    let core = platform
                        .controller()
                        .CoreWebView2()
                        .map_err(|_| Phase09Error::internal())?;
                    let core7: ICoreWebView2_7 = core
                        .cast()
                        .map_err(|_| Phase09Error::platform_unsupported())?;
                    let settings = platform
                        .environment()
                        .CreatePrintSettings()
                        .map_err(|_| Phase09Error::internal())?;
                    settings
                        .SetShouldPrintHeaderAndFooter(false.into())
                        .map_err(|_| Phase09Error::internal())?;
                    settings
                        .SetShouldPrintBackgrounds(true.into())
                        .map_err(|_| Phase09Error::internal())?;
                    settings
                        .SetOrientation(if landscape {
                            COREWEBVIEW2_PRINT_ORIENTATION_LANDSCAPE
                        } else {
                            COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT
                        })
                        .map_err(|_| Phase09Error::internal())?;
                    let wide = output
                        .as_os_str()
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect::<Vec<_>>();
                    let callback_sender = sender.clone();
                    let handler = PrintToPdfCompletedHandler::create(Box::new(
                        move |result: windows::core::Result<()>, successful: bool| {
                            let completed =
                                result.map_err(|_| Phase09Error::internal()).and_then(|_| {
                                    if successful {
                                        Ok(())
                                    } else {
                                        Err(Phase09Error::new(
                                            "PDF_OUTPUT_FAILED",
                                            "WebView2 could not create the PDF.",
                                            true,
                                        ))
                                    }
                                });
                            let _ = callback_sender.send(completed);
                            Ok(())
                        },
                    ));
                    core7
                        .PrintToPdf(PCWSTR(wide.as_ptr()), &settings, &handler)
                        .map_err(|_| Phase09Error::internal())?;
                    Ok(())
                };
                if let Err(error) = result {
                    let _ = sender.send(Err(error));
                }
            })
            .map_err(|_| Phase09Error::internal())?;
        let result = receiver
            .recv_timeout(Duration::from_secs(30))
            .map_err(|_| {
                Phase09Error::new(
                    "PDF_OUTPUT_TIMEOUT",
                    "WebView2 PDF generation timed out.",
                    true,
                )
            })?;
        let _ = window.close();
        result
    }

    pub fn show_print_ui(app: &AppHandle<Wry>, pdf_path: &Path) -> Phase09Result<()> {
        let url = tauri::Url::from_file_path(pdf_path)
            .map_err(|_| Phase09Error::validation("Invalid historical PDF path."))?;
        let label = format!("phase09-print-{}", new_id());
        let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
            .title("POSMAN print")
            .visible(false)
            .skip_taskbar(true)
            .build()
            .map_err(|_| Phase09Error::internal())?;
        window.navigate(url).map_err(|_| Phase09Error::internal())?;
        std::thread::sleep(Duration::from_millis(350));
        let (sender, receiver) = mpsc::channel::<Phase09Result<()>>();
        window
            .as_ref()
            .with_webview(move |platform| {
                let result = unsafe {
                    let core = platform
                        .controller()
                        .CoreWebView2()
                        .map_err(|_| Phase09Error::internal())?;
                    let core16: ICoreWebView2_16 = core
                        .cast()
                        .map_err(|_| Phase09Error::platform_unsupported())?;
                    core16
                        .ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM)
                        .map_err(|_| Phase09Error::internal())?;
                    Ok(())
                };
                let _ = sender.send(result);
            })
            .map_err(|_| Phase09Error::internal())?;
        receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| {
                Phase09Error::new(
                    "PRINT_UI_TIMEOUT",
                    "The Windows print UI did not open.",
                    true,
                )
            })??;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_output_path_rejects_traversal() {
        let value = document_relative_path(
            "company-1",
            "SALES_INVOICE",
            "2026-08-07T01:02:03Z",
            "render-1",
        )
        .expect("path");
        assert_eq!(value, "company-1/sales_invoice/2026/08/render-1.pdf");
        assert!(document_relative_path("../x", "SALES_INVOICE", "2026-08-07", "r").is_err());
    }

    #[test]
    fn non_pdf_content_fails_integrity() {
        let path = std::env::temp_dir().join(format!("phase09-not-pdf-{}", new_id()));
        fs::write(&path, b"not pdf").expect("write");
        assert!(verify_pdf(&path).is_err());
        remove_if_exists(&path);
    }

    #[cfg(not(windows))]
    #[test]
    fn native_output_is_explicitly_unsupported_off_windows() {
        let error = native::generate_pdf(
            unsafe { std::mem::MaybeUninit::<&AppHandle<Wry>>::zeroed().assume_init() },
            "",
            "",
            Path::new("unused"),
        )
        .expect_err("unsupported");
        assert_eq!(error.code, "PLATFORM_UNSUPPORTED");
    }
}
