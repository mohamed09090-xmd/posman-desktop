use rusqlite::{params, OptionalExtension, Row, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use super::{
    error::{Phase09Error, Phase09Result},
    models::{
        CreateTemplateDraftRequest, PublishTemplateRequest, RetireTemplateRequest,
        TemplateConfiguration, TemplateDetail, TemplateDraftView, TemplateKeyRequest,
        TemplateSummary, TemplateVersionView, UpdateTemplateDraftRequest,
    },
    new_id, normalize_locale, now_iso,
    rendering::validate_template_configuration,
    Phase09Service,
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
const STORAGE_LOCALES: &[&str] = &["ar", "fr"];
const STRUCTURED_HTML_MARKER: &str = "POSMAN_STRUCTURED_TEMPLATE_V1";
const SAFE_A4_CSS_MARKER: &str = "POSMAN_A4_SAFE_CSS_V1";

impl Phase09Service {
    pub fn list_templates(&self, _: ()) -> Phase09Result<Vec<TemplateSummary>> {
        let context = self.authorize("documents.templates.view")?;
        self.ensure_company_templates(&context.company_id, &context.user_id)?;
        let connection = self.phase05.phase09_open_maintenance()?;
        let mut summaries = Vec::with_capacity(DOCUMENT_TYPES.len() * STORAGE_LOCALES.len());
        for document_type in DOCUMENT_TYPES {
            for storage_locale in STORAGE_LOCALES {
                summaries.push(load_summary(
                    &connection,
                    &context.company_id,
                    document_type,
                    storage_locale,
                )?);
            }
        }
        Ok(summaries)
    }

    pub fn get_template(&self, request: TemplateKeyRequest) -> Phase09Result<TemplateDetail> {
        validate_document_type(&request.document_type)?;
        let ui_locale = normalize_locale(&request.locale)?;
        let storage_locale = storage_locale(ui_locale);
        let context = self.authorize("documents.templates.view")?;
        self.ensure_company_templates(&context.company_id, &context.user_id)?;
        let connection = self.phase05.phase09_open_maintenance()?;
        let summary = load_summary(
            &connection,
            &context.company_id,
            &request.document_type,
            storage_locale,
        )?;
        let draft = connection
            .query_row(
                r#"SELECT id,document_template_id,document_type,locale,display_name,
                          title_ar,title_fr,show_logo,show_company_identity,show_trade_register,
                          show_tax_identifier,show_partner_address,show_payment_information,
                          footer_ar,footer_fr,spacing,orientation,optional_sections_json,
                          base_template_version_id,row_version,updated_at
                   FROM phase09_template_drafts
                   WHERE company_id=?1 AND document_template_id=?2 AND locale=?3 AND state='DRAFT'
                   ORDER BY version_number DESC LIMIT 1"#,
                params![context.company_id, summary.template_id, storage_locale],
                map_draft,
            )
            .optional()?;
        let mut statement = connection.prepare(
            r#"SELECT c.template_version_id,v.version_number,c.locale,v.content_hash_sha256,
                      CASE WHEN r.id IS NULL THEN 'PUBLISHED' ELSE 'RETIRED' END,
                      c.published_at,COALESCE(c.published_by,''),
                      CASE WHEN r.id IS NULL THEN 1 ELSE 2 END
               FROM phase09_template_version_configs c
               JOIN document_template_versions v
                 ON v.id=c.template_version_id AND v.company_id=c.company_id
               LEFT JOIN phase09_template_retirements r
                 ON r.company_id=c.company_id AND r.template_version_id=c.template_version_id
               WHERE c.company_id=?1 AND c.document_template_id=?2 AND c.locale=?3
               ORDER BY v.version_number DESC"#,
        )?;
        let versions = statement
            .query_map(
                params![context.company_id, summary.template_id, storage_locale],
                map_version,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TemplateDetail {
            summary,
            draft,
            versions,
        })
    }

    pub fn create_template_draft(
        &self,
        request: CreateTemplateDraftRequest,
    ) -> Phase09Result<TemplateDraftView> {
        validate_document_type(&request.document_type)?;
        let ui_locale = normalize_locale(&request.locale)?;
        let storage_locale = storage_locale(ui_locale);
        let context = self.authorize("documents.templates.manage")?;
        self.ensure_company_templates(&context.company_id, &context.user_id)?;
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let template_id: String = transaction.query_row(
            "SELECT id FROM document_templates WHERE company_id=?1 AND document_type=?2 AND code=?3",
            params![
                context.company_id,
                request.document_type,
                template_code(&request.document_type)
            ],
            |row| row.get(0),
        )?;
        let existing: Option<String> = transaction
            .query_row(
                "SELECT id FROM phase09_template_drafts WHERE company_id=?1 AND document_template_id=?2 AND locale=?3 AND state='DRAFT' ORDER BY version_number DESC LIMIT 1",
                params![context.company_id, template_id, storage_locale],
                |row| row.get(0),
            )
            .optional()?;
        if existing.is_some() {
            return Err(Phase09Error::new(
                "TEMPLATE_DRAFT_EXISTS",
                "An editable draft already exists for this template.",
                false,
            ));
        }
        let active = load_active_config(
            &transaction,
            &context.company_id,
            &template_id,
            storage_locale,
        )?;
        let configuration = active
            .as_ref()
            .map(|value| value.configuration.clone())
            .unwrap_or_else(|| default_configuration(&request.document_type));
        let default_name = localized_title(&request.document_type, ui_locale).to_owned();
        let display_name = request
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or(default_name);
        validate_display_name(&display_name)?;
        let version_number = next_version_number(&transaction, &context.company_id, &template_id)?;
        let draft_id = new_id();
        let now = now_iso()?;
        insert_draft(
            &transaction,
            DraftInsert {
                id: &draft_id,
                company_id: &context.company_id,
                template_id: &template_id,
                document_type: &request.document_type,
                storage_locale,
                version_number,
                base_version_id: active.as_ref().map(|value| value.version_id.as_str()),
                display_name: &display_name,
                configuration: &configuration,
                now: &now,
                user_id: &context.user_id,
                state: "DRAFT",
            },
        )?;
        Self::audit_success(
            &transaction,
            &context,
            "DOCUMENT_TEMPLATE_DRAFT_CREATED",
            "PHASE09_TEMPLATE_DRAFT",
            &draft_id,
            Some(&serde_json::json!({
                "documentType": request.document_type,
                "locale": ui_locale,
                "versionNumber": version_number,
            })),
        )?;
        let draft = load_draft(&transaction, &context.company_id, &draft_id)?;
        transaction.commit()?;
        Ok(draft)
    }

    pub fn update_template_draft(
        &self,
        request: UpdateTemplateDraftRequest,
    ) -> Phase09Result<TemplateDraftView> {
        if request.expected_row_version < 1 {
            return Err(Phase09Error::validation("Invalid template row version."));
        }
        validate_display_name(&request.display_name)?;
        validate_template_configuration(&request.configuration)?;
        let context = self.authorize("documents.templates.manage")?;
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = transaction.execute(
            r#"UPDATE phase09_template_drafts SET
                   display_name=?1,title_ar=?2,title_fr=?3,show_logo=?4,
                   show_company_identity=?5,show_trade_register=?6,show_tax_identifier=?7,
                   show_partner_address=?8,show_payment_information=?9,footer_ar=?10,
                   footer_fr=?11,spacing=?12,orientation=?13,optional_sections_json=?14,
                   updated_at=?15,updated_by=?16,row_version=row_version+1
               WHERE id=?17 AND company_id=?18 AND state='DRAFT' AND row_version=?19"#,
            params![
                request.display_name.trim(),
                request.configuration.document_title_ar.trim(),
                request.configuration.document_title_fr.trim(),
                bool_int(request.configuration.show_logo),
                bool_int(request.configuration.show_company_identity),
                bool_int(request.configuration.show_trade_register),
                bool_int(request.configuration.show_tax_identifier),
                bool_int(request.configuration.show_partner_address),
                bool_int(request.configuration.show_payment_information),
                request.configuration.footer_text_ar.trim(),
                request.configuration.footer_text_fr.trim(),
                request.configuration.spacing,
                request.configuration.orientation,
                serde_json::to_string(&request.configuration.enabled_sections)
                    .map_err(|_| Phase09Error::internal())?,
                now_iso()?,
                context.user_id,
                request.draft_id,
                context.company_id,
                request.expected_row_version,
            ],
        )?;
        if changed != 1 {
            let exists = transaction
                .query_row(
                    "SELECT 1 FROM phase09_template_drafts WHERE id=?1 AND company_id=?2 AND state='DRAFT'",
                    params![request.draft_id, context.company_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .is_some();
            return Err(if exists {
                Phase09Error::concurrency()
            } else {
                Phase09Error::not_found("template draft")
            });
        }
        Self::audit_success(
            &transaction,
            &context,
            "DOCUMENT_TEMPLATE_DRAFT_UPDATED",
            "PHASE09_TEMPLATE_DRAFT",
            &request.draft_id,
            None,
        )?;
        let draft = load_draft(&transaction, &context.company_id, &request.draft_id)?;
        transaction.commit()?;
        Ok(draft)
    }

    pub fn publish_template(
        &self,
        request: PublishTemplateRequest,
    ) -> Phase09Result<TemplateVersionView> {
        if !request.confirmed || request.expected_row_version < 1 {
            return Err(Phase09Error::validation(
                "Template publication requires explicit confirmation.",
            ));
        }
        let context = self.authorize("documents.templates.manage")?;
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let draft = load_publishable_draft(&transaction, &context.company_id, &request.draft_id)?;
        if draft.row_version != request.expected_row_version {
            return Err(Phase09Error::concurrency());
        }
        validate_template_configuration(&draft.configuration)?;
        let config_json =
            serde_json::to_string(&draft.configuration).map_err(|_| Phase09Error::internal())?;
        let version_id = new_id();
        let published_at = now_iso()?;
        let content_sha256 = template_content_hash(
            &draft.template_id,
            &draft.document_type,
            storage_locale(&draft.locale),
            draft.version_number,
            &config_json,
        );
        transaction.execute(
            r#"INSERT INTO document_template_versions(
                   id,company_id,document_template_id,version_number,html_template,css_template,
                   content_hash_sha256,is_published,created_at,created_by
               ) VALUES(?1,?2,?3,?4,?5,?6,?7,1,?8,?9)"#,
            params![
                version_id,
                context.company_id,
                draft.template_id,
                draft.version_number,
                STRUCTURED_HTML_MARKER,
                SAFE_A4_CSS_MARKER,
                content_sha256,
                published_at,
                context.user_id,
            ],
        )?;
        transaction.execute(
            r#"INSERT INTO phase09_template_version_configs(
                   template_version_id,company_id,document_template_id,source_draft_id,
                   document_type,locale,config_json,published_at,published_by
               ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)"#,
            params![
                version_id,
                context.company_id,
                draft.template_id,
                draft.draft_id,
                draft.document_type,
                storage_locale(&draft.locale),
                config_json,
                published_at,
                context.user_id,
            ],
        )?;
        let changed = transaction.execute(
            "UPDATE phase09_template_drafts SET state='PUBLISHED',updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE id=?3 AND company_id=?4 AND state='DRAFT' AND row_version=?5",
            params![
                published_at,
                context.user_id,
                draft.draft_id,
                context.company_id,
                request.expected_row_version
            ],
        )?;
        if changed != 1 {
            return Err(Phase09Error::concurrency());
        }
        transaction.execute(
            r#"UPDATE document_templates SET
                   name_ar=CASE WHEN ?1='ar' THEN ?2 ELSE name_ar END,
                   name_fr=CASE WHEN ?1='fr' THEN ?2 ELSE name_fr END,
                   updated_at=?3,updated_by=?4,row_version=row_version+1
               WHERE id=?5 AND company_id=?6"#,
            params![
                storage_locale(&draft.locale),
                draft.display_name,
                published_at,
                context.user_id,
                draft.template_id,
                context.company_id,
            ],
        )?;
        Self::audit_success(
            &transaction,
            &context,
            "DOCUMENT_TEMPLATE_PUBLISHED",
            "DOCUMENT_TEMPLATE_VERSION",
            &version_id,
            Some(&serde_json::json!({
                "draftId": draft.draft_id,
                "versionNumber": draft.version_number,
                "contentSha256": content_sha256,
            })),
        )?;
        transaction.commit()?;
        Ok(TemplateVersionView {
            version_id,
            version_number: draft.version_number,
            locale: draft.locale,
            content_sha256,
            status: "PUBLISHED".to_owned(),
            published_at,
            published_by: context.user_id,
            row_version: 1,
        })
    }

    pub fn retire_template(
        &self,
        request: RetireTemplateRequest,
    ) -> Phase09Result<TemplateVersionView> {
        if !request.confirmed || request.expected_row_version != 1 {
            return Err(Phase09Error::validation(
                "Template retirement requires the current published version and explicit confirmation.",
            ));
        }
        let context = self.authorize("documents.templates.manage")?;
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = load_version(
            &transaction,
            &context.company_id,
            &request.template_version_id,
        )?;
        if current.status == "RETIRED" {
            return Ok(current);
        }
        let storage = storage_locale(&current.locale);
        let replacement_exists: bool = transaction
            .query_row(
                r#"SELECT EXISTS(
                       SELECT 1 FROM phase09_template_version_configs c
                       JOIN document_template_versions v ON v.id=c.template_version_id
                       LEFT JOIN phase09_template_retirements r
                         ON r.company_id=c.company_id AND r.template_version_id=c.template_version_id
                       WHERE c.company_id=?1 AND c.document_template_id=(
                           SELECT document_template_id FROM phase09_template_version_configs
                           WHERE company_id=?1 AND template_version_id=?2
                       ) AND c.locale=?3 AND c.template_version_id<>?2 AND r.id IS NULL
                   )"#,
                params![context.company_id, request.template_version_id, storage],
                |row| row.get::<_, i64>(0),
            )?
            == 1;
        if !replacement_exists {
            return Err(Phase09Error::new(
                "TEMPLATE_REPLACEMENT_REQUIRED",
                "Publish a replacement before retiring the active template version.",
                false,
            ));
        }
        let retired_at = now_iso()?;
        transaction.execute(
            r#"INSERT INTO phase09_template_retirements(
                   id,company_id,template_version_id,retired_at,retired_by,reason
               ) VALUES(?1,?2,?3,?4,?5,'REPLACED')"#,
            params![
                new_id(),
                context.company_id,
                request.template_version_id,
                retired_at,
                context.user_id,
            ],
        )?;
        Self::audit_success(
            &transaction,
            &context,
            "DOCUMENT_TEMPLATE_RETIRED",
            "DOCUMENT_TEMPLATE_VERSION",
            &request.template_version_id,
            None,
        )?;
        transaction.commit()?;
        Ok(TemplateVersionView {
            status: "RETIRED".to_owned(),
            row_version: 2,
            ..current
        })
    }

    pub(crate) fn ensure_company_templates(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Phase09Result<()> {
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        for document_type in DOCUMENT_TYPES {
            let template_id =
                ensure_template_master(&transaction, company_id, user_id, document_type)?;
            for storage in STORAGE_LOCALES {
                let exists: bool = transaction
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM phase09_template_version_configs WHERE company_id=?1 AND document_template_id=?2 AND locale=?3)",
                        params![company_id, template_id, storage],
                        |row| row.get::<_, i64>(0),
                    )?
                    == 1;
                if !exists {
                    publish_default(
                        &transaction,
                        company_id,
                        user_id,
                        &template_id,
                        document_type,
                        storage,
                    )?;
                }
            }
        }
        transaction.commit()?;
        Ok(())
    }
}

