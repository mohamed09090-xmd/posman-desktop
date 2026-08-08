use rusqlite::{params, OptionalExtension, Row};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Wry};

use super::{
    checked_page,
    error::{Phase09Error, Phase09Result},
    models::{
        CanonicalDocumentPayload, DocumentLinePayload, DocumentRequest, Paged, PreviewContent,
        PreviewResult, RenderedDocumentKeyRequest, RenderedDocumentView, RenderedDocumentsRequest,
        TemplateConfiguration,
    },
    new_id, normalize_locale, rendering, Phase09Service,
};

const DOCUMENT_TYPES: &[&str] = &[
    "SALES_ORDER",
    "DELIVERY_NOTE",
    "SALES_INVOICE",
    "SALES_CREDIT_NOTE",
    "PURCHASE_ORDER",
    "GOODS_RECEIPT",
    "SUPPLIER_INVOICE",
    "PURCHASE_RETURN",
    "CUSTOMER_RECEIPT",
    "SUPPLIER_PAYMENT",
];

impl Phase09Service {
    pub fn preview_document(
        &self,
        app: &AppHandle<Wry>,
        request: DocumentRequest,
    ) -> Phase09Result<PreviewResult> {
        validate_document_request(&request)?;
        let locale = normalize_locale(&request.locale)?;
        let context = self.authorize("documents.render")?;
        let connection = self.phase05.phase09_open_maintenance()?;
        let template = load_published_template(
            &connection,
            &context.company_id,
            &request.document_type,
            locale,
        )?;
        let payload = load_canonical_payload(
            &connection,
            &context.company_id,
            &request.document_type,
            &request.source_document_id,
            locale,
        )?;
        let rendered = rendering::render_document(locale, &template.configuration, &payload)?;
        let preview_id = new_id();
        let direction = if locale == "ar-DZ" { "rtl" } else { "ltr" };
        let content = PreviewContent {
            preview_id: preview_id.clone(),
            locale: locale.to_owned(),
            direction: direction.to_owned(),
            html: rendered.html,
            css: rendered.css,
            content_sha256: rendered.content_sha256,
            integrity_state: "VERIFIED".to_owned(),
        };
        self.previews
            .lock()
            .map_err(|_| Phase09Error::internal())?
            .insert(preview_id.clone(), content);

        let label = format!("phase09-preview-{preview_id}");
        let url = WebviewUrl::App(format!("index.html?phase09Preview={preview_id}").into());
        if app.get_webview_window(&label).is_none() {
            WebviewWindowBuilder::new(app, &label, url)
                .title("POSMAN document preview")
                .inner_size(960.0, 760.0)
                .min_inner_size(720.0, 540.0)
                .build()
                .map_err(|_| Phase09Error::internal())?;
        }
        Ok(PreviewResult {
            preview_id,
            document_type: request.document_type,
            source_document_id: request.source_document_id,
            locale: locale.to_owned(),
            integrity_state: "VERIFIED".to_owned(),
        })
    }

    pub fn get_preview_content(&self, preview_id: String) -> Phase09Result<PreviewContent> {
        self.authorize("documents.render")?;
        let preview_id = preview_id.trim();
        if preview_id.is_empty() || preview_id.len() > 100 {
            return Err(Phase09Error::validation("Invalid preview identifier."));
        }
        self.previews
            .lock()
            .map_err(|_| Phase09Error::internal())?
            .get(preview_id)
            .cloned()
            .ok_or_else(|| Phase09Error::not_found("document preview"))
    }

    pub fn list_rendered_documents(
        &self,
        request: RenderedDocumentsRequest,
    ) -> Phase09Result<Paged<RenderedDocumentView>> {
        let context = self.authorize("documents.templates.view")?;
        let (page, page_size) = checked_page(request.page, request.page_size, 200)?;
        if let Some(document_type) = request.document_type.as_deref() {
            validate_document_type(document_type)?;
        }
        let connection = self.phase05.phase09_open_maintenance()?;
        let total = connection.query_row(
            r#"SELECT COUNT(*) FROM phase09_rendered_documents
               WHERE company_id=?1
                 AND (?2 IS NULL OR document_type=?2)
                 AND (?3 IS NULL OR source_document_id=?3)"#,
            params![
                context.company_id,
                request.document_type,
                request.source_document_id
            ],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            r#"SELECT id,document_type,source_document_id,source_document_number,
                      source_document_status,document_template_id,template_version_id,locale,
                      content_sha256,pdf_relative_path,pdf_sha256,pdf_size_bytes,rendered_at,
                      rendered_by
               FROM phase09_rendered_documents
               WHERE company_id=?1
                 AND (?2 IS NULL OR document_type=?2)
                 AND (?3 IS NULL OR source_document_id=?3)
               ORDER BY rendered_at DESC,id DESC LIMIT ?4 OFFSET ?5"#,
        )?;
        let items = statement
            .query_map(
                params![
                    context.company_id,
                    request.document_type,
                    request.source_document_id,
                    page_size,
                    (page - 1) * page_size
                ],
                map_rendered_document,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Paged {
            items,
            page,
            page_size,
            total,
        })
    }

