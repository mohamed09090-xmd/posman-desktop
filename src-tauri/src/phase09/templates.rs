use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};
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

impl Phase09Service {
    pub fn list_templates(&self, _: ()) -> Phase09Result<Vec<TemplateSummary>> {
        let context = self.authorize("documents.templates.view")?;
        self.ensure_company_templates(&context.company_id, &context.user_id)?;
        let connection = self.phase05.phase06_open()?;
        let mut statement = connection.prepare(TEMPLATE_SUMMARY_SQL)?;
        let rows = statement.query_map(params![context.company_id], map_template_summary)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Phase09Error::from)
    }

    pub fn get_template(&self, request: TemplateKeyRequest) -> Phase09Result<TemplateDetail> {
        validate_document_type(&request.document_type)?;
        let locale = normalize_locale(&request.locale)?;
        let context = self.authorize("documents.templates.view")?;
        self.ensure_company_templates(&context.company_id, &context.user_id)?;
        let connection = self.phase05.phase06_open()?;
        let summary = connection
            .query_row(
                &format!("{TEMPLATE_SUMMARY_SQL} AND t.document_type=?2 AND t.locale=?3"),
                params![context.company_id, request.document_type, locale],
                map_template_summary,
            )
            .optional()?
            .ok_or_else(|| Phase09Error::not_found("template"))?;
        let draft = connection
            .query_row(
                r#"SELECT id,document_template_id,document_type,locale,display_name,
                          configuration_json,base_template_version_id,row_version,updated_at
                   FROM document_template_drafts
                   WHERE company_id=?1 AND document_template_id=?2 AND locale=?3 AND status='DRAFT'"#,
                params![context.company_id, summary.template_id, locale],
                map_template_draft,
            )
            .optional()?;
        let mut version_statement = connection.prepare(
            r#"SELECT v.id,v.version_number,v.locale,v.content_hash_sha256,
                      p.status,COALESCE(v.published_at,v.created_at),COALESCE(v.published_by,v.created_by,''),p.row_version
               FROM document_template_versions v
               JOIN document_template_publications p ON p.template_version_id=v.id AND p.company_id=v.company_id
               WHERE v.company_id=?1 AND v.document_template_id=?2 AND v.locale=?3
               ORDER BY v.version_number DESC"#,
        )?;
        let versions = version_statement
            .query_map(params![context.company_id, summary.template_id, locale], |row| {
                Ok(TemplateVersionView {
                    version_id: row.get(0)?,
                    version_number: row.get(1)?,
                    locale: row.get(2)?,
                    content_sha256: row.get(3)?,
                    status: row.get(4)?,
                    published_at: row.get(5)?,
                    published_by: row.get(6)?,
                    row_version: row.get(7)?,
                })
            })?
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
        let locale = normalize_locale(&request.locale)?;
        let context = self.authorize("documents.templates.manage")?;
        self.ensure_company_templates(&context.company_id, &context.user_id)?;
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let template_id: String = transaction
            .query_row(
                "SELECT id FROM document_templates WHERE company_id=?1 AND document_type=?2 AND locale=?3",
                params![context.company_id, request.document_type, locale],
                |row| row.get(0),
            )?;
        if transaction
            .query_row(
                "SELECT id FROM document_template_drafts WHERE company_id=?1 AND document_template_id=?2 AND locale=?3 AND status='DRAFT'",
                params![context.company_id, template_id, locale],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .is_some()
        {
            return Err(Phase09Error::new(
                "TEMPLATE_DRAFT_EXISTS",
                "An editable draft already exists for this template.",
                false,
            ));
        }
        let source = transaction
            .query_row(
                r#"SELECT v.id,v.configuration_json,t.name_ar,COALESCE(t.name_fr,t.name_ar)
                   FROM document_template_publications p
                   JOIN document_template_versions v ON v.id=p.template_version_id
                   JOIN document_templates t ON t.id=v.document_template_id
                   WHERE p.company_id=?1 AND p.document_template_id=?2 AND p.locale=?3 AND p.status='PUBLISHED'"#,
                params![context.company_id, template_id, locale],
                |row| Ok((Some(row.get::<_, String>(0)?), row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
            )
            .optional()?
            .or_else(|| {
                transaction.query_row(
                    "SELECT NULL,configuration_json,display_name,display_name FROM document_template_defaults WHERE document_type=?1 AND locale=?2",
                    params![request.document_type, locale],
                    |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
                ).optional().ok().flatten()
            })
            .ok_or_else(|| Phase09Error::not_found("default template"))?;
        let now = now_iso()?;
        let draft_id = new_id();
        let display_name = request
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| if locale == "ar-DZ" { source.2.clone() } else { source.3.clone() });
        transaction.execute(
            r#"INSERT INTO document_template_drafts(
                   id,company_id,document_template_id,base_template_version_id,document_type,locale,
                   display_name,configuration_json,status,created_at,created_by,updated_at,updated_by,row_version
               ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'DRAFT',?9,?10,?9,?10,1)"#,
            params![draft_id, context.company_id, template_id, source.0, request.document_type, locale, display_name, source.1, now, context.user_id],
        )?;
        Self::audit_success(&transaction, &context, "DOCUMENT_TEMPLATE_DRAFT_CREATED", "DOCUMENT_TEMPLATE_DRAFT", &draft_id, None)?;
        let draft = transaction.query_row(
            r#"SELECT id,document_template_id,document_type,locale,display_name,
                      configuration_json,base_template_version_id,row_version,updated_at
               FROM document_template_drafts WHERE id=?1 AND company_id=?2"#,
            params![draft_id, context.company_id],
            map_template_draft,
        )?;
        transaction.commit()?;
        Ok(draft)
    }

    pub fn update_template_draft(
        &self,
        request: UpdateTemplateDraftRequest,
    ) -> Phase09Result<TemplateDraftView> {
        if request.expected_row_version < 1 || request.display_name.trim().is_empty() {
            return Err(Phase09Error::validation("Invalid template draft update."));
        }
        validate_template_configuration(&request.configuration)?;
        let context = self.authorize("documents.templates.manage")?;
        let configuration_json = serde_json::to_string(&request.configuration).map_err(|_| Phase09Error::internal())?;
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = transaction.execute(
            r#"UPDATE document_template_drafts
               SET display_name=?1,configuration_json=?2,updated_at=?3,updated_by=?4,row_version=row_version+1
               WHERE id=?5 AND company_id=?6 AND status='DRAFT' AND row_version=?7"#,
            params![request.display_name.trim(), configuration_json, now_iso()?, context.user_id, request.draft_id, context.company_id, request.expected_row_version],
        )?;
        if changed == 0 {
            let exists = transaction.query_row(
                "SELECT 1 FROM document_template_drafts WHERE id=?1 AND company_id=?2 AND status='DRAFT'",
                params![request.draft_id, context.company_id],
                |row| row.get::<_, i64>(0),
            ).optional()?.is_some();
            return Err(if exists { Phase09Error::concurrency() } else { Phase09Error::not_found("template draft") });
        }
        Self::audit_success(&transaction, &context, "DOCUMENT_TEMPLATE_DRAFT_UPDATED", "DOCUMENT_TEMPLATE_DRAFT", &request.draft_id, None)?;
        let draft = transaction.query_row(
            r#"SELECT id,document_template_id,document_type,locale,display_name,
                      configuration_json,base_template_version_id,row_version,updated_at
               FROM document_template_drafts WHERE id=?1 AND company_id=?2"#,
            params![request.draft_id, context.company_id],
            map_template_draft,
        )?;
        transaction.commit()?;
        Ok(draft)
    }

    pub fn publish_template(
        &self,
        request: PublishTemplateRequest,
    ) -> Phase09Result<TemplateVersionView> {
        if !request.confirmed || request.expected_row_version < 1 {
            return Err(Phase09Error::validation("Template publication requires explicit confirmation."));
        }
        let context = self.authorize("documents.templates.manage")?;
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let draft = transaction
            .query_row(
                r#"SELECT document_template_id,document_type,locale,display_name,configuration_json,row_version
                   FROM document_template_drafts
                   WHERE id=?1 AND company_id=?2 AND status='DRAFT'"#,
                params![request.draft_id, context.company_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, i64>(5)?)),
            )
            .optional()?
            .ok_or_else(|| Phase09Error::not_found("template draft"))?;
        if draft.5 != request.expected_row_version {
            return Err(Phase09Error::concurrency());
        }
        let configuration: TemplateConfiguration = serde_json::from_str(&draft.4)
            .map_err(|_| Phase09Error::validation("Template configuration is invalid."))?;
        validate_template_configuration(&configuration)?;
        let version_number: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(version_number),0)+1 FROM document_template_versions WHERE company_id=?1 AND document_template_id=?2 AND locale=?3",
            params![context.company_id, draft.0, draft.2],
            |row| row.get(0),
        )?;
        let version_id = new_id();
        let content_hash = template_content_hash(&draft.0, &draft.1, &draft.2, version_number, &draft.4);
        let now = now_iso()?;
        transaction.execute(
            r#"INSERT INTO document_template_versions(
                   id,company_id,document_template_id,version_number,html_template,css_template,
                   content_hash_sha256,is_published,created_at,created_by,locale,configuration_json,published_at,published_by
               ) VALUES(?1,?2,?3,?4,'POSMAN_STRUCTURED_TEMPLATE_V1','POSMAN_A4_SAFE_CSS_V1',?5,1,?6,?7,?8,?9,?6,?7)"#,
            params![version_id, context.company_id, draft.0, version_number, content_hash, now, context.user_id, draft.2, draft.4],
        )?;
        transaction.execute(
            r#"UPDATE document_template_publications
               SET status='RETIRED',retired_at=?1,retired_by=?2,row_version=row_version+1
               WHERE company_id=?3 AND document_template_id=?4 AND locale=?5 AND status='PUBLISHED'"#,
            params![now, context.user_id, context.company_id, draft.0, draft.2],
        )?;
        transaction.execute(
            r#"INSERT INTO document_template_publications(
                   id,company_id,document_template_id,template_version_id,locale,status,
                   activated_at,activated_by,row_version
               ) VALUES(?1,?2,?3,?4,?5,'PUBLISHED',?6,?7,1)"#,
            params![new_id(), context.company_id, draft.0, version_id, draft.2, now, context.user_id],
        )?;
        transaction.execute(
            "UPDATE document_template_drafts SET status='PUBLISHED',updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE id=?3 AND company_id=?4 AND row_version=?5",
            params![now, context.user_id, request.draft_id, context.company_id, request.expected_row_version],
        )?;
        transaction.execute(
            "UPDATE document_templates SET name_ar=CASE WHEN ?1='ar-DZ' THEN ?2 ELSE name_ar END,name_fr=CASE WHEN ?1='fr-DZ' THEN ?2 ELSE name_fr END,updated_at=?3,updated_by=?4,row_version=row_version+1 WHERE id=?5 AND company_id=?6",
            params![draft.2, draft.3, now, context.user_id, draft.0, context.company_id],
        )?;
        Self::audit_success(
            &transaction,
            &context,
            "DOCUMENT_TEMPLATE_PUBLISHED",
            "DOCUMENT_TEMPLATE_VERSION",
            &version_id,
            Some(&serde_json::json!({"versionNumber": version_number, "contentSha256": content_hash})),
        )?;
        transaction.commit()?;
        Ok(TemplateVersionView {
            version_id,
            version_number,
            locale: draft.2,
            content_sha256: content_hash,
            status: "PUBLISHED".into(),
            published_at: now,
            published_by: context.user_id,
            row_version: 1,
        })
    }

    pub fn retire_template(&self, request: RetireTemplateRequest) -> Phase09Result<TemplateVersionView> {
        if !request.confirmed || request.expected_row_version < 1 {
            return Err(Phase09Error::validation(
                "Template retirement requires explicit confirmation.",
            ));
        }
        let context = self.authorize("documents.templates.manage")?;
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = transaction
            .query_row(
                r#"SELECT p.id,p.row_version,p.status,v.version_number,v.locale,v.content_hash_sha256,
                          COALESCE(v.published_at,v.created_at),COALESCE(v.published_by,v.created_by,''),v.document_template_id
                   FROM document_template_publications p
                   JOIN document_template_versions v ON v.id=p.template_version_id
                   WHERE p.company_id=?1 AND p.template_version_id=?2"#,
                params![context.company_id, request.template_version_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| Phase09Error::not_found("template version"))?;
        if current.1 != request.expected_row_version {
            return Err(Phase09Error::concurrency());
        }
        if current.2 == "RETIRED" {
            transaction.commit()?;
            return Ok(TemplateVersionView {
                version_id: request.template_version_id,
                version_number: current.3,
                locale: current.4,
                content_sha256: current.5,
                status: current.2,
                published_at: current.6,
                published_by: current.7,
                row_version: current.1,
            });
        }
        let replacement_count: i64 = transaction.query_row(
            r#"SELECT COUNT(*) FROM document_template_publications p
               JOIN document_template_versions v ON v.id=p.template_version_id
               WHERE p.company_id=?1 AND v.document_template_id=?2 AND v.locale=?3
                 AND p.template_version_id<>?4 AND p.status='PUBLISHED'"#,
            params![
                context.company_id,
                current.8,
                current.4,
                request.template_version_id
            ],
            |row| row.get(0),
        )?;
        if replacement_count == 0 {
            return Err(Phase09Error::new(
                "TEMPLATE_REPLACEMENT_REQUIRED",
                "Publish a replacement before retiring the active version.",
                false,
            ));
        }
        let now = now_iso()?;
        let changed = transaction.execute(
            "UPDATE document_template_publications SET status='RETIRED',retired_at=?1,retired_by=?2,row_version=row_version+1 WHERE id=?3 AND company_id=?4 AND status='PUBLISHED' AND row_version=?5",
            params![
                now,
                context.user_id,
                current.0,
                context.company_id,
                request.expected_row_version
            ],
        )?;
        if changed != 1 {
            return Err(Phase09Error::concurrency());
        }
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
            version_id: request.template_version_id,
            version_number: current.3,
            locale: current.4,
            content_sha256: current.5,
            status: "RETIRED".into(),
            published_at: current.6,
            published_by: current.7,
            row_version: current.1 + 1,
        })
    }

    pub(crate) fn ensure_company_templates(&self, company_id: &str, user_id: &str) -> Phase09Result<()> {
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = now_iso()?;
        let mut defaults = transaction.prepare(
            "SELECT document_type,locale,display_name,configuration_json FROM document_template_defaults ORDER BY document_type,locale",
        )?;
        let rows = defaults.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        drop(defaults);
        for (document_type, locale, display_name, configuration_json) in rows {
            let existing = transaction.query_row(
                "SELECT id FROM document_templates WHERE company_id=?1 AND document_type=?2 AND locale=?3",
                params![company_id, document_type, locale],
                |row| row.get::<_, String>(0),
            ).optional()?;
            let template_id = existing.unwrap_or_else(new_id);
            transaction.execute(
                r#"INSERT OR IGNORE INTO document_templates(
                       id,company_id,code,document_type,name_ar,name_fr,is_active,
                       created_at,created_by,updated_at,updated_by,row_version,locale
                   ) VALUES(?1,?2,?3,?4,?5,?5,1,?6,?7,?6,?7,1,?8)"#,
                params![template_id, company_id, format!("{}-{}", document_type.to_ascii_lowercase(), locale), document_type, display_name, now, user_id, locale],
            )?;
            let active = transaction.query_row(
                "SELECT 1 FROM document_template_publications WHERE company_id=?1 AND document_template_id=?2 AND locale=?3 AND status='PUBLISHED'",
                params![company_id, template_id, locale],
                |row| row.get::<_, i64>(0),
            ).optional()?.is_some();
            if !active {
                let version_id = new_id();
                let content_hash = template_content_hash(&template_id, &document_type, &locale, 1, &configuration_json);
                transaction.execute(
                    r#"INSERT INTO document_template_versions(
                           id,company_id,document_template_id,version_number,html_template,css_template,
                           content_hash_sha256,is_published,created_at,created_by,locale,configuration_json,published_at,published_by
                       ) VALUES(?1,?2,?3,1,'POSMAN_STRUCTURED_TEMPLATE_V1','POSMAN_A4_SAFE_CSS_V1',?4,1,?5,?6,?7,?8,?5,?6)"#,
                    params![version_id, company_id, template_id, content_hash, now, user_id, locale, configuration_json],
                )?;
                transaction.execute(
                    r#"INSERT INTO document_template_publications(
                           id,company_id,document_template_id,template_version_id,locale,status,activated_at,activated_by,row_version
                       ) VALUES(?1,?2,?3,?4,?5,'PUBLISHED',?6,?7,1)"#,
                    params![new_id(), company_id, template_id, version_id, locale, now, user_id],
                )?;
            }
        }
        transaction.commit()?;
        Ok(())
    }
}