struct DraftInsert<'a> {
    id: &'a str,
    company_id: &'a str,
    template_id: &'a str,
    document_type: &'a str,
    storage_locale: &'a str,
    version_number: i64,
    base_version_id: Option<&'a str>,
    display_name: &'a str,
    configuration: &'a TemplateConfiguration,
    now: &'a str,
    user_id: &'a str,
    state: &'a str,
}

struct ActiveConfig {
    version_id: String,
    configuration: TemplateConfiguration,
}

fn publish_default(
    transaction: &Transaction<'_>,
    company_id: &str,
    user_id: &str,
    template_id: &str,
    document_type: &str,
    storage: &str,
) -> Phase09Result<()> {
    let configuration = default_configuration(document_type);
    validate_template_configuration(&configuration)?;
    let version_number = next_version_number(transaction, company_id, template_id)?;
    let draft_id = new_id();
    let now = now_iso()?;
    let ui_locale = ui_locale(storage);
    let display_name = localized_title(document_type, ui_locale);
    insert_draft(
        transaction,
        DraftInsert {
            id: &draft_id,
            company_id,
            template_id,
            document_type,
            storage_locale: storage,
            version_number,
            base_version_id: None,
            display_name,
            configuration: &configuration,
            now: &now,
            user_id,
            state: "PUBLISHED",
        },
    )?;
    let config_json =
        serde_json::to_string(&configuration).map_err(|_| Phase09Error::internal())?;
    let version_id = new_id();
    let content_sha256 = template_content_hash(
        template_id,
        document_type,
        storage,
        version_number,
        &config_json,
    );
    transaction.execute(
        r#"INSERT INTO document_template_versions(
               id,company_id,document_template_id,version_number,html_template,css_template,
               content_hash_sha256,is_published,created_at,created_by
           ) VALUES(?1,?2,?3,?4,?5,?6,?7,1,?8,?9)"#,
        params![
            version_id,
            company_id,
            template_id,
            version_number,
            STRUCTURED_HTML_MARKER,
            SAFE_A4_CSS_MARKER,
            content_sha256,
            now,
            user_id,
        ],
    )?;
    transaction.execute(
        r#"INSERT INTO phase09_template_version_configs(
               template_version_id,company_id,document_template_id,source_draft_id,
               document_type,locale,config_json,published_at,published_by
           ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)"#,
        params![
            version_id,
            company_id,
            template_id,
            draft_id,
            document_type,
            storage,
            config_json,
            now,
            user_id,
        ],
    )?;
    Ok(())
}