    pub fn get_rendered_document(
        &self,
        request: RenderedDocumentKeyRequest,
    ) -> Phase09Result<RenderedDocumentView> {
        let context = self.authorize("documents.templates.view")?;
        let connection = self.phase05.phase09_open_maintenance()?;
        connection
            .query_row(
                r#"SELECT id,document_type,source_document_id,source_document_number,
                          source_document_status,document_template_id,template_version_id,locale,
                          content_sha256,pdf_relative_path,pdf_sha256,pdf_size_bytes,rendered_at,
                          rendered_by
                   FROM phase09_rendered_documents WHERE id=?1 AND company_id=?2"#,
                params![request.render_id, context.company_id],
                map_rendered_document,
            )
            .optional()?
            .ok_or_else(|| Phase09Error::not_found("rendered document"))
    }

    pub(crate) fn load_rendered_record(
        &self,
        company_id: &str,
        render_id: &str,
    ) -> Phase09Result<RenderedRecord> {
        let connection = self.phase05.phase09_open_maintenance()?;
        connection
            .query_row(
                r#"SELECT id,company_id,document_type,source_document_id,source_document_number,
                          source_document_status,document_template_id,template_version_id,locale,
                          canonical_payload_json,rendered_html,rendered_css,content_sha256,
                          pdf_relative_path,pdf_sha256,pdf_size_bytes,rendered_at,rendered_by
                   FROM phase09_rendered_documents WHERE id=?1 AND company_id=?2"#,
                params![render_id, company_id],
                |row| {
                    Ok(RenderedRecord {
                        render_id: row.get(0)?,
                        company_id: row.get(1)?,
                        document_type: row.get(2)?,
                        source_document_id: row.get(3)?,
                        source_document_number: row.get(4)?,
                        source_document_status: row.get(5)?,
                        template_id: row.get(6)?,
                        template_version_id: row.get(7)?,
                        locale: row.get(8)?,
                        canonical_payload_json: row.get(9)?,
                        rendered_html: row.get(10)?,
                        rendered_css: row.get(11)?,
                        content_sha256: row.get(12)?,
                        pdf_relative_path: row.get(13)?,
                        pdf_sha256: row.get(14)?,
                        pdf_size_bytes: row.get(15)?,
                        rendered_at: row.get(16)?,
                        rendered_by: row.get(17)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| Phase09Error::not_found("rendered document"))
    }

    pub(crate) fn prepare_document_snapshot(
        &self,
        request: &DocumentRequest,
    ) -> Phase09Result<PreparedDocument> {
        validate_document_request(request)?;
        let locale = normalize_locale(&request.locale)?;
        let context = self.authorize("documents.render")?;
        let connection = self.phase05.phase09_open_maintenance()?;
        let template = load_published_template(
            &connection,
            &context.company_id,
            &request.document_type,
            locale,
        )?;
        let payload = load_canonical_payload(
            &connection,
            &context.company_id,
            &request.document_type,
            &request.source_document_id,
            locale,
        )?;
        if !is_finalized_status(&payload.document_status) {
            return Err(Phase09Error::new(
                "DOCUMENT_NOT_FINALIZED",
                "Only a finalized source document can create a historical render.",
                false,
            ));
        }
        let rendered = rendering::render_document(locale, &template.configuration, &payload)?;
        Ok(PreparedDocument {
            company_id: context.company_id,
            user_id: context.user_id,
            template_id: template.template_id,
            template_version_id: template.template_version_id,
            payload,
            locale: locale.to_owned(),
            canonical_payload_json: rendered.canonical_payload_json,
            html: rendered.html,
            css: rendered.css,
            content_sha256: rendered.content_sha256,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedDocument {
    pub company_id: String,
    pub user_id: String,
    pub template_id: String,
    pub template_version_id: String,
    pub payload: CanonicalDocumentPayload,
    pub locale: String,
    pub canonical_payload_json: String,
    pub html: String,
    pub css: String,
    pub content_sha256: String,
}

#[derive(Clone, Debug)]
pub(crate) struct RenderedRecord {
    pub render_id: String,
    pub company_id: String,
    pub document_type: String,
    pub source_document_id: String,
    pub source_document_number: String,
    pub source_document_status: String,
    pub template_id: String,
    pub template_version_id: String,
    pub locale: String,
    pub canonical_payload_json: String,
    pub rendered_html: String,
    pub rendered_css: String,
    pub content_sha256: String,
    pub pdf_relative_path: String,
    pub pdf_sha256: String,
    pub pdf_size_bytes: i64,
    pub rendered_at: String,
    pub rendered_by: String,
}

struct PublishedTemplate {
    template_id: String,
    template_version_id: String,
    configuration: TemplateConfiguration,
}

fn load_published_template(
    connection: &rusqlite::Connection,
    company_id: &str,
    document_type: &str,
    locale: &str,
) -> Phase09Result<PublishedTemplate> {
    connection
        .query_row(
            r#"SELECT v.document_template_id,v.id,v.configuration_json
               FROM document_template_publications p
               JOIN document_template_versions v
                 ON v.id=p.template_version_id AND v.company_id=p.company_id
               JOIN document_templates t
                 ON t.id=v.document_template_id AND t.company_id=v.company_id
               WHERE p.company_id=?1 AND t.document_type=?2 AND p.locale=?3
                 AND p.status='PUBLISHED' AND v.is_published=1
               ORDER BY v.version_number DESC LIMIT 1"#,
            params![company_id, document_type, locale],
            |row| {
                let configuration_json: String = row.get(2)?;
                let configuration = serde_json::from_str(&configuration_json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(PublishedTemplate {
                    template_id: row.get(0)?,
                    template_version_id: row.get(1)?,
                    configuration,
                })
            },
        )
        .optional()?
        .ok_or_else(|| {
            Phase09Error::new(
                "PUBLISHED_TEMPLATE_REQUIRED",
                "No active published template exists for this company, document type, and locale.",
                false,
            )
        })
}

fn load_canonical_payload(
    connection: &rusqlite::Connection,
    company_id: &str,
    document_type: &str,
    source_document_id: &str,
    locale: &str,
) -> Phase09Result<CanonicalDocumentPayload> {
    match document_type {
        "CUSTOMER_RECEIPT" => load_payment_payload(
            connection,
            company_id,
            source_document_id,
            locale,
            "RECEIPT",
            document_type,
        ),
        "SUPPLIER_PAYMENT" => load_payment_payload(
            connection,
            company_id,
            source_document_id,
            locale,
            "DISBURSEMENT",
            document_type,
        ),
        _ => load_commercial_payload(
            connection,
            company_id,
            source_document_id,
            locale,
            document_type,
        ),
    }
}

fn load_commercial_payload(
    connection: &rusqlite::Connection,
    company_id: &str,
    source_document_id: &str,
    locale: &str,
    requested_type: &str,
) -> Phase09Result<CanonicalDocumentPayload> {
    let storage_type = storage_document_type(requested_type)?;
    let header = connection
        .query_row(
            r#"SELECT c.name_ar,c.legal_name,c.address_text,c.trade_register_number,c.tax_identifier,
                      c.phone,c.email,d.document_number,d.workflow_status,d.commercial_date,d.due_date,
                      d.currency_code,d.total_ht_minor,d.total_tax_minor,d.total_ttc_minor,d.notes,
                      p.display_name_ar,p.legal_name,p.display_name_fr,p.tax_identifier,
                      (SELECT trim(pa.address_line_1 || CASE WHEN pa.address_line_2 IS NULL OR pa.address_line_2='' THEN '' ELSE ', ' || pa.address_line_2 END || CASE WHEN pa.city IS NULL OR pa.city='' THEN '' ELSE ', ' || pa.city END)
                         FROM partner_addresses pa
                        WHERE pa.company_id=d.company_id AND pa.partner_id=d.partner_id
                          AND pa.is_active=1
                        ORDER BY pa.is_default DESC,pa.created_at ASC LIMIT 1),
                      pt.name_ar,pt.name_fr
               FROM commercial_documents d
               JOIN companies c ON c.id=d.company_id
               LEFT JOIN partners p ON p.id=d.partner_id AND p.company_id=d.company_id
               LEFT JOIN payment_terms pt ON pt.id=p.payment_term_id AND pt.company_id=d.company_id
               WHERE d.id=?1 AND d.company_id=?2 AND d.document_type=?3"#,
            params![source_document_id, company_id, storage_type],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, i64>(14)?,
                    row.get::<_, Option<String>>(15)?,
                    row.get::<_, Option<String>>(16)?,
                    row.get::<_, Option<String>>(17)?,
                    row.get::<_, Option<String>>(18)?,
                    row.get::<_, Option<String>>(19)?,
                    row.get::<_, Option<String>>(20)?,
                    row.get::<_, Option<String>>(21)?,
                    row.get::<_, Option<String>>(22)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| Phase09Error::not_found("source document"))?;

    let mut statement = connection.prepare(
        r#"SELECT line_number,product_code_snapshot,description_snapshot,unit_code_snapshot,
                  quantity_scaled,unit_price_scaled,line_discount_rate_scaled,
                  line_discount_minor+allocated_header_discount_minor,tax_rate_scaled,
                  line_ht_minor,line_tax_minor,line_ttc_minor
           FROM commercial_document_lines
           WHERE company_id=?1 AND document_id=?2 ORDER BY line_number"#,
    )?;
    let lines = statement
        .query_map(params![company_id, source_document_id], |row| {
            Ok(DocumentLinePayload {
                line_number: row.get(0)?,
                product_code: row.get(1)?,
                description: row.get(2)?,
                unit_code: row.get(3)?,
                quantity_scaled: row.get(4)?,
                unit_price_scaled: row.get(5)?,
                discount_rate_scaled: row.get(6)?,
                discount_minor: row.get(7)?,
                tax_rate_scaled: row.get(8)?,
                ht_minor: row.get(9)?,
                tax_minor: row.get(10)?,
                ttc_minor: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let partner_name = if locale == "ar-DZ" {
        header.16.clone().or(header.17.clone())
    } else {
        header.18.clone().or(header.17.clone())
    };
    let payment_information = if locale == "ar-DZ" {
        header.21
    } else {
        header.22
    };
    Ok(CanonicalDocumentPayload {
        company_name: header.0,
        company_legal_name: header.1,
        company_address: header.2,
        company_trade_register: header.3,
        company_tax_identifier: header.4,
        company_phone: header.5,
        company_email: header.6,
        partner_name,
        partner_address: header.20,
        partner_tax_identifier: header.19,
        document_type: requested_type.to_owned(),
        document_number: header.7,
        document_status: header.8,
        commercial_date: header.9,
        due_date: header.10,
        currency_code: header.11,
        total_ht_minor: header.12,
        total_tax_minor: header.13,
        total_ttc_minor: header.14,
        payment_information,
        references: vec![source_document_id.to_owned()],
        notes: header.15,
        lines,
    })
}

fn load_payment_payload(
    connection: &rusqlite::Connection,
    company_id: &str,
    source_document_id: &str,
    locale: &str,
    payment_kind: &str,
    document_type: &str,
) -> Phase09Result<CanonicalDocumentPayload> {
    let row = connection
        .query_row(
            r#"SELECT c.name_ar,c.legal_name,c.address_text,c.trade_register_number,c.tax_identifier,
                      c.phone,c.email,pay.payment_number,pay.status,pay.commercial_date,
                      pay.currency_code,pay.amount_minor,pay.external_reference,pay.notes,
                      p.display_name_ar,p.legal_name,p.display_name_fr,p.tax_identifier,
                      (SELECT trim(pa.address_line_1 || CASE WHEN pa.address_line_2 IS NULL OR pa.address_line_2='' THEN '' ELSE ', ' || pa.address_line_2 END || CASE WHEN pa.city IS NULL OR pa.city='' THEN '' ELSE ', ' || pa.city END)
                         FROM partner_addresses pa
                        WHERE pa.company_id=pay.company_id AND pa.partner_id=pay.partner_id
                          AND pa.is_active=1
                        ORDER BY pa.is_default DESC,pa.created_at ASC LIMIT 1),
                      pm.name_ar,pm.name_fr
               FROM payments pay
               JOIN companies c ON c.id=pay.company_id
               JOIN partners p ON p.id=pay.partner_id AND p.company_id=pay.company_id
               JOIN payment_methods pm ON pm.id=pay.payment_method_id AND pm.company_id=pay.company_id
               WHERE pay.id=?1 AND pay.company_id=?2 AND pay.payment_kind=?3"#,
            params![source_document_id, company_id, payment_kind],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, i64>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, Option<String>>(16)?,
                    row.get::<_, Option<String>>(17)?,
                    row.get::<_, Option<String>>(18)?,
                    row.get::<_, String>(19)?,
                    row.get::<_, String>(20)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| Phase09Error::not_found("payment"))?;
    let partner_name = if locale == "ar-DZ" {
        Some(row.14.clone())
    } else {
        row.16.clone().or(Some(row.15.clone()))
    };
    let description = if payment_kind == "RECEIPT" {
        if locale == "ar-DZ" {
            "وصل زبون"
        } else {
            "Reçu client"
        }
    } else if locale == "ar-DZ" {
        "دفع مورد"
    } else {
        "Paiement fournisseur"
    };
    let unit_price_scaled = row.11.checked_mul(100).ok_or_else(Phase09Error::internal)?;
    Ok(CanonicalDocumentPayload {
        company_name: row.0,
        company_legal_name: row.1,
        company_address: row.2,
        company_trade_register: row.3,
        company_tax_identifier: row.4,
        company_phone: row.5,
        company_email: row.6,
        partner_name,
        partner_address: row.18,
        partner_tax_identifier: row.17,
        document_type: document_type.to_owned(),
        document_number: row.7,
        document_status: row.8,
        commercial_date: row.9,
        due_date: None,
        currency_code: row.10,
        total_ht_minor: row.11,
        total_tax_minor: 0,
        total_ttc_minor: row.11,
        payment_information: Some(if locale == "ar-DZ" { row.19 } else { row.20 }),
        references: row.12.into_iter().collect(),
        notes: row.13,
        lines: vec![DocumentLinePayload {
            line_number: 1,
            product_code: payment_kind.to_owned(),
            description: description.to_owned(),
            unit_code: "UNIT".to_owned(),
            quantity_scaled: 1_000_000,
            unit_price_scaled,
            discount_rate_scaled: 0,
            discount_minor: 0,
            tax_rate_scaled: 0,
            ht_minor: row.11,
            tax_minor: 0,
            ttc_minor: row.11,
        }],
    })
}

fn map_rendered_document(row: &Row<'_>) -> rusqlite::Result<RenderedDocumentView> {
    Ok(RenderedDocumentView {
        render_id: row.get(0)?,
        document_type: row.get(1)?,
        source_document_id: row.get(2)?,
        source_document_number: row.get(3)?,
        source_document_status: row.get(4)?,
        template_id: row.get(5)?,
        template_version_id: row.get(6)?,
        locale: row.get(7)?,
        content_sha256: row.get(8)?,
        pdf_relative_path: row.get(9)?,
        pdf_sha256: row.get(10)?,
        pdf_size_bytes: row.get(11)?,
        rendered_at: row.get(12)?,
        rendered_by: row.get(13)?,
        integrity_state: "UNVERIFIED".to_owned(),
    })
}

fn validate_document_request(request: &DocumentRequest) -> Phase09Result<()> {
    validate_document_type(&request.document_type)?;
    normalize_locale(&request.locale)?;
    if request.source_document_id.trim().is_empty() || request.source_document_id.len() > 100 {
        return Err(Phase09Error::validation(
            "Invalid source document identifier.",
        ));
    }
    Ok(())
}

fn validate_document_type(value: &str) -> Phase09Result<()> {
    if DOCUMENT_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(Phase09Error::validation("Unsupported document type."))
    }
}

fn storage_document_type(value: &str) -> Phase09Result<&'static str> {
    match value {
        "SALES_ORDER" => Ok("SALES_ORDER"),
        "DELIVERY_NOTE" => Ok("DELIVERY_NOTE"),
        "SALES_INVOICE" => Ok("SALES_INVOICE"),
        "SALES_CREDIT_NOTE" => Ok("SALES_CREDIT_NOTE"),
        "PURCHASE_ORDER" => Ok("PURCHASE_ORDER"),
        "GOODS_RECEIPT" => Ok("PURCHASE_RECEIPT"),
        "SUPPLIER_INVOICE" => Ok("PURCHASE_INVOICE"),
        "PURCHASE_RETURN" => Ok("PURCHASE_RETURN"),
        _ => Err(Phase09Error::validation(
            "Unsupported commercial document type.",
        )),
    }
}

fn is_finalized_status(value: &str) -> bool {
    !matches!(value, "DRAFT" | "CANCELLED" | "ON_HOLD")
}