const TEMPLATE_SUMMARY_SQL: &str = r#"
SELECT t.id,t.document_type,t.locale,
       CASE WHEN t.locale='ar-DZ' THEN t.name_ar ELSE COALESCE(t.name_fr,t.name_ar) END,
       v.id,v.version_number,v.content_hash_sha256,d.id,d.row_version,
       CASE WHEN d.id IS NOT NULL THEN 'DRAFT' WHEN p.status='PUBLISHED' THEN 'PUBLISHED' ELSE 'RETIRED' END
FROM document_templates t
LEFT JOIN document_template_publications p
  ON p.company_id=t.company_id AND p.document_template_id=t.id AND p.locale=t.locale AND p.status='PUBLISHED'
LEFT JOIN document_template_versions v ON v.id=p.template_version_id
LEFT JOIN document_template_drafts d
  ON d.company_id=t.company_id AND d.document_template_id=t.id AND d.locale=t.locale AND d.status='DRAFT'
WHERE t.company_id=?1
"#;

fn map_template_summary(row: &Row<'_>) -> rusqlite::Result<TemplateSummary> {
    Ok(TemplateSummary {
        template_id: row.get(0)?,
        document_type: row.get(1)?,
        locale: row.get(2)?,
        display_name: row.get(3)?,
        active_version_id: row.get(4)?,
        active_version_number: row.get(5)?,
        active_content_sha256: row.get(6)?,
        draft_id: row.get(7)?,
        draft_row_version: row.get(8)?,
        state: row.get(9)?,
    })
}