fn insert_draft(transaction: &Transaction<'_>, value: DraftInsert<'_>) -> Phase09Result<()> {
    transaction.execute(
        r#"INSERT INTO phase09_template_drafts(
               id,company_id,document_template_id,document_type,locale,version_number,
               base_template_version_id,state,display_name,title_ar,title_fr,show_logo,
               show_company_identity,show_trade_register,show_tax_identifier,
               show_partner_address,show_payment_information,footer_ar,footer_fr,spacing,
               orientation,optional_sections_json,created_at,created_by,updated_at,updated_by,
               row_version
           ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,
                    ?18,?19,?20,?21,?22,?23,?24,?23,?24,1)"#,
        params![
            value.id,
            value.company_id,
            value.template_id,
            value.document_type,
            value.storage_locale,
            value.version_number,
            value.base_version_id,
            value.state,
            value.display_name,
            value.configuration.document_title_ar,
            value.configuration.document_title_fr,
            bool_int(value.configuration.show_logo),
            bool_int(value.configuration.show_company_identity),
            bool_int(value.configuration.show_trade_register),
            bool_int(value.configuration.show_tax_identifier),
            bool_int(value.configuration.show_partner_address),
            bool_int(value.configuration.show_payment_information),
            value.configuration.footer_text_ar,
            value.configuration.footer_text_fr,
            value.configuration.spacing,
            value.configuration.orientation,
            serde_json::to_string(&value.configuration.enabled_sections)
                .map_err(|_| Phase09Error::internal())?,
            value.now,
            value.user_id,
        ],
    )?;
    Ok(())
}

