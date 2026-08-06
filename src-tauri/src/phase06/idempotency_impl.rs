pub(crate) fn now_iso() -> Phase06Result<String> {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| Phase06Error::internal())
}

pub(crate) fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

pub(crate) fn request_hash<T: Serialize>(value: &T) -> Phase06Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|_| Phase06Error::invalid("request"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub(crate) fn validate_idempotency_key(value: &str) -> Phase06Result<&str> {
    let trimmed = value.trim();
    if !(8..=160).contains(&trimmed.len()) {
        return Err(Phase06Error::invalid("idempotencyKey"));
    }
    Ok(trimmed)
}

pub(crate) fn document_idempotency_key(
    namespace: &str,
    key: &str,
) -> Phase06Result<String> {
    Ok(format!("{namespace}:{}", validate_idempotency_key(key)?))
}

pub(crate) enum IdempotencyStart {
    New,
    Replayed(String),
}

pub(crate) fn begin_idempotency(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    namespace: &str,
    key: &str,
    hash: &str,
) -> Phase06Result<IdempotencyStart> {
    let key = validate_idempotency_key(key)?;
    let existing = transaction
        .query_row(
            r#"
            SELECT request_hash_sha256, status, result_entity_id
            FROM idempotency_keys
            WHERE company_id=?1 AND namespace=?2 AND idempotency_key=?3
            "#,
            params![context.company_id, namespace, key],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?;

    if let Some((existing_hash, status, result_entity_id)) = existing {
        if existing_hash != hash {
            return Err(Phase06Error::idempotency_conflict());
        }
        if status == "SUCCEEDED" {
            return result_entity_id
                .map(IdempotencyStart::Replayed)
                .ok_or_else(Phase06Error::internal);
        }
        return Err(Phase06Error::new(
            "REQUEST_IN_PROGRESS",
            "The same request is already in progress.",
        ));
    }

    transaction.execute(
        r#"
        INSERT INTO idempotency_keys (
            id, company_id, namespace, idempotency_key,
            request_hash_sha256, status, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'IN_PROGRESS', ?6)
        "#,
        params![new_id(), context.company_id, namespace, key, hash, now_iso()?],
    )?;
    Ok(IdempotencyStart::New)
}

pub(crate) fn finish_idempotency(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    namespace: &str,
    key: &str,
    entity_type: &str,
    entity_id: &str,
) -> Phase06Result<()> {
    let validated_key = validate_idempotency_key(key)?;
    if entity_type == "commercial_document"
        && crate::phase08::accounting_enabled_in_tx(transaction, &context.company_id)
            .map_err(|error| Phase06Error::new(&error.code, &error.message))?
    {
        if let Some((source_event_type, include_stock_cost)) =
            crate::phase08::commercial_event_plan(namespace)
        {
            let source = crate::phase08::document_source_event_in_tx(
                transaction,
                &context.company_id,
                entity_id,
                source_event_type,
                entity_id,
                &transaction.query_row(
                    "SELECT commercial_date FROM commercial_documents WHERE id=?1 AND company_id=?2",
                    params![entity_id, context.company_id],
                    |row| row.get::<_, String>(0),
                )?,
                include_stock_cost,
            )
            .map_err(|error| Phase06Error::new(&error.code, &error.message))?;
            let accounting_request = crate::phase08::dto::Idempotent {
                idempotency_key: format!("{namespace}:{validated_key}:accounting"),
                request_hash_sha256: crate::phase08::request_hash(&source)
                    .map_err(|error| Phase06Error::new(&error.code, &error.message))?,
                payload: source,
            };
            if let Err(error) = crate::phase08::post_source_event_in_tx(
                transaction,
                context,
                &accounting_request,
            ) {
                let database_path = transaction
                    .query_row(
                        "SELECT file FROM pragma_database_list WHERE name='main'",
                        [],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()?
                    .filter(|path| !path.trim().is_empty());
                return Err(crate::phase08::phase06_error(
                    error,
                    context,
                    accounting_request,
                    database_path,
                ));
            }
        }
    }

    let changed = transaction.execute(
        r#"
        UPDATE idempotency_keys
        SET status='SUCCEEDED', result_entity_type=?1, result_entity_id=?2,
            completed_at=?3
        WHERE company_id=?4 AND namespace=?5 AND idempotency_key=?6
          AND status='IN_PROGRESS'
        "#,
        params![
            entity_type,
            entity_id,
            now_iso()?,
            context.company_id,
            namespace,
            validated_key
        ],
    )?;
    if changed != 1 {
        return Err(Phase06Error::internal());
    }
    Ok(())
}

pub(crate) fn audit(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details_json: Option<&str>,
) -> Phase06Result<()> {
    transaction.execute(
        r#"
        INSERT INTO audit_logs (
            id, company_id, actor_user_id, action_code, entity_type,
            entity_id, occurred_at, outcome, correlation_id, details_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'SUCCESS', ?8, ?9)
        "#,
        params![
            new_id(),
            context.company_id,
            context.user_id,
            action,
            entity_type,
            entity_id,
            now_iso()?,
            context.session_id,
            details_json
        ],
    )?;
    Ok(())
}