fn map_template_draft(row: &Row<'_>) -> rusqlite::Result<TemplateDraftView> {
    let configuration_json: String = row.get(5)?;
    let configuration = serde_json::from_str(&configuration_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(TemplateDraftView {
        draft_id: row.get(0)?,
        template_id: row.get(1)?,
        document_type: row.get(2)?,
        locale: row.get(3)?,
        display_name: row.get(4)?,
        configuration,
        base_template_version_id: row.get(6)?,
        row_version: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn validate_document_type(value: &str) -> Phase09Result<()> {
    if DOCUMENT_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(Phase09Error::validation("Unsupported document type."))
    }
}

pub(crate) fn source_document_type(value: &str) -> Phase09Result<(&'static str, &'static str)> {
    match value {
        "SALES_ORDER" => Ok(("COMMERCIAL_DOCUMENT", "SALES_ORDER")),
        "DELIVERY_NOTE" => Ok(("COMMERCIAL_DOCUMENT", "DELIVERY_NOTE")),
        "SALES_INVOICE" => Ok(("COMMERCIAL_DOCUMENT", "SALES_INVOICE")),
        "SALES_CREDIT_NOTE" => Ok(("COMMERCIAL_DOCUMENT", "SALES_CREDIT_NOTE")),
        "PURCHASE_ORDER" => Ok(("COMMERCIAL_DOCUMENT", "PURCHASE_ORDER")),
        "GOODS_RECEIPT" => Ok(("COMMERCIAL_DOCUMENT", "PURCHASE_RECEIPT")),
        "SUPPLIER_INVOICE" => Ok(("COMMERCIAL_DOCUMENT", "PURCHASE_INVOICE")),
        "PURCHASE_RETURN" => Ok(("COMMERCIAL_DOCUMENT", "PURCHASE_RETURN")),
        "CUSTOMER_RECEIPT" => Ok(("PAYMENT", "CUSTOMER_RECEIPT")),
        "SUPPLIER_PAYMENT" => Ok(("PAYMENT", "SUPPLIER_PAYMENT")),
        _ => Err(Phase09Error::validation("Unsupported document type.")),
    }
}

fn template_content_hash(
    template_id: &str,
    document_type: &str,
    locale: &str,
    version_number: i64,
    configuration_json: &str,
) -> String {
    let mut digest = Sha256::new();
    digest.update(template_id.as_bytes());
    digest.update([0]);
    digest.update(document_type.as_bytes());
    digest.update([0]);
    digest.update(locale.as_bytes());
    digest.update([0]);
    digest.update(version_number.to_be_bytes());
    digest.update([0]);
    digest.update(configuration_json.as_bytes());
    format!("{:x}", digest.finalize())
}