fn ensure_template_master(
    transaction: &Transaction<'_>,
    company_id: &str,
    user_id: &str,
    document_type: &str,
) -> Phase09Result<String> {
    let code = template_code(document_type);
    if let Some(id) = transaction
        .query_row(
            "SELECT id FROM document_templates WHERE company_id=?1 AND code=?2 AND document_type=?3",
            params![company_id, code, document_type],
            |row| row.get(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    let id = new_id();
    let now = now_iso()?;
    let (title_ar, title_fr) = default_titles(document_type);
    transaction.execute(
        r#"INSERT INTO document_templates(
               id,company_id,code,document_type,name_ar,name_fr,is_active,
               created_at,created_by,updated_at,updated_by,row_version
           ) VALUES(?1,?2,?3,?4,?5,?6,1,?7,?8,?7,?8,1)"#,
        params![
            id,
            company_id,
            code,
            document_type,
            title_ar,
            title_fr,
            now,
            user_id,
        ],
    )?;
    Ok(id)
}

fn load_summary(
    connection: &rusqlite::Connection,
    company_id: &str,
    document_type: &str,
    storage: &str,
) -> Phase09Result<TemplateSummary> {
    let template = connection
        .query_row(
            "SELECT id,name_ar,COALESCE(name_fr,name_ar) FROM document_templates WHERE company_id=?1 AND code=?2 AND document_type=?3",
            params![company_id, template_code(document_type), document_type],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()?
        .ok_or_else(|| Phase09Error::not_found("template"))?;
    let active = connection
        .query_row(
            r#"SELECT c.template_version_id,v.version_number,v.content_hash_sha256,d.display_name
               FROM phase09_template_version_configs c
               JOIN document_template_versions v ON v.id=c.template_version_id
               JOIN phase09_template_drafts d ON d.id=c.source_draft_id
               LEFT JOIN phase09_template_retirements r
                 ON r.company_id=c.company_id AND r.template_version_id=c.template_version_id
               WHERE c.company_id=?1 AND c.document_template_id=?2 AND c.locale=?3 AND r.id IS NULL
               ORDER BY c.published_at DESC,v.version_number DESC LIMIT 1"#,
            params![company_id, template.0, storage],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?;
    let draft = connection
        .query_row(
            "SELECT id,row_version,display_name FROM phase09_template_drafts WHERE company_id=?1 AND document_template_id=?2 AND locale=?3 AND state='DRAFT' ORDER BY version_number DESC LIMIT 1",
            params![company_id, template.0, storage],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()?;
    let display_name = draft
        .as_ref()
        .map(|value| value.2.clone())
        .or_else(|| active.as_ref().map(|value| value.3.clone()))
        .unwrap_or_else(|| {
            if storage == "ar" {
                template.1
            } else {
                template.2
            }
        });
    Ok(TemplateSummary {
        template_id: template.0,
        document_type: document_type.to_owned(),
        locale: ui_locale(storage).to_owned(),
        display_name,
        active_version_id: active.as_ref().map(|value| value.0.clone()),
        active_version_number: active.as_ref().map(|value| value.1),
        active_content_sha256: active.as_ref().map(|value| value.2.clone()),
        draft_id: draft.as_ref().map(|value| value.0.clone()),
        draft_row_version: draft.as_ref().map(|value| value.1),
        state: if draft.is_some() {
            "DRAFT".to_owned()
        } else if active.is_some() {
            "PUBLISHED".to_owned()
        } else {
            "RETIRED".to_owned()
        },
    })
}

fn load_active_config(
    transaction: &Transaction<'_>,
    company_id: &str,
    template_id: &str,
    storage: &str,
) -> Phase09Result<Option<ActiveConfig>> {
    transaction
        .query_row(
            r#"SELECT c.template_version_id,c.config_json
               FROM phase09_template_version_configs c
               LEFT JOIN phase09_template_retirements r
                 ON r.company_id=c.company_id AND r.template_version_id=c.template_version_id
               WHERE c.company_id=?1 AND c.document_template_id=?2 AND c.locale=?3 AND r.id IS NULL
               ORDER BY c.published_at DESC LIMIT 1"#,
            params![company_id, template_id, storage],
            |row| {
                let raw: String = row.get(1)?;
                let configuration = serde_json::from_str(&raw).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(ActiveConfig {
                    version_id: row.get(0)?,
                    configuration,
                })
            },
        )
        .optional()
        .map_err(Phase09Error::from)
}

fn load_draft(
    transaction: &Transaction<'_>,
    company_id: &str,
    draft_id: &str,
) -> Phase09Result<TemplateDraftView> {
    transaction
        .query_row(
            r#"SELECT id,document_template_id,document_type,locale,display_name,
                      title_ar,title_fr,show_logo,show_company_identity,show_trade_register,
                      show_tax_identifier,show_partner_address,show_payment_information,
                      footer_ar,footer_fr,spacing,orientation,optional_sections_json,
                      base_template_version_id,row_version,updated_at
               FROM phase09_template_drafts WHERE id=?1 AND company_id=?2"#,
            params![draft_id, company_id],
            map_draft,
        )
        .optional()?
        .ok_or_else(|| Phase09Error::not_found("template draft"))
}

fn load_publishable_draft(
    transaction: &Transaction<'_>,
    company_id: &str,
    draft_id: &str,
) -> Phase09Result<PublishableDraft> {
    transaction
        .query_row(
            r#"SELECT id,document_template_id,document_type,locale,version_number,display_name,
                      title_ar,title_fr,show_logo,show_company_identity,show_trade_register,
                      show_tax_identifier,show_partner_address,show_payment_information,
                      footer_ar,footer_fr,spacing,orientation,optional_sections_json,row_version
               FROM phase09_template_drafts
               WHERE id=?1 AND company_id=?2 AND state='DRAFT'"#,
            params![draft_id, company_id],
            |row| {
                let storage: String = row.get(3)?;
                let sections: String = row.get(18)?;
                Ok(PublishableDraft {
                    draft_id: row.get(0)?,
                    template_id: row.get(1)?,
                    document_type: row.get(2)?,
                    locale: ui_locale(&storage).to_owned(),
                    version_number: row.get(4)?,
                    display_name: row.get(5)?,
                    configuration: TemplateConfiguration {
                        document_title_ar: row.get(6)?,
                        document_title_fr: row.get(7)?,
                        show_logo: row.get::<_, i64>(8)? == 1,
                        show_company_identity: row.get::<_, i64>(9)? == 1,
                        show_trade_register: row.get::<_, i64>(10)? == 1,
                        show_tax_identifier: row.get::<_, i64>(11)? == 1,
                        show_partner_address: row.get::<_, i64>(12)? == 1,
                        show_payment_information: row.get::<_, i64>(13)? == 1,
                        footer_text_ar: row.get(14)?,
                        footer_text_fr: row.get(15)?,
                        spacing: row.get(16)?,
                        orientation: row.get(17)?,
                        enabled_sections: serde_json::from_str(&sections).unwrap_or_default(),
                    },
                    row_version: row.get(19)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| Phase09Error::not_found("template draft"))
}

fn load_version(
    transaction: &Transaction<'_>,
    company_id: &str,
    version_id: &str,
) -> Phase09Result<TemplateVersionView> {
    transaction
        .query_row(
            r#"SELECT c.template_version_id,v.version_number,c.locale,v.content_hash_sha256,
                      CASE WHEN r.id IS NULL THEN 'PUBLISHED' ELSE 'RETIRED' END,
                      c.published_at,COALESCE(c.published_by,''),
                      CASE WHEN r.id IS NULL THEN 1 ELSE 2 END
               FROM phase09_template_version_configs c
               JOIN document_template_versions v ON v.id=c.template_version_id
               LEFT JOIN phase09_template_retirements r
                 ON r.company_id=c.company_id AND r.template_version_id=c.template_version_id
               WHERE c.company_id=?1 AND c.template_version_id=?2"#,
            params![company_id, version_id],
            map_version,
        )
        .optional()?
        .ok_or_else(|| Phase09Error::not_found("template version"))
}

fn map_draft(row: &Row<'_>) -> rusqlite::Result<TemplateDraftView> {
    let storage: String = row.get(3)?;
    let sections: String = row.get(17)?;
    Ok(TemplateDraftView {
        draft_id: row.get(0)?,
        template_id: row.get(1)?,
        document_type: row.get(2)?,
        locale: ui_locale(&storage).to_owned(),
        display_name: row.get(4)?,
        configuration: TemplateConfiguration {
            document_title_ar: row.get(5)?,
            document_title_fr: row.get(6)?,
            show_logo: row.get::<_, i64>(7)? == 1,
            show_company_identity: row.get::<_, i64>(8)? == 1,
            show_trade_register: row.get::<_, i64>(9)? == 1,
            show_tax_identifier: row.get::<_, i64>(10)? == 1,
            show_partner_address: row.get::<_, i64>(11)? == 1,
            show_payment_information: row.get::<_, i64>(12)? == 1,
            footer_text_ar: row.get(13)?,
            footer_text_fr: row.get(14)?,
            spacing: row.get(15)?,
            orientation: row.get(16)?,
            enabled_sections: serde_json::from_str(&sections).unwrap_or_default(),
        },
        base_template_version_id: row.get(18)?,
        row_version: row.get(19)?,
        updated_at: row.get(20)?,
    })
}

fn map_version(row: &Row<'_>) -> rusqlite::Result<TemplateVersionView> {
    let storage: String = row.get(2)?;
    Ok(TemplateVersionView {
        version_id: row.get(0)?,
        version_number: row.get(1)?,
        locale: ui_locale(&storage).to_owned(),
        content_sha256: row.get(3)?,
        status: row.get(4)?,
        published_at: row.get(5)?,
        published_by: row.get(6)?,
        row_version: row.get(7)?,
    })
}

struct PublishableDraft {
    draft_id: String,
    template_id: String,
    document_type: String,
    locale: String,
    version_number: i64,
    display_name: String,
    configuration: TemplateConfiguration,
    row_version: i64,
}

fn next_version_number(
    transaction: &Transaction<'_>,
    company_id: &str,
    template_id: &str,
) -> Phase09Result<i64> {
    transaction
        .query_row(
            "SELECT COALESCE(MAX(version_number),0)+1 FROM phase09_template_drafts WHERE company_id=?1 AND document_template_id=?2",
            params![company_id, template_id],
            |row| row.get(0),
        )
        .map_err(Phase09Error::from)
}

fn validate_document_type(value: &str) -> Phase09Result<()> {
    if DOCUMENT_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(Phase09Error::validation("Unsupported document type."))
    }
}

fn validate_display_name(value: &str) -> Phase09Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 160 {
        return Err(Phase09Error::validation(
            "Template display name must contain between 1 and 160 characters.",
        ));
    }
    Ok(())
}

fn storage_locale(ui_locale: &str) -> &'static str {
    if ui_locale == "fr-DZ" {
        "fr"
    } else {
        "ar"
    }
}

fn ui_locale(storage: &str) -> &'static str {
    if storage == "fr" {
        "fr-DZ"
    } else {
        "ar-DZ"
    }
}

fn bool_int(value: bool) -> i64 {
    i64::from(value)
}

fn template_code(document_type: &str) -> String {
    format!("P09-{document_type}")
}

fn template_content_hash(
    template_id: &str,
    document_type: &str,
    storage: &str,
    version_number: i64,
    config_json: &str,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"POSMAN_PHASE09_TEMPLATE_V1\n");
    digest.update(template_id.as_bytes());
    digest.update(b"\n");
    digest.update(document_type.as_bytes());
    digest.update(b"\n");
    digest.update(storage.as_bytes());
    digest.update(b"\n");
    digest.update(version_number.to_string().as_bytes());
    digest.update(b"\n");
    digest.update(config_json.as_bytes());
    format!("{:x}", digest.finalize())
}

fn default_configuration(document_type: &str) -> TemplateConfiguration {
    let (title_ar, title_fr) = default_titles(document_type);
    TemplateConfiguration {
        document_title_ar: title_ar.to_owned(),
        document_title_fr: title_fr.to_owned(),
        show_logo: true,
        show_company_identity: true,
        show_trade_register: true,
        show_tax_identifier: true,
        show_partner_address: true,
        show_payment_information: true,
        footer_text_ar: "وثيقة أنشئت محليًا بواسطة POSMAN".to_owned(),
        footer_text_fr: "Document généré localement par POSMAN".to_owned(),
        spacing: "NORMAL".to_owned(),
        orientation: "PORTRAIT".to_owned(),
        enabled_sections: vec!["TOTALS".to_owned(), "REFERENCES".to_owned()],
    }
}

fn localized_title(document_type: &str, locale: &str) -> &'static str {
    let (ar, fr) = default_titles(document_type);
    if locale == "fr-DZ" {
        fr
    } else {
        ar
    }
}

fn default_titles(document_type: &str) -> (&'static str, &'static str) {
    match document_type {
        "SALES_ORDER" => ("طلب بيع", "Commande client"),
        "DELIVERY_NOTE" => ("وصل تسليم", "Bon de livraison"),
        "SALES_INVOICE" => ("فاتورة بيع", "Facture de vente"),
        "SALES_CREDIT_NOTE" => ("إشعار دائن", "Avoir client"),
        "PURCHASE_ORDER" => ("طلب شراء", "Commande fournisseur"),
        "GOODS_RECEIPT" => ("وصل استلام", "Bon de réception"),
        "SUPPLIER_INVOICE" => ("فاتورة مورد", "Facture fournisseur"),
        "PURCHASE_RETURN" => ("إرجاع شراء", "Retour fournisseur"),
        "CUSTOMER_RECEIPT" => ("وصل قبض", "Reçu client"),
        "SUPPLIER_PAYMENT" => ("وصل دفع", "Paiement fournisseur"),
        _ => ("وثيقة", "Document"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_and_storage_locales_are_separated() {
        assert_eq!(storage_locale("ar-DZ"), "ar");
        assert_eq!(storage_locale("fr-DZ"), "fr");
        assert_eq!(ui_locale("ar"), "ar-DZ");
        assert_eq!(ui_locale("fr"), "fr-DZ");
    }

    #[test]
    fn every_required_document_type_has_safe_bilingual_defaults() {
        for document_type in DOCUMENT_TYPES {
            let configuration = default_configuration(document_type);
            assert!(!configuration.document_title_ar.is_empty());
            assert!(!configuration.document_title_fr.is_empty());
            validate_template_configuration(&configuration).expect("safe default");
        }
    }

    #[test]
    fn publication_hash_changes_with_locale_and_version() {
        let a = template_content_hash("t", "SALES_INVOICE", "ar", 1, "{}");
        let b = template_content_hash("t", "SALES_INVOICE", "fr", 2, "{}");
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }
}
